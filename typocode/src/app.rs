//! Top-level application state and run loop.
//!
//! TEA-inspired: the loop is `poll → handle_event → view`. Subsequent FR
//! branches grow the state (pages, cursor, stats, timer) and the view.

use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph},
};

use crate::file::SourceFile;
use crate::stats::Stats;
use crate::text::{Cell, CellState, Page, Pages, gutter_labels, paginate};
use crate::timer::{Stopwatch, format_mm_ss};
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
    source: SourceFile,
    pages: Option<Pages>,
    cursor: Cursor,
    stopwatch: Stopwatch,
    stats: Stats,
    phase: Phase,
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
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        let gutter_width = gutter_column_width(self.source.line_count);
        let [gutter_area, body_area] =
            Layout::horizontal([Constraint::Length(gutter_width), Constraint::Min(0)])
                .areas(main_area);

        self.ensure_paginated(body_area.height, body_area.width);

        let mut cursor_screen: Option<(u16, u16)> = None;
        let (gutter_text, body_text, footer_text) = match &self.pages {
            Some(pages) => {
                let page = pages.current();
                let page_chars = page.chars();
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
                let body = styled_body(&page.cells, body_area.width as usize);
                cursor_screen = Some(cursor_screen_pos(
                    &page.cells,
                    self.cursor.cu_ptr,
                    self.cursor.extras.len(),
                    body_area.width,
                ));
                let elapsed = format_mm_ss(self.stopwatch.elapsed(Instant::now()));
                let accuracy = self.stats.accuracy_percent();
                (
                    Text::from(gutter),
                    body,
                    format!(
                        "{elapsed}  {accuracy}%  page {} / {}",
                        pages.current_index(),
                        pages.total()
                    ),
                )
            }
            None => (Text::default(), Text::default(), String::new()),
        };

        frame.render_widget(Paragraph::new(gutter_text), gutter_area);
        frame.render_widget(Paragraph::new(body_text), body_area);
        frame.render_widget(Paragraph::new(footer_text), footer_area);

        if let Some(pages) = &self.pages {
            draw_extras_overlay(frame, body_area, pages.current(), &self.cursor);
        }

        if self.phase == Phase::Finished {
            let elapsed = format_mm_ss(self.stopwatch.elapsed(Instant::now()));
            let accuracy = self.stats.accuracy_percent();
            draw_summary_overlay(frame, body_area, &elapsed, accuracy);
            return;
        }

        if let Some((col, row)) = cursor_screen {
            let x = body_area.x.saturating_add(col);
            let y = body_area.y.saturating_add(row);
            if x < body_area.x + body_area.width && y < body_area.y + body_area.height {
                frame.set_cursor_position((x, y));
            }
        }
    }
}

/// Reserves enough columns for the widest source-line number plus a
/// single trailing space separating the gutter from the body.
fn gutter_column_width(total_lines: usize) -> u16 {
    let digits = total_lines.max(1).to_string().len() as u16;
    digits + 1
}

/// Maps `(cu_ptr, extras_len)` to the `(col, row)` the cursor should
/// occupy within the body area, applying char-wrap at `body_cols`.
///
/// Cells before `cu_ptr` already contributed to the visual layout —
/// each non-newline advances the column by one (wrapping at
/// `body_cols`), each newline resets to column 0 on the next row.
/// Extras are rendered to the right of `cu_ptr` so they bump the
/// column further, wrapping the same way. Returned `(0, 0)` when
/// `body_cols` is zero.
fn cursor_screen_pos(
    cells: &[Cell],
    cu_ptr: usize,
    extras_len: usize,
    body_cols: u16,
) -> (u16, u16) {
    if body_cols == 0 {
        return (0, 0);
    }
    let cols = body_cols as usize;
    let mut row: usize = 0;
    let mut col: usize = 0;
    let end = cu_ptr.min(cells.len());
    for cell in &cells[..end] {
        if cell.ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
            if col >= cols {
                col = 0;
                row += 1;
            }
        }
    }
    col += extras_len;
    if col >= cols {
        row += col / cols;
        col %= cols;
    }
    (col as u16, row as u16)
}

/// Draws the pending wrong-keystroke buffer on top of the rendered
/// body. Each extra visually sits at the cell index `cu_ptr + i`
/// (char-wrapped like the body). Space and newline inputs render as
/// `_` so the player can see that a special key was pressed in the
/// wrong place — matching the C version's underscore glyph. Extras
/// are painted red (FR-03) so wrong keystrokes stand out.
fn draw_extras_overlay(frame: &mut Frame, body_area: Rect, page: &Page, cursor: &Cursor) {
    if cursor.extras.is_empty() || body_area.width == 0 {
        return;
    }
    let style = Style::default().fg(Color::Red);
    let buf = frame.buffer_mut();
    for (i, &ex) in cursor.extras.iter().enumerate() {
        let (col, row) = cursor_screen_pos(&page.cells, cursor.cu_ptr, i, body_area.width);
        let x = body_area.x.saturating_add(col);
        let y = body_area.y.saturating_add(row);
        if x >= body_area.x + body_area.width || y >= body_area.y + body_area.height {
            continue;
        }
        let display = match ex {
            ' ' | '\n' => '_',
            c => c,
        };
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(display);
            cell.set_style(style);
        }
    }
}

/// Lays out `cells` into styled visual rows at width `cols`, mirroring
/// [`crate::text::wrap_content`] but emitting one [`Span`] per cell so
/// the renderer can colour each character by its [`CellState`]. Correct
/// cells are green, pending cells use the terminal default. `cols` is
/// clamped to 1 to match the wrap helper.
fn styled_body(cells: &[Cell], cols: usize) -> Text<'static> {
    let cols = cols.max(1);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();

    for cell in cells {
        if cell.ch == '\n' {
            lines.push(Line::from(std::mem::take(&mut current)));
            continue;
        }
        if current.len() == cols {
            lines.push(Line::from(std::mem::take(&mut current)));
        }
        current.push(styled_span(cell));
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    Text::from(lines)
}

/// Renders the end-of-run summary centred inside `body_area`: elapsed
/// time, accuracy, and the Tab / Esc hints. Uses [`Clear`] to erase
/// the typed body underneath so the summary is always legible even on
/// dense source files.
fn draw_summary_overlay(frame: &mut Frame, body_area: Rect, elapsed: &str, accuracy: u8) {
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
    let height = lines.len() as u16;
    let width = lines
        .iter()
        .map(|l| l.width() as u16)
        .max()
        .unwrap_or(0)
        .saturating_add(4);
    if body_area.width < width || body_area.height < height {
        // Viewport too small for the summary panel — fall back to
        // rendering it left-aligned over the body area so the finish
        // state is still visible.
        frame.render_widget(Clear, body_area);
        frame.render_widget(
            Paragraph::new(Text::from(lines.to_vec())).alignment(Alignment::Left),
            body_area,
        );
        return;
    }
    let x = body_area.x + (body_area.width - width) / 2;
    let y = body_area.y + (body_area.height - height) / 2;
    let area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines.to_vec())).alignment(Alignment::Center),
        area,
    );
}

fn styled_span(cell: &Cell) -> Span<'static> {
    let style = match cell.state {
        CellState::Correct => Style::default().fg(Color::Green),
        CellState::Pending => Style::default(),
    };
    Span::styled(cell.ch.to_string(), style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Cell;

    fn cells(s: &str) -> Vec<Cell> {
        s.chars().map(Cell::pending).collect()
    }

    #[test]
    fn cursor_at_origin_with_no_extras() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 0, 0, 10), (0, 0));
    }

    #[test]
    fn cursor_advances_within_a_line() {
        let cs = cells("abcdef");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 10), (3, 0));
    }

    #[test]
    fn newline_drops_cursor_to_next_row() {
        let cs = cells("ab\ncd");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 10), (0, 1));
        assert_eq!(cursor_screen_pos(&cs, 5, 0, 10), (2, 1));
    }

    #[test]
    fn char_wrap_bumps_row() {
        let cs = cells("abcdef");
        assert_eq!(cursor_screen_pos(&cs, 3, 0, 3), (0, 1));
        assert_eq!(cursor_screen_pos(&cs, 6, 0, 3), (0, 2));
    }

    #[test]
    fn extras_bump_column_and_may_wrap() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 1, 2, 10), (3, 0));
        // cu_ptr at col 2 with 4 extras at cols=4 → wraps once.
        assert_eq!(cursor_screen_pos(&cs, 2, 4, 4), (2, 1));
    }

    #[test]
    fn zero_body_cols_short_circuits_to_origin() {
        let cs = cells("abc");
        assert_eq!(cursor_screen_pos(&cs, 2, 0, 0), (0, 0));
    }
}
