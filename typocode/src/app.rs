//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. The skeleton
//! renders a placeholder screen and exits on `Esc`; real game state lands
//! as the per-FR branches merge in.

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Combined keyboard / tick poll interval. A tick fires whenever
/// `event::poll` returns `false` after this many milliseconds, which
/// later FR branches use to drive the timer widget.
const TICK_RATE: Duration = Duration::from_millis(250);

/// Top-level application state.
///
/// The skeleton tracks only the quit flag. Pages, cursor, stats, timer
/// and phase are added by subsequent FR branches.
#[derive(Debug, Default)]
pub struct App {
    should_quit: bool,
}

/// Initialises a Ratatui terminal, runs the event loop until the user
/// quits, and restores the terminal. Any error bubbles up to `main`
/// where `color-eyre` formats it.
pub fn run() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run_loop(&mut terminal);
    ratatui::restore();
    result
}

impl App {
    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.view(frame))?;

            if event::poll(TICK_RATE)? {
                self.handle_event(event::read()?);
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Esc
        {
            self.should_quit = true;
        }
    }

    fn view(&self, frame: &mut Frame) {
        let lines = vec![
            Line::from(Span::styled(
                "TypoCode",
                Style::new().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Skeleton run loop — press Esc to quit."),
        ];
        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, frame.area());
    }
}
