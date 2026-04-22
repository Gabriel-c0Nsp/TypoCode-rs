//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. Subsequent FR
//! branches grow the state (pages, cursor, stats, timer) and the view.

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::Paragraph,
};

use crate::file::SourceFile;
use crate::text::{Pages, gutter_labels, paginate, wrap_content};
use crate::update::{self, Msg};

/// Combined keyboard / tick poll interval. A tick fires whenever
/// `event::poll` returns `false` after this many milliseconds, which
/// later FR branches use to drive the timer widget.
const TICK_RATE: Duration = Duration::from_millis(250);

/// Player's position within the current page's cells plus any wrong
/// characters typed past the expected position.
///
/// `cu_ptr` is the index of the next cell the player is expected to
/// type. `extras` holds wrong characters typed past `cu_ptr` — the C
/// version's `offset` counter, promoted here into an owned buffer so
/// the renderer can draw them verbatim. Extras must be cleared via
/// backspace before typing can advance again.
#[derive(Debug, Default)]
pub struct Cursor {
    pub cu_ptr: usize,
    pub extras: Vec<char>,
}

impl Cursor {
    pub fn reset(&mut self) {
        self.cu_ptr = 0;
        self.extras.clear();
    }
}

/// Top-level application state.
///
/// Pagination is deferred to the first render so [`Pages`] can be sized
/// against the actual viewport rather than the C version's frozen
/// startup `LINES` value. Stats, timer and phase are added by
/// subsequent FR branches.
#[derive(Debug)]
pub struct App {
    source: SourceFile,
    pages: Option<Pages>,
    cursor: Cursor,
    last_viewport_rows: u16,
    last_viewport_cols: u16,
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
            cursor: Cursor::default(),
            last_viewport_rows: 0,
            last_viewport_cols: 0,
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

    /// Builds or rebuilds [`Pages`] whenever the viewport dimensions
    /// change. Called before each render; the C version froze
    /// pagination at startup and broke on resize — here we recompute so
    /// layout always tracks the live row/column budget. The current
    /// page index resets to 0 because page boundaries shift under the
    /// new budget, and preserving the cursor across a reflow belongs
    /// to later FRs.
    fn ensure_paginated(&mut self, viewport_rows: u16, viewport_cols: u16) {
        if self.pages.is_some()
            && self.last_viewport_rows == viewport_rows
            && self.last_viewport_cols == viewport_cols
        {
            return;
        }
        let pages = paginate(
            &self.source.content,
            viewport_rows as usize,
            viewport_cols as usize,
        );
        self.pages = Pages::new(pages);
        self.cursor.reset();
        self.last_viewport_rows = viewport_rows;
        self.last_viewport_cols = viewport_cols;
    }

    fn handle_event(&mut self, event: Event) {
        let Some(msg) = update::from_key_event(&event) else {
            return;
        };
        self.dispatch(msg);
    }

    fn dispatch(&mut self, msg: Msg) {
        if matches!(msg, Msg::Quit) {
            self.should_quit = true;
            return;
        }
        let Some(pages) = self.pages.as_mut() else {
            return;
        };
        let outcome = update::update(pages, &mut self.cursor, msg);
        if outcome.should_quit {
            self.should_quit = true;
        }
    }

    fn view(&mut self, frame: &mut Frame) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        let gutter_width = gutter_column_width(self.source.line_count);
        let [gutter_area, body_area] =
            Layout::horizontal([Constraint::Length(gutter_width), Constraint::Min(0)])
                .areas(main_area);

        self.ensure_paginated(body_area.height, body_area.width);

        let (gutter_text, body_text, footer_text) = match &self.pages {
            Some(pages) => {
                let page = pages.current();
                let page_chars = page.chars();
                let rows = wrap_content(&page_chars, body_area.width as usize);
                let labels = gutter_labels(&page_chars, body_area.width as usize, page.line_start);
                let digit_width = (gutter_width.saturating_sub(1)) as usize;
                let gutter = labels
                    .iter()
                    .map(|label| match label {
                        Some(n) => format!("{n:>digit_width$} "),
                        None => " ".repeat(digit_width + 1),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let body = rows
                    .iter()
                    .map(|row| row.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n");
                (
                    gutter,
                    body,
                    format!("page {} / {}", pages.current_index(), pages.total()),
                )
            }
            None => (String::new(), String::new(), String::new()),
        };

        frame.render_widget(Paragraph::new(gutter_text), gutter_area);
        frame.render_widget(Paragraph::new(body_text), body_area);
        frame.render_widget(Paragraph::new(footer_text), footer_area);
    }
}

/// Reserves enough columns for the widest source-line number plus a
/// single trailing space separating the gutter from the body.
fn gutter_column_width(total_lines: usize) -> u16 {
    let digits = total_lines.max(1).to_string().len() as u16;
    digits + 1
}
