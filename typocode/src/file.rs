//! Source-file loading.
//!
//! Reads a UTF-8 source file from disk, expands tabs to [`TAB_WIDTH`]
//! spaces, normalises a handful of un-typeable typographic codepoints
//! to their ASCII equivalents, and rejects empty inputs — matching the
//! semantics of the original C version's `file/file.c` and
//! `buffer/buffer.c`, with the normalisation added because strict-match
//! typing would otherwise stall on characters a standard keyboard
//! can't produce (em dash, smart quotes, NBSP).

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

/// Loads `path` as UTF-8, expands tabs, and validates the result.
///
/// # Errors
///
/// Fails on I/O errors, non-UTF-8 content, or when the resulting
/// expanded text is empty — you can't play a typing game on a blank
/// file.
pub fn load(path: &Path) -> Result<SourceFile> {
    let raw = fs::read_to_string(path)
        .wrap_err_with(|| format!("failed to read `{}`", path.display()))?;
    let mut source = parse(&raw)?;
    source.display_name = basename(path);
    Ok(source)
}

/// Pure helper: expands tabs in `raw`, counts source lines, and returns
/// a [`SourceFile`] with an empty `display_name`. Exposed at crate level
/// so tests can exercise the expansion logic without touching disk.
pub(crate) fn parse(raw: &str) -> Result<SourceFile> {
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
        assert_eq!(parsed.content, vec!['c', 'a', 'f', 'é', '\n']);
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
}
