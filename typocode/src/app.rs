//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. Subsequent FR
//! branches grow the state (pages, cursor, stats, timer) and the view.

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};

use crate::file::SourceFile;

/// Combined keyboard / tick poll interval. A tick fires whenever
/// `event::poll` returns `false` after this many milliseconds, which
/// later FR branches use to drive the timer widget.
const TICK_RATE: Duration = Duration::from_millis(250);

/// Top-level application state.
///
/// Currently holds the loaded source file and the quit flag. Pages,
/// cursor, stats, timer and phase are added by subsequent FR branches.
#[derive(Debug)]
pub struct App {
    source: SourceFile,
    should_quit: bool,
}

/// Initialises a Ratatui terminal, runs the event loop until the user
/// quits, and restores the terminal. Any error bubbles up to `main`
/// where `color-eyre` formats it.
pub fn run(source: SourceFile) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(source).run_loop(&mut terminal);
    ratatui::restore();
    result
}

impl App {
    /// Builds an `App` ready to be driven by [`run`]. Exposed so
    /// integration tests can construct one against a `TestBackend`.
    pub fn new(source: SourceFile) -> Self {
        Self {
            source,
            should_quit: false,
        }
    }

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
        let text: String = self.source.content.iter().collect();
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
