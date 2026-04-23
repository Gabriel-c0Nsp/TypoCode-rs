//! Line-number gutter widget.
//!
//! Produces the right-aligned 1-based source-line labels the wrap helper
//! emits for the current [`Page`]. Labels are `None` on visual-wrap
//! rows so the counter only increments on real source newlines (FR-05).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::text::{Page, gutter_labels};

/// Reserves enough columns for the widest source-line number plus a
/// single trailing space separating the gutter from the body.
pub fn column_width(total_lines: usize) -> u16 {
    let digits = total_lines.max(1).to_string().len() as u16;
    digits + 1
}

/// Renders the gutter text for `page` inside `area`. `body_cols` is the
/// column count of the body region that `page` was wrapped to so the
/// label sequence matches the body's visual rows exactly.
pub fn render(frame: &mut Frame, area: Rect, page: &Page, body_cols: u16, gutter_width: u16) {
    let page_chars = page.chars();
    let labels = gutter_labels(&page_chars, body_cols as usize, page.line_start);
    let digit_width = gutter_width.saturating_sub(1) as usize;
    let style = Style::default().fg(Color::Yellow);
    let lines: Vec<Line<'static>> = labels
        .iter()
        .map(|label| match label {
            Some(n) => Line::from(Span::styled(format!("{n:>digit_width$} "), style)),
            None => Line::from(" ".repeat(digit_width + 1)),
        })
        .collect();
    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}
