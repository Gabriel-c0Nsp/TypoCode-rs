//! Splits expanded source text into [`Page`]s sized to a visual row
//! budget.
//!
//! A source line that's wider than the viewport wraps to multiple
//! visual rows (see [`super::wrap`]); pagination packs source lines
//! into a page until adding the next one would exceed
//! `rows_per_page` visual rows. A source line that's taller than the
//! whole budget on its own lands on a page by itself and is allowed
//! to overflow — we never split a source line across pages, so the
//! line-number gutter stays coherent.
//!
//! `rows_per_page` and `cols` are each clamped to 1 so pagination
//! always makes forward progress; empty input yields an empty page
//! list.

use super::wrap::visual_rows_for_line;
use super::{Cell, Page};

/// Splits `content` into pages.
///
/// `rows_per_page` is the page's visual-row budget (body area height);
/// `cols` is the body area width used to compute per-source-line wrap.
/// The returned pages cover `content` end-to-end without gaps or
/// overlap: concatenating their cells' chars reproduces the input.
pub fn paginate(content: &[char], rows_per_page: usize, cols: usize) -> Vec<Page> {
    let rows = rows_per_page.max(1);
    let cols = cols.max(1);
    let mut pages = Vec::new();
    if content.is_empty() {
        return pages;
    }

    let mut page_cells: Vec<Cell> = Vec::new();
    let mut page_rows = 0usize;
    let mut line_start = 1usize;
    // Line number the next character belongs to.
    let mut line_cursor = 1usize;
    let mut current_line: Vec<Cell> = Vec::new();

    for &c in content {
        if c == '\n' {
            let line_rows = visual_rows_for_line(current_line.len(), cols);
            if !page_cells.is_empty() && page_rows + line_rows > rows {
                pages.push(Page {
                    cells: std::mem::take(&mut page_cells),
                    line_start,
                    line_end: line_cursor - 1,
                });
                line_start = line_cursor;
                page_rows = 0;
            }
            page_cells.append(&mut current_line);
            page_cells.push(Cell::pending('\n'));
            page_rows += line_rows;
            line_cursor += 1;
        } else {
            current_line.push(Cell::pending(c));
        }
    }

    // Trailing partial line with no terminating newline.
    if !current_line.is_empty() {
        let line_rows = visual_rows_for_line(current_line.len(), cols);
        if !page_cells.is_empty() && page_rows + line_rows > rows {
            pages.push(Page {
                cells: std::mem::take(&mut page_cells),
                line_start,
                line_end: line_cursor - 1,
            });
            line_start = line_cursor;
        }
        page_cells.append(&mut current_line);
    }

    if !page_cells.is_empty() {
        let line_end = if page_cells.last().map(|c| c.ch) == Some('\n') {
            line_cursor - 1
        } else {
            line_cursor
        };
        pages.push(Page {
            cells: page_cells,
            line_start,
            line_end,
        });
    }

    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    fn page_chars(page: &Page) -> Vec<char> {
        page.cells.iter().map(|c| c.ch).collect()
    }

    #[test]
    fn empty_input_yields_no_pages() {
        assert!(paginate(&[], 5, 80).is_empty());
    }

    #[test]
    fn single_line_without_newline_is_one_page() {
        let pages = paginate(&chars("hello"), 3, 80);
        assert_eq!(pages.len(), 1);
        assert_eq!(page_chars(&pages[0]), chars("hello"));
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 1);
    }

    #[test]
    fn single_line_with_trailing_newline_is_one_page() {
        let pages = paginate(&chars("hello\n"), 3, 80);
        assert_eq!(pages.len(), 1);
        assert_eq!(page_chars(&pages[0]), chars("hello\n"));
        assert_eq!(pages[0].line_end, 1);
    }

    #[test]
    fn fits_in_budget_as_single_page() {
        let pages = paginate(&chars("a\nb\nc"), 3, 80);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 3);
    }

    #[test]
    fn splits_when_exceeding_budget() {
        let pages = paginate(&chars("a\nb\nc\nd"), 2, 80);
        assert_eq!(pages.len(), 2);
        assert_eq!(page_chars(&pages[0]), chars("a\nb\n"));
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 2);
        assert_eq!(page_chars(&pages[1]), chars("c\nd"));
        assert_eq!(pages[1].line_start, 3);
        assert_eq!(pages[1].line_end, 4);
    }

    #[test]
    fn rows_per_page_zero_is_clamped_to_one() {
        let pages = paginate(&chars("a\nb\n"), 0, 80);
        assert_eq!(pages.len(), 2);
        assert_eq!(page_chars(&pages[0]), chars("a\n"));
        assert_eq!(page_chars(&pages[1]), chars("b\n"));
    }

    #[test]
    fn pages_concatenate_to_original_input() {
        let input = chars("line1\nline2\nline3\nline4\nline5");
        let pages = paginate(&input, 2, 80);
        let rejoined: Vec<char> = pages
            .iter()
            .flat_map(|p| p.cells.iter().map(|c| c.ch))
            .collect();
        assert_eq!(rejoined, input);
    }

    #[test]
    fn tracks_line_ranges_across_multiple_pages() {
        let pages = paginate(&chars("1\n2\n3\n4\n5\n"), 2, 80);
        assert_eq!(pages.len(), 3);
        assert_eq!((pages[0].line_start, pages[0].line_end), (1, 2));
        assert_eq!((pages[1].line_start, pages[1].line_end), (3, 4));
        assert_eq!((pages[2].line_start, pages[2].line_end), (5, 5));
    }

    #[test]
    fn wrapped_line_consumes_multiple_visual_rows() {
        let pages = paginate(&chars("abcdef\nx"), 2, 3);
        assert_eq!(pages.len(), 2);
        assert_eq!(page_chars(&pages[0]), chars("abcdef\n"));
        assert_eq!((pages[0].line_start, pages[0].line_end), (1, 1));
        assert_eq!(page_chars(&pages[1]), chars("x"));
        assert_eq!((pages[1].line_start, pages[1].line_end), (2, 2));
    }

    #[test]
    fn line_longer_than_budget_gets_its_own_overflowing_page() {
        let pages = paginate(&chars("abcdefghijkl\nfoo"), 2, 3);
        assert_eq!(pages.len(), 2);
        assert_eq!(page_chars(&pages[0]), chars("abcdefghijkl\n"));
        assert_eq!((pages[0].line_start, pages[0].line_end), (1, 1));
        assert_eq!(page_chars(&pages[1]), chars("foo"));
    }

    #[test]
    fn blank_line_still_occupies_one_row() {
        let pages = paginate(&chars("a\n\nb\nc"), 2, 80);
        assert_eq!(pages.len(), 2);
        assert_eq!(page_chars(&pages[0]), chars("a\n\n"));
        assert_eq!((pages[0].line_start, pages[0].line_end), (1, 2));
        assert_eq!(page_chars(&pages[1]), chars("b\nc"));
        assert_eq!((pages[1].line_start, pages[1].line_end), (3, 4));
    }

    #[test]
    fn cells_start_in_pending_state() {
        let pages = paginate(&chars("hi"), 3, 80);
        for cell in &pages[0].cells {
            assert_eq!(cell.state, super::super::CellState::Pending);
        }
    }
}
