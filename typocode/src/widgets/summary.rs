//! End-of-run summary overlay.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Renders the end-of-run summary centred inside `body_area`: accuracy,
/// elapsed time, WPM, and the Tab / Esc hints. Row order mirrors the
/// live footer (accuracy → time → wpm). Uses [`Clear`] to erase the
/// typed body underneath so the summary is always legible even on dense
/// source files.
pub fn render(frame: &mut Frame, body_area: Rect, elapsed: &str, accuracy: u8, wpm: u32) {
    let accuracy_value = format!("{accuracy}%");
    let wpm_value = format!("{wpm}");
    let label_w = "Accuracy:".len();
    let value_w = elapsed.len().max(accuracy_value.len()).max(wpm_value.len());
    let lines = [
        Line::from(Span::styled(
            "Run complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "{:<label_w$}  {:>value_w$}",
            "Accuracy:", accuracy_value
        )),
        Line::from(format!("{:<label_w$}  {:>value_w$}", "Time:", elapsed)),
        Line::from(format!("{:<label_w$}  {:>value_w$}", "WPM:", wpm_value)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    fn buffer_to_string(buf: &Buffer) -> String {
        let area = buf.area();
        let mut out = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn render_summary(elapsed: &str, accuracy: u8, wpm: u32) -> String {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render(frame, area, elapsed, accuracy, wpm);
            })
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    #[test]
    fn summary_at_100_percent() {
        insta::assert_snapshot!(render_summary("00:12", 100, 65));
    }

    #[test]
    fn summary_at_80_percent() {
        insta::assert_snapshot!(render_summary("00:12", 80, 52));
    }

    #[test]
    fn summary_at_single_digit_accuracy() {
        insta::assert_snapshot!(render_summary("00:12", 7, 4));
    }

    #[test]
    fn summary_at_zero_accuracy() {
        insta::assert_snapshot!(render_summary("00:12", 0, 0));
    }

    #[test]
    fn summary_shows_three_digit_wpm() {
        insta::assert_snapshot!(render_summary("00:12", 100, 142));
    }
}
