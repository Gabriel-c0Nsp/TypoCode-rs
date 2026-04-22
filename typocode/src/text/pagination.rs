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
