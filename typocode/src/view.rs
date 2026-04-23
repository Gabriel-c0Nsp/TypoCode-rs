//! Top-level render composer.
//!
//! Pulls current state off [`App`], runs [`App::ensure_paginated`] with
//! the concrete body area, and delegates each frame region to the
//! matching widget in [`crate::widgets`]. Render-only logic (layout,
//! cursor placement, phase-dependent overlays) lives here; cell state
//! and extras management stay on the update side.

use std::time::Instant;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::app::{App, Phase};
use crate::timer::format_mm_ss;
use crate::widgets::{body, footer, gutter, summary};

/// Draws one frame of the running app.
pub fn render(app: &mut App, frame: &mut Frame) {
    let [main_area, footer_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

    let gutter_width = gutter::column_width(app.source.line_count);
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

    let x = body_area.x.saturating_add(cursor_col);
    let y = body_area.y.saturating_add(cursor_row);
    if x < body_area.x + body_area.width && y < body_area.y + body_area.height {
        frame.set_cursor_position((x, y));
    }
}
