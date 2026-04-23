//! Elapsed-time tracking for the active typing run.
//!
//! [`Stopwatch`] holds an optional start [`Instant`] and reports the
//! time between it and a caller-supplied "now". Passing the clock in
//! keeps the type deterministic for unit tests — production code hands
//! it [`Instant::now`], tests hand it a fixed value.
//!
//! The stopwatch is idle until the first keystroke of a run reaches the
//! update layer, mirroring the C version's `started_test` flag. Tab
//! restart resets it back to idle so the next keystroke begins a fresh
//! timing.

use std::time::{Duration, Instant};

/// Tracks the elapsed time of the current typing run.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stopwatch {
    started_at: Option<Instant>,
}

impl Stopwatch {
    /// A fresh, idle stopwatch.
    pub fn new() -> Self {
        Self { started_at: None }
    }

    /// Starts the stopwatch if it isn't already running. Subsequent
    /// calls are no-ops so re-invoking on every keystroke is safe.
    pub fn start(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
    }

    /// Returns the time elapsed since [`start`] was first called, or
    /// zero if the stopwatch is idle. Uses
    /// [`Instant::saturating_duration_since`] so callers never observe
    /// a negative/panic if the passed `now` is somehow earlier than the
    /// recorded start — that can only happen under a mocked clock.
    pub fn elapsed(&self, now: Instant) -> Duration {
        self.started_at
            .map(|s| now.saturating_duration_since(s))
            .unwrap_or_default()
    }

    /// Returns the stopwatch to its idle state.
    pub fn reset(&mut self) {
        self.started_at = None;
    }

    /// Whether the stopwatch has been started.
    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }
}

/// Formats `d` as `mm:ss`, clamped to two-digit minutes in the common
/// case. Runs longer than 99 minutes keep growing the minutes field
/// rather than wrapping so the footer never silently lies.
pub fn format_mm_ss(d: Duration) -> String {
    let total = d.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_stopwatch_reports_zero() {
        let sw = Stopwatch::new();
        let now = Instant::now();
        assert_eq!(sw.elapsed(now), Duration::ZERO);
        assert!(!sw.is_running());
    }

    #[test]
    fn start_sets_running_and_elapsed_tracks_delta() {
        let mut sw = Stopwatch::new();
        let t0 = Instant::now();
        sw.start(t0);
        assert!(sw.is_running());
        assert_eq!(
            sw.elapsed(t0 + Duration::from_secs(3)),
            Duration::from_secs(3)
        );
    }

    #[test]
    fn second_start_is_ignored() {
        let mut sw = Stopwatch::new();
        let t0 = Instant::now();
        sw.start(t0);
        sw.start(t0 + Duration::from_secs(5));
        assert_eq!(
            sw.elapsed(t0 + Duration::from_secs(10)),
            Duration::from_secs(10)
        );
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut sw = Stopwatch::new();
        let t0 = Instant::now();
        sw.start(t0);
        sw.reset();
        assert!(!sw.is_running());
        assert_eq!(sw.elapsed(t0 + Duration::from_secs(5)), Duration::ZERO);
    }

    #[test]
    fn reset_then_start_picks_up_new_origin() {
        let mut sw = Stopwatch::new();
        let t0 = Instant::now();
        sw.start(t0);
        sw.reset();
        let t1 = t0 + Duration::from_secs(10);
        sw.start(t1);
        assert_eq!(
            sw.elapsed(t1 + Duration::from_secs(2)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn format_mm_ss_pads_both_fields() {
        assert_eq!(format_mm_ss(Duration::from_secs(0)), "00:00");
        assert_eq!(format_mm_ss(Duration::from_secs(7)), "00:07");
        assert_eq!(format_mm_ss(Duration::from_secs(65)), "01:05");
        assert_eq!(format_mm_ss(Duration::from_secs(60 * 12 + 34)), "12:34");
    }

    #[test]
    fn format_mm_ss_grows_minutes_past_100() {
        assert_eq!(format_mm_ss(Duration::from_secs(60 * 120)), "120:00");
    }
}
