//! Visual layout of page content at a given column width.
//!
//! Implements a deterministic character-based wrap: each source line
//! (run of characters ended by `\n`) is chopped into `cols`-wide visual
//! rows. Empty source lines produce exactly one empty visual row so
//! line counting stays in sync between pagination and rendering.
//!
//! Word-aware wrap is intentionally avoided — source code is the input,
//! mid-token breaks are harmless, and the char-based algorithm keeps
//! [`paginate`](crate::text::paginate) and the renderer perfectly
//! aligned. Terminal-width assumptions aside, every visible cell in
//! this module is one column wide.

/// Number of visual rows a source line of `line_len` characters occupies
/// at width `cols`. An empty line (no characters between newlines) still
/// uses one row — the cursor lives somewhere.
///
/// `cols` is clamped to 1 to avoid a divide-by-zero when the caller
/// hasn't yet learned the viewport width.
pub fn visual_rows_for_line(line_len: usize, cols: usize) -> usize {
    let cols = cols.max(1);
    if line_len == 0 {
        1
    } else {
        line_len.div_ceil(cols)
    }
}

/// Lays out `content` into visual rows at width `cols`.
///
/// Each source line is split into `cols`-wide chunks; the terminating
/// `\n` is dropped from the output (rendering doesn't need it, and it
/// would inflate the row count). Empty source lines produce an empty
/// row so blank lines remain visible. `cols` is clamped to 1.
pub fn wrap(content: &[char], cols: usize) -> Vec<Vec<char>> {
    let cols = cols.max(1);
    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut current_line: Vec<char> = Vec::new();

    for &c in content {
        if c == '\n' {
            push_wrapped(&mut rows, &current_line, cols);
            current_line.clear();
        } else {
            current_line.push(c);
        }
    }
    // Trailing partial line with no terminating newline.
    if !current_line.is_empty() {
        push_wrapped(&mut rows, &current_line, cols);
    }

    rows
}

fn push_wrapped(rows: &mut Vec<Vec<char>>, line: &[char], cols: usize) {
    if line.is_empty() {
        rows.push(Vec::new());
        return;
    }
    for chunk in line.chunks(cols) {
        rows.push(chunk.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn empty_line_takes_one_row() {
        assert_eq!(visual_rows_for_line(0, 10), 1);
    }

    #[test]
    fn line_shorter_than_width_takes_one_row() {
        assert_eq!(visual_rows_for_line(5, 10), 1);
    }

    #[test]
    fn line_equal_to_width_takes_one_row() {
        assert_eq!(visual_rows_for_line(10, 10), 1);
    }

    #[test]
    fn line_longer_than_width_wraps() {
        assert_eq!(visual_rows_for_line(11, 10), 2);
        assert_eq!(visual_rows_for_line(25, 10), 3);
    }

    #[test]
    fn cols_zero_is_clamped_to_one() {
        assert_eq!(visual_rows_for_line(3, 0), 3);
    }

    #[test]
    fn wrap_splits_long_line_into_chunks() {
        let rows = wrap(&chars("abcdefgh"), 3);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], chars("abc"));
        assert_eq!(rows[1], chars("def"));
        assert_eq!(rows[2], chars("gh"));
    }

    #[test]
    fn wrap_preserves_blank_lines() {
        let rows = wrap(&chars("a\n\nb"), 5);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], chars("a"));
        assert!(rows[1].is_empty());
        assert_eq!(rows[2], chars("b"));
    }

    #[test]
    fn wrap_drops_newline_terminators() {
        let rows = wrap(&chars("a\nb\n"), 5);
        assert_eq!(rows, vec![chars("a"), chars("b")]);
    }

    #[test]
    fn wrap_row_count_matches_visual_rows_helper() {
        let content = chars("abcdefghij\nxy");
        let rows = wrap(&content, 4);
        let expected = visual_rows_for_line(10, 4) + visual_rows_for_line(2, 4);
        assert_eq!(rows.len(), expected);
    }
}
