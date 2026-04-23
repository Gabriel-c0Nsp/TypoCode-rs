//! End-of-run summary overlay.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Renders the end-of-run summary centred inside `body_area`: elapsed
/// time, accuracy, and the Tab / Esc hints. Uses [`Clear`] to erase
/// the typed body underneath so the summary is always legible even on
/// dense source files.
pub fn render(frame: &mut Frame, body_area: Rect, elapsed: &str, accuracy: u8) {
    let lines = [
        Line::from(Span::styled(
            "Run complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Time:     {elapsed}")),
        Line::from(format!("Accuracy: {accuracy}%")),
        Line::from(""),
        Line::from(Span::styled(
            "Tab to restart   Esc to quit",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];
    let inner_height = lines.len() as u16;
    let inner_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(0);
    let height = inner_height.saturating_add(2);
    let width = inner_width.saturating_add(4);
    let block = Block::default().borders(Borders::ALL).title(" summary ");
    if body_area.width < width || body_area.height < height {
        frame.render_widget(Clear, body_area);
        frame.render_widget(
            Paragraph::new(Text::from(lines.to_vec()))
                .block(block)
                .alignment(Alignment::Center),
            body_area,
        );
        return;
    }
    let x = body_area.x + (body_area.width - width) / 2;
    let y = body_area.y + (body_area.height - height) / 2;
    let area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines.to_vec()))
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}
