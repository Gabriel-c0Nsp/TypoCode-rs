//! Splits expanded source text into [`Page`]s sized to a row budget.
//!
//! Mirrors the C version's `buffer/buffer.c:create_buffer` semantics:
//! each page holds at most `rows_per_page` source lines (newline-terminated
//! runs). A `rows_per_page` of 0 is clamped to 1 so every page makes
//! forward progress; an empty input yields an empty page list.

use super::Page;

/// Splits `content` into pages, each spanning at most `rows_per_page`
/// source lines.
///
/// Source lines are counted by `\n`. Trailing content without a final
/// newline becomes its own (short) final page. The returned pages cover
/// `content` end-to-end without gaps or overlap; concatenating their
/// `content` fields reproduces the input.
pub fn paginate(content: &[char], rows_per_page: usize) -> Vec<Page> {
    let rows = rows_per_page.max(1);
    let mut pages = Vec::new();
    if content.is_empty() {
        return pages;
    }

    let mut page_chars: Vec<char> = Vec::new();
    let mut page_newlines = 0usize;
    let mut line_start = 1usize;
    // Line number the next character belongs to.
    let mut line_cursor = 1usize;

    for &c in content {
        page_chars.push(c);
        if c == '\n' {
            page_newlines += 1;
            let closed_line = line_cursor;
            line_cursor += 1;
            if page_newlines >= rows {
                pages.push(Page {
                    content: std::mem::take(&mut page_chars),
                    line_start,
                    line_end: closed_line,
                });
                line_start = line_cursor;
                page_newlines = 0;
            }
        }
    }

    if !page_chars.is_empty() {
        // Trailing run without a terminal newline, or a page that didn't
        // reach the row budget. `line_cursor` points at the line the next
        // (nonexistent) character would occupy; subtract one when the
        // last char was a newline to avoid counting an empty trailing
        // line.
        let ends_with_newline = page_chars.last() == Some(&'\n');
        let line_end = if ends_with_newline {
            line_cursor - 1
        } else {
            line_cursor
        };
        pages.push(Page {
            content: page_chars,
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

    #[test]
    fn empty_input_yields_no_pages() {
        assert!(paginate(&[], 5).is_empty());
    }

    #[test]
    fn single_line_without_newline_is_one_page() {
        let pages = paginate(&chars("hello"), 3);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].content, chars("hello"));
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 1);
    }

    #[test]
    fn single_line_with_trailing_newline_is_one_page() {
        let pages = paginate(&chars("hello\n"), 3);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].content, chars("hello\n"));
        assert_eq!(pages[0].line_end, 1);
    }

    #[test]
    fn fits_in_budget_as_single_page() {
        let pages = paginate(&chars("a\nb\nc"), 3);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 3);
    }

    #[test]
    fn splits_when_exceeding_budget() {
        let pages = paginate(&chars("a\nb\nc\nd"), 2);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].content, chars("a\nb\n"));
        assert_eq!(pages[0].line_start, 1);
        assert_eq!(pages[0].line_end, 2);
        assert_eq!(pages[1].content, chars("c\nd"));
        assert_eq!(pages[1].line_start, 3);
        assert_eq!(pages[1].line_end, 4);
    }

    #[test]
    fn rows_per_page_zero_is_clamped_to_one() {
        let pages = paginate(&chars("a\nb\n"), 0);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].content, chars("a\n"));
        assert_eq!(pages[1].content, chars("b\n"));
    }

    #[test]
    fn pages_concatenate_to_original_input() {
        let input = chars("line1\nline2\nline3\nline4\nline5");
        let pages = paginate(&input, 2);
        let rejoined: Vec<char> = pages
            .iter()
            .flat_map(|p| p.content.iter().copied())
            .collect();
        assert_eq!(rejoined, input);
    }

    #[test]
    fn tracks_line_ranges_across_multiple_pages() {
        let pages = paginate(&chars("1\n2\n3\n4\n5\n"), 2);
        assert_eq!(pages.len(), 3);
        assert_eq!((pages[0].line_start, pages[0].line_end), (1, 2));
        assert_eq!((pages[1].line_start, pages[1].line_end), (3, 4));
        assert_eq!((pages[2].line_start, pages[2].line_end), (5, 5));
    }
}
