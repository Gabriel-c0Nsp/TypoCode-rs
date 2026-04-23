//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. Subsequent FR
//! branches grow the state (pages, cursor, stats, timer) and the view.

use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

use crate::file::SourceFile;
use crate::stats::Stats;
use crate::text::{Pages, paginate};
use crate::timer::Stopwatch;
use crate::update::{self, Msg};
use crate::view;

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
    /// Clears the cursor position and the extras buffer — used when
    /// repagination invalidates page offsets and when Tab restarts
    /// the run.
    pub fn reset(&mut self) {
        self.cu_ptr = 0;
        self.extras.clear();
    }
}

/// Lifecycle of the current typing run.
///
/// The app starts in [`Phase::Playing`] and transitions to
/// [`Phase::Finished`] the moment the player reaches the end of the
/// final page with no pending extras (FR-08). Tab in either phase
/// restarts back to [`Phase::Playing`]; Esc quits from either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Finished,
}

/// Top-level application state.
///
/// Pagination is deferred to the first render so [`Pages`] can be sized
/// against the actual viewport rather than the C version's frozen
/// startup `LINES` value. Stats, timer and phase are added by
/// subsequent FR branches.
#[derive(Debug)]
pub struct App {
    pub(crate) source: SourceFile,
    pub(crate) pages: Option<Pages>,
    pub(crate) cursor: Cursor,
    pub(crate) stopwatch: Stopwatch,
    pub(crate) stats: Stats,
    pub(crate) phase: Phase,
    pub(crate) last_viewport_rows: u16,
    pub(crate) last_viewport_cols: u16,
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
            stopwatch: Stopwatch::new(),
            stats: Stats::new(),
            phase: Phase::Playing,
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
    /// pagination at startup and broke on resize. Here we recompute
    /// against the live row/column budget and preserve typed progress
    /// across the reflow: the global char offset derived from the old
    /// `(page, cu_ptr)` is replayed into the new pagination via
    /// [`Pages::restore_progress`], so cell colours (FR-03) and the
    /// cursor survive a terminal resize. Pending extras are preserved
    /// verbatim: wrong keystrokes that haven't been cleared stay on
    /// screen at the new cursor position.
    pub(crate) fn ensure_paginated(&mut self, viewport_rows: u16, viewport_cols: u16) {
        if self.pages.is_some()
            && self.last_viewport_rows == viewport_rows
            && self.last_viewport_cols == viewport_cols
        {
            return;
        }
        let global_progress = self
            .pages
            .as_ref()
            .map(|pages| pages.global_progress(self.cursor.cu_ptr));
        let pages = paginate(
            &self.source.content,
            viewport_rows as usize,
            viewport_cols as usize,
        );
        self.pages = Pages::new(pages);
        match (self.pages.as_mut(), global_progress) {
            (Some(pages), Some(global)) => {
                self.cursor.cu_ptr = pages.restore_progress(global);
            }
            _ => self.cursor.reset(),
        }
        self.last_viewport_rows = viewport_rows;
        self.last_viewport_cols = viewport_cols;
    }

    fn handle_event(&mut self, event: Event) {
        let Some(msg) = update::from_key_event(&event) else {
            return;
        };
        self.dispatch(msg);
    }

    pub(crate) fn dispatch(&mut self, msg: Msg) {
        if matches!(msg, Msg::Quit) {
            self.should_quit = true;
            return;
        }
        // On the finish screen only Tab (restart) is meaningful; every
        // other key is swallowed so stray typing can't disturb the
        // frozen summary.
        if self.phase == Phase::Finished && !matches!(msg, Msg::Tab) {
            return;
        }
        let Some(pages) = self.pages.as_mut() else {
            return;
        };
        let outcome = update::update(pages, &mut self.cursor, msg);
        // Timer starts on the first typing keystroke of the run and
        // resets on Tab (restart), matching the C version's
        // `started_test` flag. Quit is already handled above.
        match msg {
            Msg::Tab => {
                self.stopwatch.reset();
                self.stats.reset();
                self.phase = Phase::Playing;
            }
            _ => self.stopwatch.start(Instant::now()),
        }
        if let Some(kind) = outcome.keystroke {
            self.stats.record(kind);
        }
        if outcome.should_quit {
            self.should_quit = true;
        }
        if self.phase == Phase::Playing && self.is_run_complete() {
            self.phase = Phase::Finished;
            self.stopwatch.stop(Instant::now());
        }
    }

    /// The run is complete when the player sits past the last cell of
    /// the final page with no pending extras. Enter on the penultimate
    /// cell advances `cu_ptr` to `cells.len()` (extras cleared by the
    /// handler), which is the expected landing spot; typing the last
    /// non-newline char does the same via `handle_char`.
    fn is_run_complete(&self) -> bool {
        let Some(pages) = &self.pages else {
            return false;
        };
        pages.is_last()
            && self.cursor.cu_ptr >= pages.current().cells.len()
            && self.cursor.extras.is_empty()
    }

    fn view(&mut self, frame: &mut Frame) {
        view::render(self, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::CellState;

    fn app_with_source(content: &str) -> App {
        let chars: Vec<char> = content.chars().collect();
        let line_count = content.lines().count().max(1);
        let source = SourceFile {
            display_name: "test".to_string(),
            content: chars,
            line_count,
        };
        App::new(source)
    }

    #[test]
    fn reflow_preserves_cursor_progress_and_cell_states() {
        let mut app = app_with_source("line1\nline2\nline3\nline4\n");
        app.ensure_paginated(10, 80);

        for ch in ['l', 'i', 'n', 'e', '1'] {
            app.dispatch(Msg::Char(ch));
        }
        app.dispatch(Msg::Enter);
        app.dispatch(Msg::Char('l'));
        app.dispatch(Msg::Char('i'));
        let before_global = app
            .pages
            .as_ref()
            .unwrap()
            .global_progress(app.cursor.cu_ptr);
        assert!(before_global >= 8);

        app.dispatch(Msg::Char('X'));
        assert_eq!(app.cursor.extras, vec!['X']);

        app.ensure_paginated(2, 80);

        let after_global = app
            .pages
            .as_ref()
            .unwrap()
            .global_progress(app.cursor.cu_ptr);
        assert_eq!(after_global, before_global);
        assert_eq!(app.cursor.extras, vec!['X']);

        let pages = app.pages.as_ref().unwrap();
        let mut seen = 0usize;
        for page in pages.iter() {
            for cell in &page.cells {
                let expected = if seen < after_global {
                    CellState::Correct
                } else {
                    CellState::Pending
                };
                assert_eq!(cell.state, expected, "cell {seen}");
                seen += 1;
            }
        }
    }

    #[test]
    fn reflow_on_same_dimensions_is_noop() {
        let mut app = app_with_source("abcdef");
        app.ensure_paginated(5, 10);
        app.dispatch(Msg::Char('a'));
        app.dispatch(Msg::Char('b'));
        let ptr = app.cursor.cu_ptr;
        app.ensure_paginated(5, 10);
        assert_eq!(app.cursor.cu_ptr, ptr);
    }
}
