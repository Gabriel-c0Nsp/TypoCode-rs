//! Status footer: bat-style "Page: X/Y" indicator, elapsed time,
//! accuracy, and WPM.
//!
//! The footer sits in the row between the bottom interior rule and the
//! bottom border. The `Page:` label is styled to match the C version's
//! `draw_page_number` — blue label, bold values — and leads the line so
//! the page indicator stays in the same screen position the C version
//! anchored it to. Stats follow in time → accuracy → WPM order.

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
    wpm: u32,
    current_page: usize,
    total_pages: usize,
) {
    let line = Line::from(vec![
        Span::styled("Page: ", Style::default().fg(Color::Blue)),
        Span::styled(
            format!("{current_page}/{total_pages}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {elapsed}  {accuracy}%  {wpm} WPM")),
    ]);
    frame.render_widget(Paragraph::new(line), area);
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

    fn render_footer(elapsed: &str, accuracy: u8, wpm: u32) -> String {
        let backend = TestBackend::new(40, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render(frame, area, elapsed, accuracy, wpm, 1, 3);
            })
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    #[test]
    fn footer_order_is_page_time_accuracy_wpm() {
        insta::assert_snapshot!(render_footer("01:23", 95, 78));
    }

    #[test]
    fn footer_at_run_start_reads_zero_wpm() {
        insta::assert_snapshot!(render_footer("00:00", 100, 0));
    }
}
