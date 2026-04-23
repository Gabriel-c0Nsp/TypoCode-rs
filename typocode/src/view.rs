//! Top-level render composer.
//!
//! Lays out the bat-inspired frame: top rule, `File:` header, interior
//! rule, typing body (with gutter), interior rule, status footer,
//! bottom rule. Chrome is drawn first and content widgets paint their
//! own rectangles on top. Pagination is sized against the final
//! typing-body rectangle so the page split tracks what the player
//! actually sees.
//!
//! When the viewport can't fit the full frame we fall back to the plain
//! pre-chrome layout (body + one-line footer) so tiny terminals stay
//! usable.

use std::time::Instant;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::app::{App, Phase};
use crate::timer::format_mm_ss;
use crate::widgets::{body, chrome, footer, gutter, header, summary};

/// Rows consumed by the frame itself: top, header, mid1, mid2, footer,
/// bottom. Body gets whatever is left above this budget.
const CHROME_ROWS: u16 = 6;

/// Draws one frame of the running app.
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let gutter_width = gutter::column_width(app.source.line_count);
    let seam_col = gutter_width;
    // Two extra cols are the seam itself plus a trailing space; we need
    // at least one body column past that or the typing area has no room.
    let chrome_min_cols = seam_col + 3;

    if area.height < CHROME_ROWS + 1 || area.width < chrome_min_cols {
        render_plain(app, frame, area, gutter_width);
        return;
    }

    render_framed(app, frame, area, gutter_width, seam_col);
}

fn render_framed(app: &mut App, frame: &mut Frame, area: Rect, gutter_width: u16, seam_col: u16) {
    let [_top, header_row, _mid1, body_band, _mid2, footer_row, _bot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let mid1_y = 2;
    let mid2_y = area.height - 3;
    chrome::render(frame, area, seam_col, mid1_y, mid2_y);

    let horizontal = [
        Constraint::Length(gutter_width),
        Constraint::Length(2),
        Constraint::Min(0),
    ];
    let [_, _, header_content] = Layout::horizontal(horizontal).areas(header_row);
    let [gutter_area, _sep, body_area] = Layout::horizontal(horizontal).areas(body_band);
    let [_, _, footer_content] = Layout::horizontal(horizontal).areas(footer_row);

    app.ensure_paginated(body_area.height, body_area.width);

    let Some(pages) = app.pages.as_ref() else {
        return;
    };
    let page = pages.current();

    header::render(frame, header_content, &app.source.display_name);
    gutter::render(frame, gutter_area, page, body_area.width, gutter_width);
    let (cursor_col, cursor_row) = body::render(frame, body_area, page, &app.cursor);

    let elapsed = format_mm_ss(app.stopwatch.elapsed(Instant::now()));
    let accuracy = app.stats.accuracy_percent();
    footer::render(
        frame,
        footer_content,
        &elapsed,
        accuracy,
        pages.current_index(),
        pages.total(),
    );

    if app.phase == Phase::Finished {
        summary::render(frame, body_area, &elapsed, accuracy);
        return;
    }

    place_cursor(frame, body_area, cursor_col, cursor_row);
}

/// Viewport-too-small fallback: single-row footer, no frame chrome.
/// Keeps the player able to finish a run on tiny terminals at the cost
/// of the bat-style decoration.
fn render_plain(app: &mut App, frame: &mut Frame, area: Rect, gutter_width: u16) {
    let [main_area, footer_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    let [gutter_area, body_area] =
        Layout::horizontal([Constraint::Length(gutter_width), Constraint::Min(0)])
            .areas(main_area);

    app.ensure_paginated(body_area.height, body_area.width);

    let Some(pages) = app.pages.as_ref() else {
        return;
    };
    let page = pages.current();
    gutter::render(frame, gutter_area, page, body_area.width, gutter_width);
    let (cursor_col, cursor_row) = body::render(frame, body_area, page, &app.cursor);

    let elapsed = format_mm_ss(app.stopwatch.elapsed(Instant::now()));
    let accuracy = app.stats.accuracy_percent();
    footer::render(
        frame,
        footer_area,
        &elapsed,
        accuracy,
        pages.current_index(),
        pages.total(),
    );

    if app.phase == Phase::Finished {
        summary::render(frame, body_area, &elapsed, accuracy);
        return;
    }

    place_cursor(frame, body_area, cursor_col, cursor_row);
}

fn place_cursor(frame: &mut Frame, body_area: Rect, cursor_col: u16, cursor_row: u16) {
    let x = body_area.x.saturating_add(cursor_col);
    let y = body_area.y.saturating_add(cursor_row);
    if x < body_area.x + body_area.width && y < body_area.y + body_area.height {
        frame.set_cursor_position((x, y));
    }
}
