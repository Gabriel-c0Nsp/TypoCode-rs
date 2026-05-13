//! Core comment-stripping engine.
//!
//! Two passes:
//! 1. A state-machine scan classifies every byte of the input as
//!    either kept (code) or dropped (comment). String literals are
//!    classified as code so a comment-looking marker that lives
//!    inside data never triggers stripping.
//! 2. A line-pruning post-pass walks the input line by line: if a line
//!    contained any comment-classified byte AND nothing but whitespace
//!    survives the filter, the entire line plus its trailing newline
//!    disappears; otherwise the surviving code is emitted with trailing
//!    whitespace trimmed.
//!
//! The post-pass is what turns `// header\n` into nothing at all,
//! instead of an empty line the player would have to Enter through.

use super::spec::{BlockComment, LanguageSpec, StringLiteral};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Class {
    Code,
    Comment,
}

/// Removes comments from `raw` according to `spec`.
pub fn strip(raw: &str, spec: &LanguageSpec) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let mut class = vec![Class::Code; raw.len()];
    let mut i = 0;
    while i < raw.len() {
        i = step(raw, i, &mut class, spec);
    }
    emit(raw, &class)
}

fn step(raw: &str, i: usize, class: &mut [Class], spec: &LanguageSpec) -> usize {
    if let Some(end) = try_string(raw, i, spec) {
        return end;
    }
    if let Some(end) = try_block_comment(raw, i, class, spec) {
        return end;
    }
    if let Some(end) = try_line_comment(raw, i, class, spec) {
        return end;
    }
    i + char_len(raw, i)
}

fn try_line_comment(
    raw: &str,
    i: usize,
    class: &mut [Class],
    spec: &LanguageSpec,
) -> Option<usize> {
    let rest = &raw[i..];
    spec.line_comments
        .iter()
        .find(|prefix| rest.starts_with(**prefix))?;
    let end = match rest.find('\n') {
        Some(n) => i + n + 1,
        None => raw.len(),
    };
    for slot in &mut class[i..end] {
        *slot = Class::Comment;
    }
    Some(end)
}

fn try_block_comment(
    raw: &str,
    i: usize,
    class: &mut [Class],
    spec: &LanguageSpec,
) -> Option<usize> {
    for block in spec.block_comments {
        if let Some(end) = match_block_comment(raw, i, block) {
            for slot in &mut class[i..end] {
                *slot = Class::Comment;
            }
            return Some(end);
        }
    }
    None
}

fn match_block_comment(raw: &str, i: usize, block: &BlockComment) -> Option<usize> {
    if block.long_bracket {
        return match_long_bracket_block(raw, i, block);
    }
    let rest = &raw[i..];
    if !rest.starts_with(block.start) {
        return None;
    }
    let mut depth = 1usize;
    let mut p = i + block.start.len();
    while p < raw.len() && depth > 0 {
        let sub = &raw[p..];
        if block.nestable && sub.starts_with(block.start) {
            depth += 1;
            p += block.start.len();
        } else if sub.starts_with(block.end) {
            depth -= 1;
            p += block.end.len();
        } else {
            p += char_len(raw, p);
        }
    }
    Some(p)
}

fn match_long_bracket_block(raw: &str, i: usize, block: &BlockComment) -> Option<usize> {
    let rest = &raw[i..];
    let after_prefix = rest.strip_prefix(block.start)?;
    let (open_extra, eq_count) = parse_long_bracket_open(after_prefix)?;
    let open_len = block.start.len() + open_extra;
    let body_start = i + open_len;
    Some(find_long_bracket_close(raw, body_start, eq_count))
}

fn try_string(raw: &str, i: usize, spec: &LanguageSpec) -> Option<usize> {
    for variant in spec.strings {
        if let Some(end) = match_string(raw, i, variant) {
            return Some(end);
        }
    }
    None
}

fn match_string(raw: &str, i: usize, variant: &StringLiteral) -> Option<usize> {
    match *variant {
        StringLiteral::Simple { delim, escape } => match_simple_string(raw, i, delim, escape),
        StringLiteral::Triple { delim } => match_triple_string(raw, i, delim),
        StringLiteral::Backtick => match_backtick_string(raw, i),
        StringLiteral::RawHash { prefix, delim } => match_raw_hash_string(raw, i, prefix, delim),
        StringLiteral::LongBracket => match_long_bracket_string(raw, i),
    }
}

fn match_simple_string(raw: &str, i: usize, delim: char, escape: Option<char>) -> Option<usize> {
    let rest = &raw[i..];
    let mut chars = rest.chars();
    if chars.next()? != delim {
        return None;
    }
    let mut p = i + delim.len_utf8();
    while p < raw.len() {
        let ch = raw[p..].chars().next()?;
        if Some(ch) == escape {
            p += ch.len_utf8();
            if p < raw.len() {
                p += char_len(raw, p);
            }
            continue;
        }
        if ch == delim {
            return Some(p + ch.len_utf8());
        }
        if ch == '\n' {
            // Most simple strings don't span lines; bail so we don't
            // swallow whole files on an unterminated quote.
            return Some(p);
        }
        p += ch.len_utf8();
    }
    Some(p)
}

fn match_triple_string(raw: &str, i: usize, delim: char) -> Option<usize> {
    let triple: String = std::iter::repeat_n(delim, 3).collect();
    if !raw[i..].starts_with(&triple) {
        return None;
    }
    let mut p = i + triple.len();
    while p < raw.len() {
        if raw[p..].starts_with(&triple) {
            return Some(p + triple.len());
        }
        p += char_len(raw, p);
    }
    Some(p)
}

fn match_backtick_string(raw: &str, i: usize) -> Option<usize> {
    if !raw[i..].starts_with('`') {
        return None;
    }
    let mut p = i + 1;
    while p < raw.len() {
        let ch = raw[p..].chars().next()?;
        if ch == '\\' {
            p += 1;
            if p < raw.len() {
                p += char_len(raw, p);
            }
            continue;
        }
        if ch == '`' {
            return Some(p + 1);
        }
        p += ch.len_utf8();
    }
    Some(p)
}

fn match_raw_hash_string(raw: &str, i: usize, prefix: &str, delim: char) -> Option<usize> {
    let rest = &raw[i..];
    let after_prefix = rest.strip_prefix(prefix)?;
    let after_bytes = after_prefix.as_bytes();
    let mut h = 0;
    while h < after_bytes.len() && after_bytes[h] == b'#' {
        h += 1;
    }
    let next_ch = after_prefix[h..].chars().next()?;
    if next_ch != delim {
        return None;
    }
    let open_len = prefix.len() + h + delim.len_utf8();
    let mut p = i + open_len;
    let close: String = std::iter::once(delim)
        .chain(std::iter::repeat_n('#', h))
        .collect();
    while p < raw.len() {
        if raw[p..].starts_with(&close) {
            return Some(p + close.len());
        }
        p += char_len(raw, p);
    }
    Some(p)
}

fn match_long_bracket_string(raw: &str, i: usize) -> Option<usize> {
    let rest = &raw[i..];
    let (open_extra, eq_count) = parse_long_bracket_open(rest)?;
    let body_start = i + open_extra;
    Some(find_long_bracket_close(raw, body_start, eq_count))
}

/// Tries to read `[=*[` at the start of `s`. On success returns
/// `(consumed_bytes, eq_count)`.
fn parse_long_bracket_open(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    if bytes.first()? != &b'[' {
        return None;
    }
    let mut p = 1;
    while p < bytes.len() && bytes[p] == b'=' {
        p += 1;
    }
    if bytes.get(p)? != &b'[' {
        return None;
    }
    let eq_count = p - 1;
    Some((p + 1, eq_count))
}

/// Returns the absolute index just past the matching `]=*]`, or
/// `raw.len()` if the close is missing.
fn find_long_bracket_close(raw: &str, body_start: usize, eq_count: usize) -> usize {
    let close: String = std::iter::once(']')
        .chain(std::iter::repeat_n('=', eq_count))
        .chain(std::iter::once(']'))
        .collect();
    let mut p = body_start;
    while p < raw.len() {
        if raw[p..].starts_with(&close) {
            return p + close.len();
        }
        p += char_len(raw, p);
    }
    raw.len()
}

fn char_len(raw: &str, i: usize) -> usize {
    raw[i..].chars().next().map_or(1, |c| c.len_utf8())
}

fn emit(raw: &str, class: &[Class]) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut line_start = 0;
    let mut i = 0;
    loop {
        let at_eof = i == bytes.len();
        let at_newline = !at_eof && bytes[i] == b'\n';
        if at_eof || at_newline {
            let newline_in_comment = at_newline && class[i] == Class::Comment;
            let body_had_comment = class[line_start..i].contains(&Class::Comment);
            let had_comment = newline_in_comment || body_had_comment;

            let kept = filter_code(raw, class, line_start, i);
            let trimmed_end = kept.trim_end();

            if had_comment && trimmed_end.is_empty() {
                // Drop the whole line, newline included.
            } else {
                out.push_str(trimmed_end);
                if at_newline {
                    out.push('\n');
                }
            }

            if at_eof {
                break;
            }
            i += 1;
            line_start = i;
            continue;
        }
        i += 1;
    }
    out
}

fn filter_code(raw: &str, class: &[Class], start: usize, end: usize) -> String {
    let mut out = String::with_capacity(end - start);
    let mut j = start;
    while j < end {
        let len = char_len(raw, j);
        if class[j] == Class::Code {
            out.push_str(&raw[j..j + len]);
        }
        j += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN_BLOCK: BlockComment = BlockComment {
        start: "/*",
        end: "*/",
        nestable: false,
        long_bracket: false,
    };
    const NESTABLE_BLOCK: BlockComment = BlockComment {
        start: "/*",
        end: "*/",
        nestable: true,
        long_bracket: false,
    };
    const LONG_BRACKET_BLOCK: BlockComment = BlockComment {
        start: "--",
        end: "",
        nestable: false,
        long_bracket: true,
    };

    fn c_like() -> LanguageSpec {
        LanguageSpec {
            names: &["test-c"],
            extensions: &["test"],
            line_comments: &["//"],
            block_comments: &[PLAIN_BLOCK],
            strings: &[
                StringLiteral::Simple {
                    delim: '"',
                    escape: Some('\\'),
                },
                StringLiteral::Simple {
                    delim: '\'',
                    escape: Some('\\'),
                },
            ],
        }
    }

    fn rust_like() -> LanguageSpec {
        LanguageSpec {
            names: &["test-rust"],
            extensions: &["test"],
            line_comments: &["//"],
            block_comments: &[NESTABLE_BLOCK],
            strings: &[
                StringLiteral::RawHash {
                    prefix: "r",
                    delim: '"',
                },
                StringLiteral::Simple {
                    delim: '"',
                    escape: Some('\\'),
                },
            ],
        }
    }

    fn python_like() -> LanguageSpec {
        LanguageSpec {
            names: &["test-py"],
            extensions: &["test"],
            line_comments: &["#"],
            block_comments: &[],
            strings: &[
                StringLiteral::Triple { delim: '"' },
                StringLiteral::Triple { delim: '\'' },
                StringLiteral::Simple {
                    delim: '"',
                    escape: Some('\\'),
                },
                StringLiteral::Simple {
                    delim: '\'',
                    escape: Some('\\'),
                },
            ],
        }
    }

    fn lua_like() -> LanguageSpec {
        LanguageSpec {
            names: &["test-lua"],
            extensions: &["test"],
            line_comments: &["--"],
            block_comments: &[LONG_BRACKET_BLOCK],
            strings: &[
                StringLiteral::LongBracket,
                StringLiteral::Simple {
                    delim: '"',
                    escape: Some('\\'),
                },
            ],
        }
    }

    fn js_like() -> LanguageSpec {
        LanguageSpec {
            names: &["test-js"],
            extensions: &["test"],
            line_comments: &["//"],
            block_comments: &[PLAIN_BLOCK],
            strings: &[
                StringLiteral::Backtick,
                StringLiteral::Simple {
                    delim: '"',
                    escape: Some('\\'),
                },
                StringLiteral::Simple {
                    delim: '\'',
                    escape: Some('\\'),
                },
            ],
        }
    }

    #[test]
    fn code_only_passes_through() {
        let input = "let x = 1;\nlet y = 2;\n";
        assert_eq!(strip(input, &c_like()), input);
    }

    #[test]
    fn line_comment_alone_drops_line() {
        let input = "// header\nlet x = 1;\n";
        assert_eq!(strip(input, &c_like()), "let x = 1;\n");
    }

    #[test]
    fn line_comment_after_code_keeps_code() {
        let input = "let x = 1; // tail\nlet y = 2;\n";
        assert_eq!(strip(input, &c_like()), "let x = 1;\nlet y = 2;\n");
    }

    #[test]
    fn indented_line_comment_drops_line() {
        let input = "code;\n    // indented\nmore;\n";
        assert_eq!(strip(input, &c_like()), "code;\nmore;\n");
    }

    #[test]
    fn block_comment_alone_drops_line() {
        let input = "code;\n/* block */\nmore;\n";
        assert_eq!(strip(input, &c_like()), "code;\nmore;\n");
    }

    #[test]
    fn block_comment_inline_keeps_surrounding_code() {
        let input = "a /* hidden */ b\n";
        let out = strip(input, &c_like());
        assert!(out.starts_with("a"));
        assert!(out.contains("b"));
        assert!(!out.contains("hidden"));
        assert!(out.ends_with("\n"));
    }

    #[test]
    fn block_comment_spans_multiple_lines() {
        let input = "code;\n/* line1\n   line2\n   line3 */\nmore;\n";
        assert_eq!(strip(input, &c_like()), "code;\nmore;\n");
    }

    #[test]
    fn block_comment_starting_after_code_keeps_first_line() {
        let input = "code; /* tail\n   continues */\nmore;\n";
        assert_eq!(strip(input, &c_like()), "code;\nmore;\n");
    }

    #[test]
    fn nested_block_balances() {
        let input = "a /* outer /* inner */ outer */ b\n";
        let out = strip(input, &rust_like());
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        assert!(!out.contains("inner"));
        assert!(!out.contains("outer"));
    }

    #[test]
    fn nestable_engine_does_not_descend_when_disabled() {
        // With nestable=false, the first `*/` closes regardless of
        // depth, leaving the trailing `outer */` as code.
        let input = "/* outer /* inner */ outer */\n";
        let out = strip(input, &c_like());
        assert!(out.contains("outer"));
    }

    #[test]
    fn line_comment_inside_simple_string_preserved() {
        let input = "let s = \"http://x\";\n";
        assert_eq!(strip(input, &c_like()), input);
    }

    #[test]
    fn block_comment_marker_inside_simple_string_preserved() {
        let input = "let s = \"/* not a comment */\";\n";
        assert_eq!(strip(input, &c_like()), input);
    }

    #[test]
    fn escaped_quote_does_not_close_simple_string() {
        let input = "let s = \"he said \\\"hi\\\" // ok\";\n";
        assert_eq!(strip(input, &c_like()), input);
    }

    #[test]
    fn triple_quote_string_preserves_hash() {
        let input = "x = \"\"\"contains # but not a comment\"\"\"\n";
        assert_eq!(strip(input, &python_like()), input);
    }

    #[test]
    fn raw_hash_string_preserves_quote_until_matching_hashes() {
        let input = "let s = r#\"has \"quote\" inside\"#;\n";
        assert_eq!(strip(input, &rust_like()), input);
    }

    #[test]
    fn raw_hash_string_preserves_inner_line_comment() {
        let input = "let s = r\"// not a comment\";\n";
        assert_eq!(strip(input, &rust_like()), input);
    }

    #[test]
    fn long_bracket_string_preserves_inner_line_comment_marker() {
        let input = "local s = [==[has -- and ]=] inside]==]\nlocal y = 1\n";
        assert_eq!(strip(input, &lua_like()), input);
    }

    #[test]
    fn long_bracket_block_comment_drops_lines() {
        let input = "code\n--[[ block\n   spans ]]\nmore\n";
        assert_eq!(strip(input, &lua_like()), "code\nmore\n");
    }

    #[test]
    fn long_bracket_block_with_eq_levels() {
        let input = "code\n--[==[ ]] ]=] still inside ]==]\nmore\n";
        assert_eq!(strip(input, &lua_like()), "code\nmore\n");
    }

    #[test]
    fn backtick_string_preserves_comment_markers() {
        let input = "let s = `// not a comment /* also */`;\n";
        assert_eq!(strip(input, &js_like()), input);
    }

    #[test]
    fn file_entirely_comments_yields_empty() {
        let input = "// a\n// b\n/* c */\n";
        assert_eq!(strip(input, &c_like()), "");
    }

    #[test]
    fn trailing_whitespace_on_kept_line_is_trimmed() {
        let input = "code;   // tail\n";
        assert_eq!(strip(input, &c_like()), "code;\n");
    }

    #[test]
    fn no_trailing_newline_handled() {
        let input = "code // tail";
        assert_eq!(strip(input, &c_like()), "code");
    }

    #[test]
    fn unterminated_block_consumes_to_eof() {
        let input = "code\n/* unterminated\n";
        assert_eq!(strip(input, &c_like()), "code\n");
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert_eq!(strip("", &c_like()), "");
    }

    #[test]
    fn preserves_internal_blank_lines() {
        let input = "a\n\nb\n";
        assert_eq!(strip(input, &c_like()), input);
    }

    #[test]
    fn utf8_content_preserved() {
        let input = "// café\nlet x = \"naïve\";\n";
        assert_eq!(strip(input, &c_like()), "let x = \"naïve\";\n");
    }
}
