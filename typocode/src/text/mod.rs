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
/// A cell is `Pending` until the typing engine commits it as `Correct`
/// via a strict match. Wrong keystrokes never mutate cells — they pile
/// into the cursor's extras buffer — so there's no `Wrong` variant
/// here. Colour styling lives in the render layer (FR-03); this enum
/// only encodes commit state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Pending,
    Correct,
}

/// One typeable character plus the state of the player's last pass
/// through it. Pagination, wrap and gutter helpers treat a cell's `ch`
/// the same way they treated a raw `char` — layout is independent of
/// state — so [`wrap`] continues to own visual-row structure.
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

    /// Resets every cell on every page to [`CellState::Pending`] and
    /// jumps back to the first page — the data-side of a restart.
    pub fn restart(&mut self) {
        for page in &mut self.pages {
            for cell in &mut page.cells {
                cell.state = CellState::Pending;
            }
        }
        self.current = 0;
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

    /// Total cell count summed across every page. Used by reflow
    /// preservation to translate between local `(page, cu_ptr)` and a
    /// single global character offset into the source text.
    pub fn total_cells(&self) -> usize {
        self.pages.iter().map(|p| p.cells.len()).sum()
    }

    /// Converts a local cursor position `(current, cu_ptr)` into a
    /// global character offset into the source text. Repagination
    /// destroys page boundaries but source content is unchanged, so
    /// this offset is the stable handle across a resize.
    pub fn global_progress(&self, cu_ptr: usize) -> usize {
        let prior: usize = self.pages[..self.current]
            .iter()
            .map(|p| p.cells.len())
            .sum();
        prior + cu_ptr
    }

    /// Restores typed progress after a fresh pagination. Marks the
    /// first `global` cells [`CellState::Correct`], leaves the rest
    /// [`CellState::Pending`], picks the page that contains the
    /// boundary, and returns the local `cu_ptr` within it. `global` is
    /// clamped to `total_cells` so an out-of-range value can't produce
    /// an invalid state.
    ///
    /// When `global` lands exactly at a non-last page's trailing edge
    /// the cursor is placed at offset 0 of the next page, matching the
    /// Enter-driven page advance: the player never observes `cu_ptr ==
    /// page_len` on a non-final page under normal play.
    pub fn restore_progress(&mut self, global: usize) -> usize {
        let num_pages = self.pages.len();
        let total = self.total_cells();
        for page in &mut self.pages {
            for cell in &mut page.cells {
                cell.state = CellState::Pending;
            }
        }
        let clamped = global.min(total);
        let mut remaining = clamped;
        let mut target = 0usize;
        for (i, page) in self.pages.iter_mut().enumerate() {
            let len = page.cells.len();
            if remaining >= len {
                for cell in &mut page.cells {
                    cell.state = CellState::Correct;
                }
                remaining -= len;
                target = i;
                if remaining == 0 {
                    if i + 1 < num_pages {
                        target = i + 1;
                    }
                    break;
                }
            } else {
                for cell in &mut page.cells[..remaining] {
                    cell.state = CellState::Correct;
                }
                target = i;
                break;
            }
        }
        self.current = target;
        let prior: usize = self.pages[..self.current]
            .iter()
            .map(|p| p.cells.len())
            .sum();
        clamped - prior
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

    #[test]
    fn global_progress_sums_prior_pages_plus_cu_ptr() {
        let mut pages = Pages::new(vec![page("abc", 1, 1), page("de", 2, 2)]).unwrap();
        assert_eq!(pages.global_progress(0), 0);
        assert_eq!(pages.global_progress(2), 2);
        pages.next();
        assert_eq!(pages.global_progress(0), 3);
        assert_eq!(pages.global_progress(1), 4);
    }

    #[test]
    fn restore_progress_marks_prefix_correct_and_lands_on_target_page() {
        let mut pages = Pages::new(vec![page("abc", 1, 1), page("def", 2, 2)]).unwrap();
        let cu_ptr = pages.restore_progress(4);
        assert_eq!(pages.current_index(), 2);
        assert_eq!(cu_ptr, 1);
        for cell in &pages.pages[0].cells {
            assert_eq!(cell.state, CellState::Correct);
        }
        assert_eq!(pages.pages[1].cells[0].state, CellState::Correct);
        assert_eq!(pages.pages[1].cells[1].state, CellState::Pending);
    }

    #[test]
    fn restore_progress_zero_stays_on_first_page() {
        let mut pages = Pages::new(vec![page("abc", 1, 1), page("de", 2, 2)]).unwrap();
        pages.next();
        let cu_ptr = pages.restore_progress(0);
        assert_eq!(pages.current_index(), 1);
        assert_eq!(cu_ptr, 0);
        for cell in &pages.pages[0].cells {
            assert_eq!(cell.state, CellState::Pending);
        }
    }

    #[test]
    fn restore_progress_at_page_boundary_advances_to_next_page() {
        let mut pages = Pages::new(vec![page("abc", 1, 1), page("de", 2, 2)]).unwrap();
        let cu_ptr = pages.restore_progress(3);
        assert_eq!(pages.current_index(), 2);
        assert_eq!(cu_ptr, 0);
        for cell in &pages.pages[0].cells {
            assert_eq!(cell.state, CellState::Correct);
        }
        for cell in &pages.pages[1].cells {
            assert_eq!(cell.state, CellState::Pending);
        }
    }

    #[test]
    fn restore_progress_at_end_of_last_page_parks_cursor_past_end() {
        let mut pages = Pages::new(vec![page("abc", 1, 1), page("de", 2, 2)]).unwrap();
        let cu_ptr = pages.restore_progress(5);
        assert_eq!(pages.current_index(), 2);
        assert_eq!(cu_ptr, 2);
        for p in &pages.pages {
            for cell in &p.cells {
                assert_eq!(cell.state, CellState::Correct);
            }
        }
    }

    #[test]
    fn restore_progress_clamps_overflow_to_total() {
        let mut pages = Pages::new(vec![page("ab", 1, 1)]).unwrap();
        let cu_ptr = pages.restore_progress(99);
        assert_eq!(cu_ptr, 2);
        assert_eq!(pages.current_index(), 1);
    }

    #[test]
    fn restore_progress_resets_stale_correct_states_outside_prefix() {
        let mut pages = Pages::new(vec![page("abcd", 1, 1)]).unwrap();
        for cell in &mut pages.pages[0].cells {
            cell.state = CellState::Correct;
        }
        let cu_ptr = pages.restore_progress(2);
        assert_eq!(cu_ptr, 2);
        assert_eq!(pages.pages[0].cells[0].state, CellState::Correct);
        assert_eq!(pages.pages[0].cells[1].state, CellState::Correct);
        assert_eq!(pages.pages[0].cells[2].state, CellState::Pending);
        assert_eq!(pages.pages[0].cells[3].state, CellState::Pending);
    }
}
