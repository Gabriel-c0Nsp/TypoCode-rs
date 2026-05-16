//! Source-file loading.
//!
//! Reads a UTF-8 source file from disk, trims trailing blank lines and
//! whitespace so the run can finish the moment the last visible
//! character is typed, expands tabs to [`TAB_WIDTH`] spaces, normalises
//! a handful of un-typeable typographic codepoints to their ASCII
//! equivalents, and rejects empty inputs — matching the semantics of
//! the original C version's `file/file.c` and `buffer/buffer.c`, with
//! the normalisation added because strict-match typing would otherwise
//! stall on characters a standard keyboard can't produce (em dash,
//! smart quotes, NBSP).

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use color_eyre::eyre::{Context, Result, bail};

/// Number of spaces each `\t` in the source expands to.
pub const TAB_WIDTH: usize = 2;

/// Rewrites a typographic character to an ASCII equivalent that can
/// actually be typed on a standard keyboard. Unhandled codepoints pass
/// through unchanged so legitimate source-code content (comparison
/// operators, arrows written as `->`, etc.) stays intact.
fn normalize_char(c: char) -> char {
    match c {
        '\u{2013}' | '\u{2014}' => '-',  // en/em dashes
        '\u{2018}' | '\u{2019}' => '\'', // curly single quotes
        '\u{201C}' | '\u{201D}' => '"',  // curly double quotes
        '\u{00A0}' | '\u{202F}' => ' ',  // no-break / narrow no-break
        _ => c,
    }
}

/// A source file loaded into memory with tabs expanded.
///
/// `content` is the expanded character sequence used by the typing
/// engine. `line_count` is the number of logical source lines (one per
/// newline in the raw input, counted via [`str::lines`]). `display_name`
/// is the basename shown in the header widget — never used for file I/O.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub display_name: String,
    pub content: Vec<char>,
    pub line_count: usize,
}

/// Optional behaviours applied during [`load`]. Grouped into a struct
/// so additional preprocessing knobs (line caps, include expansion,
/// etc.) can land without churning the public signature.
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadOptions {
    /// When `true`, comments are removed from the raw text before any
    /// other processing. Languages are detected from the file
    /// extension; unknown extensions silently pass through.
    pub strip_comments: bool,
}

/// Loads `path` as UTF-8, optionally strips comments, expands tabs,
/// and validates the result.
///
/// # Errors
///
/// Fails on I/O errors, non-UTF-8 content, or when the resulting
/// expanded text is empty — you can't play a typing game on a blank
/// file.
pub fn load(path: &Path, opts: LoadOptions) -> Result<SourceFile> {
    let raw = fs::read_to_string(path)
        .wrap_err_with(|| format!("failed to read `{}`", path.display()))?;
    let raw = if opts.strip_comments {
        let ext = path.extension().and_then(OsStr::to_str);
        crate::comments::strip_by_extension(&raw, ext).unwrap_or(raw)
    } else {
        raw
    };
    let mut source = parse(&raw)?;
    source.display_name = basename(path);
    Ok(source)
}

/// Pure helper: trims trailing blank lines and whitespace, expands
/// tabs in `raw`, counts source lines, and returns a [`SourceFile`]
/// with an empty `display_name`. Exposed at crate level so tests can
/// exercise the parsing logic without touching disk.
pub(crate) fn parse(raw: &str) -> Result<SourceFile> {
    // Trim trailing blank lines and whitespace so the final cell is a
    // visible character: this lets the run auto-finish the moment the
    // player types it correctly, instead of forcing them to hit Enter
    // through every trailing newline.
    let raw = raw.trim_end_matches(['\n', '\r', ' ', '\t']);

    let mut content = Vec::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '\t' => content.extend(std::iter::repeat_n(' ', TAB_WIDTH)),
            // CRLF → LF: drop the CR so the cell stream has a single
            // newline character and Enter matches it.
            '\r' => {}
            _ => content.push(normalize_char(c)),
        }
    }

    if content.is_empty() {
        bail!("cannot play with an empty file");
    }

    let line_count = raw.lines().count();

    Ok(SourceFile {
        display_name: String::new(),
        content,
        line_count,
    })
}

fn basename(path: &Path) -> String {
    path.file_name()
        .map(|os| os.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tabs_to_two_spaces() {
        let parsed = parse("a\tb").unwrap();
        assert_eq!(parsed.content, vec!['a', ' ', ' ', 'b']);
    }

    #[test]
    fn expands_back_to_back_tabs() {
        let parsed = parse("x\t\ty").unwrap();
        assert_eq!(parsed.content, vec!['x', ' ', ' ', ' ', ' ', 'y']);
    }

    #[test]
    fn rejects_empty_input() {
        let err = parse("").unwrap_err();
        assert!(
            err.to_string().contains("empty"),
            "expected empty-file error, got: {err}"
        );
    }

    #[test]
    fn preserves_utf8_multibyte_chars() {
        let parsed = parse("café\n").unwrap();
        assert_eq!(parsed.content, vec!['c', 'a', 'f', 'é']);
    }

    #[test]
    fn counts_logical_source_lines() {
        assert_eq!(parse("single").unwrap().line_count, 1);
        assert_eq!(parse("a\nb\nc").unwrap().line_count, 3);
        assert_eq!(parse("a\nb\nc\n").unwrap().line_count, 3);
    }

    #[test]
    fn normalizes_em_and_en_dashes_to_hyphen() {
        let parsed = parse("a\u{2014}b\u{2013}c").unwrap();
        assert_eq!(parsed.content, vec!['a', '-', 'b', '-', 'c']);
    }

    #[test]
    fn normalizes_smart_quotes_to_straight_quotes() {
        let parsed = parse("\u{201C}hi\u{201D} \u{2018}x\u{2019}").unwrap();
        assert_eq!(
            parsed.content,
            vec!['"', 'h', 'i', '"', ' ', '\'', 'x', '\'']
        );
    }

    #[test]
    fn normalizes_non_breaking_spaces_to_regular_spaces() {
        let parsed = parse("a\u{00A0}b\u{202F}c").unwrap();
        assert_eq!(parsed.content, vec!['a', ' ', 'b', ' ', 'c']);
    }

    #[test]
    fn leaves_unrelated_characters_untouched() {
        let parsed = parse("café-→").unwrap();
        assert_eq!(parsed.content, vec!['c', 'a', 'f', 'é', '-', '→']);
    }

    #[test]
    fn strips_carriage_returns_to_normalize_crlf() {
        let parsed = parse("a\r\nb\r\nc").unwrap();
        assert_eq!(parsed.content, vec!['a', '\n', 'b', '\n', 'c']);
    }

    #[test]
    fn strips_standalone_carriage_returns() {
        let parsed = parse("a\rb").unwrap();
        assert_eq!(parsed.content, vec!['a', 'b']);
    }

    #[test]
    fn strips_trailing_blank_lines() {
        let parsed = parse("abc\n\n\n").unwrap();
        assert_eq!(parsed.content, vec!['a', 'b', 'c']);
        assert_eq!(parsed.line_count, 1);
    }

    #[test]
    fn strips_trailing_whitespace_inside_last_line() {
        let parsed = parse("abc   \t  ").unwrap();
        assert_eq!(parsed.content, vec!['a', 'b', 'c']);
    }

    #[test]
    fn strips_mixed_trailing_blank_lines_and_whitespace() {
        let parsed = parse("a\nb\n   \n\t\n").unwrap();
        assert_eq!(parsed.content, vec!['a', '\n', 'b']);
        assert_eq!(parsed.line_count, 2);
    }

    #[test]
    fn preserves_internal_blank_lines() {
        let parsed = parse("a\n\nb\n").unwrap();
        assert_eq!(parsed.content, vec!['a', '\n', '\n', 'b']);
        assert_eq!(parsed.line_count, 3);
    }

    #[test]
    fn whitespace_only_file_rejected_as_empty() {
        let err = parse("\n\n   \t\n").unwrap_err();
        assert!(
            err.to_string().contains("empty"),
            "expected empty-file error, got: {err}"
        );
    }

    fn write_temp_file(name: &str, body: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("typocode-test-{}-{}", std::process::id(), name));
        fs::write(&path, body).expect("write tempfile");
        path
    }

    #[test]
    fn load_with_strip_comments_removes_rust_comments() {
        let path = write_temp_file(
            "strip.rs",
            "// header\nfn main() {\n    println!(\"hi\");\n}\n",
        );
        let source = load(
            &path,
            LoadOptions {
                strip_comments: true,
            },
        )
        .unwrap();
        let text: String = source.content.iter().collect();
        assert!(!text.contains("header"));
        assert!(text.contains("fn main"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_with_strip_comments_passes_through_unknown_extension() {
        let path = write_temp_file("passthrough.unknownext", "// keep me\nbody\n");
        let source = load(
            &path,
            LoadOptions {
                strip_comments: true,
            },
        )
        .unwrap();
        let text: String = source.content.iter().collect();
        assert!(text.contains("keep me"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_without_strip_comments_keeps_comments() {
        let path = write_temp_file("keep.rs", "// keep\nfn main() {}\n");
        let source = load(&path, LoadOptions::default()).unwrap();
        let text: String = source.content.iter().collect();
        assert!(text.contains("keep"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_with_strip_comments_drops_leading_comment_and_separator() {
        let path = write_temp_file("leading.rs", "// header\n\nfn main() {\n    body();\n}\n");
        let source = load(
            &path,
            LoadOptions {
                strip_comments: true,
            },
        )
        .unwrap();
        assert_eq!(
            source.content.first(),
            Some(&'f'),
            "expected first cell to be code, got: {:?}",
            source.content.first()
        );
        assert_ne!(source.content.first(), Some(&'\n'));
        let _ = fs::remove_file(&path);
    }
}
