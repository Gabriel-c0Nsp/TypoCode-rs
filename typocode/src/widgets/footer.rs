//! Status footer: elapsed time, accuracy, and page indicator.

use ratatui::{Frame, layout::Rect, widgets::Paragraph};

/// Renders the one-line status footer into `area`.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    elapsed: &str,
    accuracy: u8,
    current_page: usize,
    total_pages: usize,
) {
    let text = format!("{elapsed}  {accuracy}%  page {current_page} / {total_pages}");
    frame.render_widget(Paragraph::new(text), area);
}
