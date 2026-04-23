//! Bat-style "File: <name>" header.
//!
//! Mirrors the C version's `draw_file_name` — blue label, bold filename
//! — rendered into the single-row strip between the top border and the
//! first interior rule.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders `File: <display_name>` into `area`.
pub fn render(frame: &mut Frame, area: Rect, display_name: &str) {
    let line = Line::from(vec![
        Span::styled("File: ", Style::default().fg(Color::Blue)),
        Span::styled(
            display_name.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
