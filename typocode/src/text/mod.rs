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

/// Per-character state of a [`Cell`].
///
/// Every cell starts `Pending` and transitions to `Correct` or `Wrong`
/// when the typing engine advances past it. `Wrong` is latched — the
/// player must backspace to clear a mistake, which resets the cell to
/// `Pending` again. Colour styling lives in the render layer (FR-03);
/// the state enum only encodes what happened, not how it looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Pending,
    Correct,
    Wrong,
}

/// One typeable character plus the state of the player's last pass
/// through it. Pagination, wrap and gutter helpers treat a cell's `ch`
/// the same way they treated a raw `char` — layout is independent of
/// state — so [`wrap`](crate::text::wrap) continues to own visual-row
/// structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub state: CellState,
}

impl Cell {
    /// Constructs a fresh cell for `ch` that hasn't been typed yet.
    pub fn pending(ch: char) -> Self {
        Self {
            ch,
            state: CellState::Pending,
        }
    }
}

/// A single page of expanded source text.
///
/// `cells` holds the expected characters (tabs already expanded by the
/// file loader) paired with their current typing state. `line_start`
/// and `line_end` are the inclusive 1-based source-line range this
/// page spans — used by the line-number gutter (FR-05) so visual wraps
/// don't disturb the counter.
#[derive(Debug, Clone)]
pub struct Page {
    pub cells: Vec<Cell>,
    pub line_start: usize,
    pub line_end: usize,
}

impl Page {
    /// Collects the page's expected characters into a fresh `Vec<char>`
    /// for layout helpers that don't care about state.
    pub fn chars(&self) -> Vec<char> {
        self.cells.iter().map(|c| c.ch).collect()
    }
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

    /// Mutable access to the current page — needed by the update layer
    /// to mark cell states as the player types through them.
    pub fn current_mut(&mut self) -> &mut Page {
        &mut self.pages[self.current]
    }

    /// Whether the current page is the last one.
    pub fn is_last(&self) -> bool {
        self.current + 1 == self.pages.len()
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
            cells: content.chars().map(Cell::pending).collect(),
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
