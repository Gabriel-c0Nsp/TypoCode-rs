//! Pure data model for the typing text.
//!
//! Source text is split into [`Page`]s that each fit within a given row
//! budget. [`Pages`] owns the collection plus the index of the page the
//! player is currently typing through. Render and input layers consume
//! these types without touching the filesystem or the terminal.

pub mod pagination;
pub mod wrap;

pub use pagination::paginate;
pub use wrap::{gutter_labels, visual_rows_for_line, wrap as wrap_content};

/// A single page of expanded source text.
///
/// `content` holds the characters to render (tabs already expanded by
/// the file loader). `line_start` and `line_end` are the inclusive
/// 1-based source-line range this page spans — used by the line-number
/// gutter (FR-05) so visual wraps don't disturb the counter.
#[derive(Debug, Clone)]
pub struct Page {
    pub content: Vec<char>,
    pub line_start: usize,
    pub line_end: usize,
}

/// Ordered collection of [`Page`]s plus the currently displayed index.
///
/// The index is held inside the type so `update`-layer navigation can
/// mutate it without exposing the underlying `Vec`. `current` is always
/// a valid index as long as `pages` is non-empty.
#[derive(Debug, Clone)]
pub struct Pages {
    pages: Vec<Page>,
    current: usize,
}

impl Pages {
    /// Wraps a non-empty page list. Returns `None` when `pages` is empty
    /// so callers don't accidentally create an un-navigable state.
    pub fn new(pages: Vec<Page>) -> Option<Self> {
        if pages.is_empty() {
            None
        } else {
            Some(Self { pages, current: 0 })
        }
    }

    /// Current page reference. Safe to call because `new` rejects empty
    /// input and `next`/`prev` are bounded.
    pub fn current(&self) -> &Page {
        &self.pages[self.current]
    }

    /// 1-based index of the current page, for display in the footer.
    pub fn current_index(&self) -> usize {
        self.current + 1
    }

    /// Total page count.
    pub fn total(&self) -> usize {
        self.pages.len()
    }

    /// Advance to the next page, saturating at the last index.
    pub fn next(&mut self) {
        if self.current + 1 < self.pages.len() {
            self.current += 1;
        }
    }

    /// Step back one page, saturating at 0.
    pub fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(content: &str, line_start: usize, line_end: usize) -> Page {
        Page {
            content: content.chars().collect(),
            line_start,
            line_end,
        }
    }

    #[test]
    fn new_rejects_empty_pages() {
        assert!(Pages::new(Vec::new()).is_none());
    }

    #[test]
    fn next_and_prev_saturate_at_bounds() {
        let mut pages = Pages::new(vec![page("a", 1, 1), page("b", 2, 2)]).unwrap();
        assert_eq!(pages.current_index(), 1);
        pages.prev();
        assert_eq!(pages.current_index(), 1);
        pages.next();
        assert_eq!(pages.current_index(), 2);
        pages.next();
        assert_eq!(pages.current_index(), 2);
        pages.prev();
        assert_eq!(pages.current_index(), 1);
    }

    #[test]
    fn total_reflects_page_count() {
        let pages = Pages::new(vec![page("a", 1, 1), page("b", 2, 2), page("c", 3, 3)]).unwrap();
        assert_eq!(pages.total(), 3);
    }
}
