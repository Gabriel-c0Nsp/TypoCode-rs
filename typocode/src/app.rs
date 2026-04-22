//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. Subsequent FR
//! branches grow the state (pages, cursor, stats, timer) and the view.

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};

use crate::file::SourceFile;
use crate::text::{Pages, paginate};

/// Combined keyboard / tick poll interval. A tick fires whenever
/// `event::poll` returns `false` after this many milliseconds, which
/// later FR branches use to drive the timer widget.
const TICK_RATE: Duration = Duration::from_millis(250);

/// Top-level application state.
///
/// Pagination is deferred to the first render so [`Pages`] can be sized
/// against the actual viewport rather than the C version's frozen
/// startup `LINES` value. Cursor, stats, timer and phase are added by
/// subsequent FR branches.
#[derive(Debug)]
pub struct App {
    source: SourceFile,
    pages: Option<Pages>,
    last_viewport_rows: u16,
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
            pages: None,
            last_viewport_rows: 0,
            should_quit: false,
        }
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| {
                self.ensure_paginated(frame.area().height);
                self.view(frame);
            })?;

            if event::poll(TICK_RATE)? {
                self.handle_event(event::read()?);
            }
        }
        Ok(())
    }

    /// Builds or rebuilds [`Pages`] whenever the viewport height
    /// changes. Called before each render; the C version froze
    /// pagination at startup and broke on resize — here we recompute so
    /// layout always tracks the live row budget. The current page index
    /// resets to 0 because page boundaries shift with the new budget,
    /// and preserving the cursor across a reflow belongs to later FRs.
    fn ensure_paginated(&mut self, viewport_rows: u16) {
        if self.pages.is_some() && self.last_viewport_rows == viewport_rows {
            return;
        }
        let pages = paginate(&self.source.content, viewport_rows as usize);
        self.pages = Pages::new(pages);
        self.last_viewport_rows = viewport_rows;
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Esc => self.should_quit = true,
                KeyCode::PageDown => {
                    if let Some(pages) = self.pages.as_mut() {
                        pages.next();
                    }
                }
                KeyCode::PageUp => {
                    if let Some(pages) = self.pages.as_mut() {
                        pages.prev();
                    }
                }
                _ => {}
            }
        }
    }

    fn view(&self, frame: &mut Frame) {
        let text: String = match &self.pages {
            Some(pages) => pages.current().content.iter().collect(),
            None => String::new(),
        };
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
