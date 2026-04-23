//! Bat-inspired frame chrome.
//!
//! Draws the top/middle/bottom horizontal rules and the vertical seam
//! between the gutter and the typing body, using Unicode box-drawing
//! characters (`─ │ ┬ ┼ ┴`). The C version does the same in `tui.c` —
//! see `draw_display_panel` — and we intentionally match both the
//! glyphs and the junction placement so the port looks familiar.
//!
//! Chrome is rendered first; content widgets paint on top of their own
//! rectangles afterwards, leaving the frame intact around them.

use ratatui::{Frame, layout::Rect};

/// Draws the frame into `area`.
///
/// `seam_col` is the column — relative to `area` — where the vertical
/// `│` separator sits (and where the `┬ ┼ ┴` junctions appear on the
/// horizontal rules). `mid1_y` and `mid2_y` are the row offsets — also
/// relative to `area` — of the two interior horizontal rules that
/// fence the header and footer bands. The caller is responsible for
/// having reserved rows 0 and `area.height - 1` for the top and
/// bottom rules.
pub fn render(frame: &mut Frame, area: Rect, seam_col: u16, mid1_y: u16, mid2_y: u16) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let buf = frame.buffer_mut();
    let top = 0u16;
    let bot = area.height - 1;
    for dy in 0..area.height {
        let y = area.y + dy;
        let is_top = dy == top;
        let is_bot = dy == bot;
        let is_mid = dy == mid1_y || dy == mid2_y;
        for dx in 0..area.width {
            let x = area.x + dx;
            let is_seam = dx == seam_col;
            let ch = match (is_top, is_bot, is_mid, is_seam) {
                (true, _, _, true) => '┬',
                (_, true, _, true) => '┴',
                (_, _, true, true) => '┼',
                (true, _, _, false) | (_, true, _, false) | (_, _, true, false) => '─',
                (_, _, _, true) => '│',
                _ => continue,
            };
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(ch);
            }
        }
    }
}
