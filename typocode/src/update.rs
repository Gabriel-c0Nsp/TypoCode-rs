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
        Msg::Space => {
            // Space is an ordinary character as far as strict match is
            // concerned — including the cap-at-one extras rule when the
            // expected cell is a newline, which handle_char already
            // enforces.
            handle_char(pages, cursor, ' ');
            UpdateOutcome::default()
        }
        Msg::Backspace => {
            handle_backspace(pages, cursor);
            UpdateOutcome::default()
        }
        Msg::Enter => {
            handle_enter(pages, cursor);
            UpdateOutcome::default()
        }
        Msg::Tab => {
            handle_restart(pages, cursor);
            UpdateOutcome::default()
        }
    }
}

/// Handles a printable character keystroke. Strict match: the typed
/// char must equal the expected cell's char to advance `cu_ptr`, and
/// any mismatch piles onto `extras` up to the end-of-line cap — a
/// wrong keystroke cannot push the visual cursor past one column
/// beyond the terminating newline of the current source line. This
/// fixes the C version's overflow, where typing errors near the
/// trailing whitespace of a line could run off the right edge.
fn handle_char(pages: &mut Pages, cursor: &mut Cursor, ch: char) {
    let page = pages.current_mut();
    let Some(expected_cell) = page.cells.get(cursor.cu_ptr) else {
        // Past end of page — waiting for Enter / Tab / Backspace.
        return;
    };
    let expected = expected_cell.ch;

    if ch == expected && expected != '\n' {
        let state = if cursor.extras.is_empty() {
            CellState::Correct
        } else {
            CellState::Wrong
        };
        page.cells[cursor.cu_ptr].state = state;
        cursor.cu_ptr += 1;
    } else {
        push_extra(&page.cells, cursor, ch);
    }
}

/// Appends `ch` to `cursor.extras` unless the line-tail cap is already
/// saturated. The cap is `remaining_to_newline + 1` — one extra past
/// the terminating newline, mirroring how a wrong keystroke at an
/// expected `\n` shows a single placeholder glyph past the line end.
fn push_extra(cells: &[crate::text::Cell], cursor: &mut Cursor, ch: char) {
    let limit = remaining_to_eol(cells, cursor.cu_ptr) + 1;
    if cursor.extras.len() < limit {
        cursor.extras.push(ch);
    }
}

/// Number of non-newline characters between `cu_ptr` and the next
/// `\n` (exclusive). Returns 0 when the cursor is already sitting on
/// a `\n` or past the end of the cell list.
fn remaining_to_eol(cells: &[crate::text::Cell], cu_ptr: usize) -> usize {
    let mut count = 0;
    for cell in cells.iter().skip(cu_ptr) {
        if cell.ch == '\n' {
            break;
        }
        count += 1;
    }
    count
}

/// Handles a backspace. Pops the most recent extra first so wrongs
/// are peeled off before already-typed cells revert. Cells are
/// reverted to [`CellState::Pending`] so the player can retype them.
/// If the player is at the start of a fresh line (landing here via
/// the auto-skip after an Enter), a single backspace rewinds past
/// the leading whitespace all the way to the preceding newline cell,
/// matching the C version's cross-line behaviour. At `cu_ptr == 0`
/// on a non-first page, the cursor steps back to the end of the
/// previous page.
fn handle_backspace(pages: &mut Pages, cursor: &mut Cursor) {
    if cursor.extras.pop().is_some() {
        return;
    }

    if cursor.cu_ptr == 0 {
        if pages.current_index() > 1 {
            pages.prev();
            cursor.cu_ptr = pages.current().cells.len();
        }
        return;
    }

    let page = pages.current_mut();
    // Scan backwards through trailing spaces; if we hit a newline before
    // any other non-space char, rewind all the way to that newline so
    // the cursor lands on the terminator of the previous line.
    let mut scan = cursor.cu_ptr;
    loop {
        if scan == 0 {
            break;
        }
        scan -= 1;
        let ch = page.cells[scan].ch;
        if ch == '\n' {
            for idx in scan..cursor.cu_ptr {
                page.cells[idx].state = CellState::Pending;
            }
            cursor.cu_ptr = scan;
            return;
        }
        if ch != ' ' {
            break;
        }
    }

    cursor.cu_ptr -= 1;
    page.cells[cursor.cu_ptr].state = CellState::Pending;
}

/// Handles an Enter keystroke.
///
/// Enter on the last cell of a non-final page advances to the next
/// page and auto-skips its leading whitespace (the C version's
/// end-of-buffer shortcut). On a newline cell with no pending extras
/// it's a correct keystroke: mark the cell Correct, advance, and skip
/// leading whitespace. On any other expected cell it's a wrong
/// keystroke that pushes onto extras — a placeholder `\n` so the
/// renderer can show a "wrong enter" glyph. Enter on a `\n` cell with
/// pending extras is a no-op so the player has to clear the extras
/// first, matching the C fall-through.
fn handle_enter(pages: &mut Pages, cursor: &mut Cursor) {
    let page_len = pages.current().cells.len();

    // Advance page when the cursor is at the last cell of a page that
    // isn't the final page. The C check `vect_buff[cu_ptr+1] == '\0'`
    // fires at cu_ptr == size - 1; we additionally catch cu_ptr == size
    // which can happen after the trailing char on the page was typed.
    if cursor.cu_ptr + 1 >= page_len && !pages.is_last() {
        let extras_empty = cursor.extras.is_empty();
        if cursor.cu_ptr < page_len {
            let state = if extras_empty {
                CellState::Correct
            } else {
                CellState::Wrong
            };
            pages.current_mut().cells[cursor.cu_ptr].state = state;
        }
        pages.next();
        cursor.cu_ptr = 0;
        cursor.extras.clear();
        skip_leading_whitespace(pages, cursor);
        return;
    }

    if cursor.cu_ptr >= page_len {
        return;
    }

    let expected = pages.current().cells[cursor.cu_ptr].ch;
    if expected == '\n' {
        if !cursor.extras.is_empty() {
            // Wait until the player backspaces the pending extras.
            return;
        }
        pages.current_mut().cells[cursor.cu_ptr].state = CellState::Correct;
        cursor.cu_ptr += 1;
        skip_leading_whitespace(pages, cursor);
    } else {
        let cells = &pages.current().cells;
        push_extra(cells, cursor, '\n');
    }
}

/// Advances `cu_ptr` past any consecutive space cells starting at the
/// current position, marking each one [`CellState::Correct`]. Used
/// after a correct Enter to skip the indentation of the next source
/// line so the player doesn't have to type it.
fn skip_leading_whitespace(pages: &mut Pages, cursor: &mut Cursor) {
    let page = pages.current_mut();
    while cursor.cu_ptr < page.cells.len() && page.cells[cursor.cu_ptr].ch == ' ' {
        page.cells[cursor.cu_ptr].state = CellState::Correct;
        cursor.cu_ptr += 1;
    }
}

/// Restarts the current run: every page's cells revert to Pending,
/// the current page goes back to the first, and the cursor resets.
/// Timer restart is wired up separately in FR-06.
fn handle_restart(pages: &mut Pages, cursor: &mut Cursor) {
    pages.restart();
    cursor.reset();
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
    fn extras_cap_at_remaining_to_eol_plus_one() {
        // Expected "hello\n"; cu_ptr at 'h'. Remaining-to-\n = 5, so up
        // to 6 wrong keystrokes (the +1 lands one column past \n) are
        // allowed; the 7th should be rejected.
        let mut pages = make_pages("hello\n");
        let mut cursor = Cursor::default();
        for _ in 0..6 {
            update(&mut pages, &mut cursor, Msg::Char('x'));
        }
        assert_eq!(cursor.extras.len(), 6);
        update(&mut pages, &mut cursor, Msg::Char('x'));
        assert_eq!(cursor.extras.len(), 6);
    }

    #[test]
    fn extras_cap_shrinks_as_cursor_advances_toward_eol() {
        let mut pages = make_pages("abc\n");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 2); // cu_ptr at 'c'
        // Remaining = 1 ('c'); cap = 2.
        for _ in 0..5 {
            update(&mut pages, &mut cursor, Msg::Char('x'));
        }
        assert_eq!(cursor.extras.len(), 2);
    }

    #[test]
    fn wrong_enter_mid_line_respects_cap() {
        // Two Enters at 'a': first pushes '\n' placeholder, second at
        // the cap edge still fits (remaining=2 chars → cap=3).
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        for _ in 0..10 {
            update(&mut pages, &mut cursor, Msg::Enter);
        }
        assert_eq!(cursor.extras.len(), 4);
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

    fn mark_correct(pages: &mut Pages, cursor: &mut Cursor, n: usize) {
        for _ in 0..n {
            let ch = pages.current().cells[cursor.cu_ptr].ch;
            update(pages, cursor, Msg::Char(ch));
        }
    }

    #[test]
    fn backspace_pops_extras_before_reverting_cells() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1);
        update(&mut pages, &mut cursor, Msg::Char('x'));
        update(&mut pages, &mut cursor, Msg::Char('y'));
        assert_eq!(cursor.extras, vec!['x', 'y']);

        update(&mut pages, &mut cursor, Msg::Backspace);
        assert_eq!(cursor.extras, vec!['x']);
        assert_eq!(cursor.cu_ptr, 1);

        update(&mut pages, &mut cursor, Msg::Backspace);
        assert!(cursor.extras.is_empty());
        assert_eq!(cursor.cu_ptr, 1);
    }

    #[test]
    fn backspace_reverts_last_cell_to_pending() {
        let mut pages = make_pages("ab");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 2);
        assert_eq!(pages.current().cells[1].state, CellState::Correct);

        update(&mut pages, &mut cursor, Msg::Backspace);
        assert_eq!(cursor.cu_ptr, 1);
        assert_eq!(pages.current().cells[1].state, CellState::Pending);
    }

    #[test]
    fn backspace_at_origin_of_first_page_is_noop() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Backspace);
        assert_eq!(cursor.cu_ptr, 0);
        assert!(cursor.extras.is_empty());
    }

    #[test]
    fn backspace_rewinds_across_auto_skipped_whitespace_to_newline() {
        // Simulate the state that Enter's auto-skip would leave: "a\n  b"
        // after "a" + Enter + 2 spaces consumed; cursor at 'b' with those
        // cells already marked Correct.
        let mut pages = make_pages("a\n  b");
        let mut cursor = Cursor::default();
        for i in 0..4 {
            pages.current_mut().cells[i].state = CellState::Correct;
        }
        cursor.cu_ptr = 4;

        update(&mut pages, &mut cursor, Msg::Backspace);
        assert_eq!(cursor.cu_ptr, 1);
        for i in 1..4 {
            assert_eq!(pages.current().cells[i].state, CellState::Pending);
        }
        assert_eq!(pages.current().cells[0].state, CellState::Correct);
    }

    fn make_two_pages() -> Pages {
        let page_a = Page {
            cells: "ab".chars().map(Cell::pending).collect(),
            line_start: 1,
            line_end: 1,
        };
        let page_b = Page {
            cells: "cd".chars().map(Cell::pending).collect(),
            line_start: 2,
            line_end: 2,
        };
        let mut pages = Pages::new(vec![page_a, page_b]).unwrap();
        pages.next();
        pages
    }

    #[test]
    fn backspace_at_page_start_rewinds_to_previous_page() {
        let mut pages = make_two_pages();
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Backspace);
        assert_eq!(pages.current_index(), 1);
        assert_eq!(cursor.cu_ptr, pages.current().cells.len());
    }

    #[test]
    fn enter_on_newline_advances_and_skips_leading_spaces() {
        let mut pages = make_pages("a\n  b");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1); // type 'a'
        assert_eq!(cursor.cu_ptr, 1); // at \n

        update(&mut pages, &mut cursor, Msg::Enter);
        // Cursor landed on 'b' (index 4): \n + two spaces auto-skipped.
        assert_eq!(cursor.cu_ptr, 4);
        assert_eq!(pages.current().cells[1].state, CellState::Correct);
        assert_eq!(pages.current().cells[2].state, CellState::Correct);
        assert_eq!(pages.current().cells[3].state, CellState::Correct);
        assert_eq!(pages.current().cells[4].state, CellState::Pending);
    }

    #[test]
    fn enter_mid_line_pushes_extra_without_advancing() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1);
        update(&mut pages, &mut cursor, Msg::Enter);
        assert_eq!(cursor.cu_ptr, 1);
        assert_eq!(cursor.extras, vec!['\n']);
    }

    #[test]
    fn enter_at_newline_with_pending_extras_is_noop() {
        let mut pages = make_pages("a\nb");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1);
        update(&mut pages, &mut cursor, Msg::Char('x'));
        assert_eq!(cursor.extras, vec!['x']);

        update(&mut pages, &mut cursor, Msg::Enter);
        assert_eq!(cursor.cu_ptr, 1);
        assert_eq!(cursor.extras, vec!['x']);
        assert_eq!(pages.current().cells[1].state, CellState::Pending);
    }

    #[test]
    fn enter_at_last_cell_advances_to_next_page() {
        let mut pages = make_two_pages();
        let mut cursor = Cursor::default();
        pages.prev(); // start on page 1
        mark_correct(&mut pages, &mut cursor, 1); // "ab", now at 'b' (last cell)
        assert_eq!(pages.current_index(), 1);
        assert_eq!(cursor.cu_ptr, 1);

        update(&mut pages, &mut cursor, Msg::Enter);
        assert_eq!(pages.current_index(), 2);
        assert_eq!(cursor.cu_ptr, 0);
        assert!(cursor.extras.is_empty());
    }

    #[test]
    fn enter_on_final_page_last_cell_is_noop() {
        let mut pages = make_pages("ab");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1); // at 'b' (last cell of only page)

        update(&mut pages, &mut cursor, Msg::Enter);
        assert_eq!(pages.current_index(), 1);
        assert_eq!(cursor.cu_ptr, 1);
        // At a non-newline cell it's a wrong Enter, so it pushes an extra.
        assert_eq!(cursor.extras, vec!['\n']);
    }

    #[test]
    fn space_on_expected_space_advances_cursor() {
        let mut pages = make_pages("a b");
        let mut cursor = Cursor::default();
        mark_correct(&mut pages, &mut cursor, 1);
        update(&mut pages, &mut cursor, Msg::Space);
        assert_eq!(cursor.cu_ptr, 2);
        assert_eq!(pages.current().cells[1].state, CellState::Correct);
    }

    #[test]
    fn space_against_expected_newline_caps_extras_at_one() {
        let mut pages = make_pages("\n");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Space);
        update(&mut pages, &mut cursor, Msg::Space);
        update(&mut pages, &mut cursor, Msg::Space);
        assert_eq!(cursor.cu_ptr, 0);
        assert_eq!(cursor.extras, vec![' ']);
    }

    #[test]
    fn space_mid_line_wrong_stacks_extras() {
        let mut pages = make_pages("abc");
        let mut cursor = Cursor::default();
        update(&mut pages, &mut cursor, Msg::Space);
        update(&mut pages, &mut cursor, Msg::Space);
        assert_eq!(cursor.cu_ptr, 0);
        assert_eq!(cursor.extras, vec![' ', ' ']);
    }

    #[test]
    fn tab_restarts_cells_cursor_and_page() {
        let mut pages = make_two_pages();
        let mut cursor = Cursor::default();
        // Dirty state: advance to page 2, type a wrong extra.
        mark_correct(&mut pages, &mut cursor, 1);
        update(&mut pages, &mut cursor, Msg::Char('z'));
        assert_eq!(pages.current_index(), 2);
        assert!(!cursor.extras.is_empty());

        update(&mut pages, &mut cursor, Msg::Tab);
        assert_eq!(pages.current_index(), 1);
        assert_eq!(cursor.cu_ptr, 0);
        assert!(cursor.extras.is_empty());
    }
}
