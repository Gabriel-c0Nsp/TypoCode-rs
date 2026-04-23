//! Typing body: styled cells, extras overlay, cursor position.
//!
//! The body widget renders a [`Page`]'s cells into the area, colouring
//! each character by its [`CellState`] (FR-03: correct = green, pending
//! = terminal default). Wrong keystrokes never mutate cells — they pile
//! into [`Cursor::extras`] and are painted red on top via the extras
//! overlay so the player sees their miss without losing the expected
//! glyph underneath.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::app::Cursor;
use crate::text::{Cell, CellState, Page};

/// Renders `page`'s cells into `area`, returning the `(col, row)` where
/// the cursor should sit relative to `area` so the caller can map it to
/// absolute screen coordinates.
pub fn render(frame: &mut Frame, area: Rect, page: &Page, cursor: &Cursor) -> (u16, u16) {
    let body = styled_body(&page.cells, area.width as usize);
    frame.render_widget(Paragraph::new(body), area);
    draw_extras_overlay(frame, area, page, cursor);
    cursor_screen_pos(&page.cells, cursor.cu_ptr, cursor.extras.len(), area.width)
}

/// Lays out `cells` into styled visual rows at width `cols`, mirroring
/// [`crate::text::wrap_content`] but emitting one [`Span`] per cell so
/// the renderer can colour each character by its [`CellState`]. Correct
/// cells are green, pending cells use the terminal default. `cols` is
/// clamped to 1 to match the wrap helper.
fn styled_body(cells: &[Cell], cols: usize) -> Text<'static> {
    let cols = cols.max(1);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();

    for cell in cells {
        if cell.ch == '\n' {
            lines.push(Line::from(std::mem::take(&mut current)));
            continue;
        }
        if current.len() == cols {
            lines.push(Line::from(std::mem::take(&mut current)));
        }
        current.push(styled_span(cell));
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    Text::from(lines)
}

fn styled_span(cell: &Cell) -> Span<'static> {
    let style = match cell.state {
        CellState::Correct => Style::default().fg(Color::Green),
        CellState::Pending => Style::default(),
    };
    Span::styled(cell.ch.to_string(), style)
}

/// Draws the pending wrong-keystroke buffer on top of the rendered
/// body. Each extra visually sits at the cell index `cu_ptr + i`
/// (char-wrapped like the body). Space and newline inputs render as
/// `_` so the player can see that a special key was pressed in the
/// wrong place — matching the C version's underscore glyph. Extras
/// are painted red (FR-03) so wrong keystrokes stand out.
fn draw_extras_overlay(frame: &mut Frame, body_area: Rect, page: &Page, cursor: &Cursor) {
    if cursor.extras.is_empty() || body_area.width == 0 {
        return;
    }
    let style = Style::default().fg(Color::Red);
    let buf = frame.buffer_mut();
    for (i, &ex) in cursor.extras.iter().enumerate() {
        let (col, row) = cursor_screen_pos(&page.cells, cursor.cu_ptr, i, body_area.width);
        let x = body_area.x.saturating_add(col);
        let y = body_area.y.saturating_add(row);
        if x >= body_area.x + body_area.width || y >= body_area.y + body_area.height {
            continue;
        }
        let display = match ex {
            ' ' | '\n' => '_',
            c => c,
        };
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(display);
            cell.set_style(style);
        }
    }
}

/// Maps `(cu_ptr, extras_len)` to the `(col, row)` the cursor should
/// occupy within the body area, applying char-wrap at `body_cols`.
///
/// Cells before `cu_ptr` already contributed to the visual layout —
/// each non-newline advances the column by one (wrapping at
/// `body_cols`), each newline resets to column 0 on the next row.
/// Extras are rendered to the right of `cu_ptr` so they bump the
/// column further, wrapping the same way. Returned `(0, 0)` when
/// `body_cols` is zero.
pub fn cursor_screen_pos(
    cells: &[Cell],
    cu_ptr: usize,
    extras_len: usize,
    body_cols: u16,
) -> (u16, u16) {
    if body_cols == 0 {
        return (0, 0);
    }
    let cols = body_cols as usize;
    let mut row: usize = 0;
    let mut col: usize = 0;
    let end = cu_ptr.min(cells.len());
    for cell in &cells[..end] {
        if cell.ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
            if col >= cols {
                col = 0;
                row += 1;
            }
        }
    }
    col += extras_len;
    if col >= cols {
        row += col / cols;
        col %= cols;
    }
    (col as u16, row as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<Cell> {
        s.chars().map(Cell::pending).collect()
    }

    #[test]
    fn cursor_at_origin_with_no_extras() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 0, 0, 10), (0, 0));
    }

    #[test]
    fn cursor_advances_within_a_line() {
        let cs = cells("abcdef");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 10), (3, 0));
    }

    #[test]
    fn newline_drops_cursor_to_next_row() {
        let cs = cells("ab\ncd");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 10), (0, 1));
        assert_eq!(cursor_screen_pos(&cs, 5, 0, 10), (2, 1));
    }

    #[test]
    fn char_wrap_bumps_row() {
        let cs = cells("abcdef");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 3), (0, 1));
        assert_eq!(cursor_screen_pos(&cs, 6, 0, 3), (0, 2));
    }

    #[test]
    fn extras_bump_column_and_may_wrap() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 1, 2, 10), (3, 0));
        assert_eq!(cursor_screen_pos(&cs, 2, 4, 4), (2, 1));
    }

    #[test]
    fn zero_body_cols_short_circuits_to_origin() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 2, 0, 0), (0, 0));
    }
}
