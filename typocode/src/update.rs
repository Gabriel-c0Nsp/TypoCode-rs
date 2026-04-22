//! Input-to-state dispatch.
//!
//! The run loop translates each crossterm event into a [`Msg`] via
//! [`from_key_event`], then calls [`update`] to mutate the application
//! state. Handlers live in this module — kept free of rendering so
//! unit tests can drive the full typing machine without a terminal.
//!
//! The per-key semantics mirror the C version's `input/input.c`:
//! strict character match, wrong keystrokes stack as extras, backspace
//! is required to recover, and Tab restarts the run.

use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::app::Cursor;
use crate::text::{CellState, Pages};

/// One typing-loop event, normalised across the platform key variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    /// A printable character that isn't space, tab, enter or escape.
    Char(char),
    /// Space bar.
    Space,
    /// Enter / return.
    Enter,
    /// Backspace.
    Backspace,
    /// Tab — restart the current run.
    Tab,
    /// Escape — quit the app.
    Quit,
}

/// Outcome of dispatching a single [`Msg`].
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateOutcome {
    /// Set when the player asked to exit; the run loop should tear down.
    pub should_quit: bool,
}

/// Converts a crossterm event into a [`Msg`]. Returns `None` for events
/// we don't care about (key release / repeat, non-key events, modifier
/// chords). The caller should ignore `None`.
pub fn from_key_event(event: &Event) -> Option<Msg> {
    let Event::Key(key) = event else { return None };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Esc => Some(Msg::Quit),
        KeyCode::Tab => Some(Msg::Tab),
        KeyCode::Backspace => Some(Msg::Backspace),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Char(' ') => Some(Msg::Space),
        KeyCode::Char(c) => Some(Msg::Char(c)),
        _ => None,
    }
}

/// Applies `msg` against the current [`Pages`] and [`Cursor`].
///
/// The handlers mutate the current page's cell states and the cursor
/// in place. Page advance on Enter and the restart-on-Tab behaviour
/// are added by later commits in this FR.
pub fn update(pages: &mut Pages, cursor: &mut Cursor, msg: Msg) -> UpdateOutcome {
    match msg {
        Msg::Quit => UpdateOutcome { should_quit: true },
        Msg::Char(c) => {
            handle_char(pages, cursor, c);
            UpdateOutcome::default()
        }
        Msg::Space | Msg::Enter | Msg::Backspace | Msg::Tab => UpdateOutcome::default(),
    }
}

/// Handles a printable character keystroke. Strict match: the typed
/// char must equal the expected cell's char to advance `cu_ptr`, and
/// any mismatch piles onto `extras` (capped at one entry when the
/// expected cell is a newline, mirroring the C version's offset cap).
fn handle_char(pages: &mut Pages, cursor: &mut Cursor, ch: char) {
    let page = pages.current_mut();
    let Some(expected_cell) = page.cells.get(cursor.cu_ptr) else {
        // Past end of page — waiting for Enter / Tab / Backspace.
        return;
    };
    let expected = expected_cell.ch;

    if expected == '\n' {
        // Typing any non-newline char against an expected newline is
        // wrong; only the first such extra is visible (cap at 1).
        if cursor.extras.is_empty() {
            cursor.extras.push(ch);
        }
        return;
    }

    if ch == expected {
        let state = if cursor.extras.is_empty() {
            CellState::Correct
        } else {
            CellState::Wrong
        };
        page.cells[cursor.cu_ptr].state = state;
        cursor.cu_ptr += 1;
    } else {
        cursor.extras.push(ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Cell, Page, Pages};

    fn make_pages(content: &str) -> Pages {
        let cells: Vec<Cell> = content.chars().map(Cell::pending).collect();
        Pages::new(vec![Page {
            cells,
            line_start: 1,
            line_end: 1,
        }])
        .unwrap()
    }

    #[test]
    fn correct_char_marks_cell_and_advances_cursor() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Char('a'));
        assert_eq!(cursor.cu_ptr, 1);
        assert!(cursor.extras.is_empty());
        assert_eq!(pages.current().cells[0].state, CellState::Correct);
    }

    #[test]
    fn wrong_char_pushes_extra_without_advancing() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Char('x'));
        assert_eq!(cursor.cu_ptr, 0);
        assert_eq!(cursor.extras, vec!['x']);
        assert_eq!(pages.current().cells[0].state, CellState::Pending);
    }

    #[test]
    fn correct_char_with_pending_extras_marks_cell_wrong() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Char('x'));
        update(&mut pages, &mut cursor, Msg::Char('a'));
        assert_eq!(cursor.cu_ptr, 1);
        assert_eq!(cursor.extras, vec!['x']);
        assert_eq!(pages.current().cells[0].state, CellState::Wrong);
    }

    #[test]
    fn wrong_char_against_expected_newline_caps_extras_at_one() {
        let mut pages = make_pages("\nx");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Char('a'));
        update(&mut pages, &mut cursor, Msg::Char('b'));
        update(&mut pages, &mut cursor, Msg::Char('c'));
        assert_eq!(cursor.cu_ptr, 0);
        assert_eq!(cursor.extras, vec!['a']);
    }

    #[test]
    fn char_at_end_of_page_is_ignored() {
        let mut pages = make_pages("a");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Char('a'));
        assert_eq!(cursor.cu_ptr, 1);
        // Past end: further Char input should be a no-op.
        update(&mut pages, &mut cursor, Msg::Char('z'));
        assert_eq!(cursor.cu_ptr, 1);
        assert!(cursor.extras.is_empty());
    }
}
