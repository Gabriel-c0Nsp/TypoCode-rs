//! Status footer: elapsed time, accuracy, and bat-style "Page: X/Y"
//! indicator.
//!
//! The footer sits in the row between the bottom interior rule and the
//! bottom border. The `Page:` label is styled to match the C version's
//! `draw_page_number` — blue label, bold values — while the stats block
//! (elapsed + accuracy) uses the terminal default so it doesn't compete
//! with the indicator for attention.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders the one-line status footer into `area`.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    elapsed: &str,
    accuracy: u8,
    current_page: usize,
    total_pages: usize,
) {
    let line = Line::from(vec![
        Span::raw(format!("{elapsed}  {accuracy}%  ")),
        Span::styled("Page: ", Style::default().fg(Color::Blue)),
        Span::styled(
            format!("{current_page}/{total_pages}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
