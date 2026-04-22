//! Source-file loading.
//!
//! Reads a UTF-8 source file from disk, expands tabs to [`TAB_WIDTH`]
//! spaces, and rejects empty inputs — matching the semantics of the
//! original C version's `file/file.c` and `buffer/buffer.c`.

use std::fs;
use std::path::Path;

use color_eyre::eyre::{Context, Result, bail};

/// Number of spaces each `\t` in the source expands to.
pub const TAB_WIDTH: usize = 2;

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
        if c == '\t' {
            content.extend(std::iter::repeat_n(' ', TAB_WIDTH));
        } else {
            content.push(c);
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
}
