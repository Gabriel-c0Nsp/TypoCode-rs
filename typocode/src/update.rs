//! Input-to-state dispatch.
//!
//! The run loop translates each crossterm event into a [`Msg`] via
//! [`from_key_event`], then calls [`update`] to mutate the application
//! state. Handlers live in this module — kept free of rendering so
//! unit tests can drive the full typing machine without a terminal.
//!
//! The per-key semantics mirror the C version's `input/input.c`:
//! strict character match, wrong keystrokes stack as extras, backspace
//! is required to recover, and Tab restarts the run.

use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::app::Cursor;
use crate::text::Pages;

/// One typing-loop event, normalised across the platform key variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    /// A printable character that isn't space, tab, enter or escape.
    Char(char),
    /// Space bar.
    Space,
    /// Enter / return.
    Enter,
    /// Backspace.
    Backspace,
    /// Tab — restart the current run.
    Tab,
    /// Escape — quit the app.
    Quit,
}

/// Outcome of dispatching a single [`Msg`].
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateOutcome {
    /// Set when the player asked to exit; the run loop should tear down.
    pub should_quit: bool,
}

/// Converts a crossterm event into a [`Msg`]. Returns `None` for events
/// we don't care about (key release / repeat, non-key events, modifier
/// chords). The caller should ignore `None`.
pub fn from_key_event(event: &Event) -> Option<Msg> {
    let Event::Key(key) = event else { return None };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Esc => Some(Msg::Quit),
        KeyCode::Tab => Some(Msg::Tab),
        KeyCode::Backspace => Some(Msg::Backspace),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Char(' ') => Some(Msg::Space),
        KeyCode::Char(c) => Some(Msg::Char(c)),
        _ => None,
    }
}

/// Applies `msg` against the current [`Pages`] and [`Cursor`].
///
/// Handlers for typing keys are added by subsequent commits; this
/// skeleton only wires up the quit path so the run loop can adopt the
/// `Msg` abstraction in one go.
pub fn update(_pages: &mut Pages, _cursor: &mut Cursor, msg: Msg) -> UpdateOutcome {
    match msg {
        Msg::Quit => UpdateOutcome { should_quit: true },
        Msg::Char(_) | Msg::Space | Msg::Enter | Msg::Backspace | Msg::Tab => {
            UpdateOutcome::default()
        }
    }
}
