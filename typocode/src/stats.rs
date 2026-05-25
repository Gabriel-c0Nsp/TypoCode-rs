//! Keystroke accuracy tracking.
//!
//! [`Stats`] counts every typing keystroke exactly once: the player
//! pressed a key, the update layer classified it as [`Keystroke::Correct`]
//! or [`Keystroke::Incorrect`], and the counter ticks up. Backspace,
//! Tab and Escape are *not* keystrokes for accuracy purposes — correct
//! the C version likewise doesn't penalise the player for recovering.
//!
//! Accuracy is reported as a rounded percentage so the footer can show
//! a compact `100%` / `97%` instead of `0.9712…`.
//!
//! Words-per-minute follows the SpeedTyper.dev convention: gross WPM =
//! (correct chars / 5) / elapsed minutes. Only correct keystrokes
//! contribute, so spamming wrongs never inflates WPM — incorrects show
//! up through the accuracy figure instead.

use std::time::Duration;

/// Classification of a single typing keystroke, produced by the update
/// layer and consumed by [`Stats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keystroke {
    /// The key matched the expected cell and advanced the cursor.
    Correct,
    /// The key was rejected; it either stacked as an extra or was
    /// swallowed at the line-tail cap.
    Incorrect,
}

/// Running totals of correct vs incorrect keystrokes for the current
/// run. Reset on Tab so restarts start from a clean slate.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    correct: u64,
    incorrect: u64,
}

impl Stats {
    /// A fresh stats counter with zero keystrokes recorded.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one keystroke of the given classification.
    pub fn record(&mut self, kind: Keystroke) {
        match kind {
            Keystroke::Correct => self.correct += 1,
            Keystroke::Incorrect => self.incorrect += 1,
        }
    }

    /// Total keystrokes recorded across both classifications.
    pub fn total(&self) -> u64 {
        self.correct + self.incorrect
    }

    /// Correct keystrokes only.
    pub fn correct(&self) -> u64 {
        self.correct
    }

    /// Incorrect keystrokes only.
    pub fn incorrect(&self) -> u64 {
        self.incorrect
    }

    /// Accuracy as a rounded 0..=100 percentage. Returns 100 when no
    /// keystrokes have been recorded so the footer reads cleanly at
    /// the start of a run rather than flashing `0%`.
    pub fn accuracy_percent(&self) -> u8 {
        let total = self.total();
        if total == 0 {
            return 100;
        }
        // Rounded integer percent: (correct * 100 + total/2) / total.
        let numerator = self.correct * 100 + total / 2;
        (numerator / total) as u8
    }

    /// Gross WPM rounded to the nearest whole number. Formula matches
    /// SpeedTyper.dev: `(correct / 5) / minutes`. Returns 0 while the
    /// stopwatch is idle or no correct keystrokes have landed yet so the
    /// footer doesn't divide-by-zero or flash an absurd burst figure on
    /// the very first keystroke.
    pub fn wpm(&self, elapsed: Duration) -> u32 {
        let secs = elapsed.as_secs_f64();
        if secs <= 0.0 || self.correct == 0 {
            return 0;
        }
        let minutes = secs / 60.0;
        let wpm = (self.correct as f64 / 5.0) / minutes;
        wpm.round() as u32
    }

    /// Clears both counters.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_stats_report_full_accuracy() {
        let s = Stats::new();
        assert_eq!(s.total(), 0);
        assert_eq!(s.accuracy_percent(), 100);
    }

    #[test]
    fn record_correct_increments_correct_only() {
        let mut s = Stats::new();
        s.record(Keystroke::Correct);
        assert_eq!(s.correct(), 1);
        assert_eq!(s.incorrect(), 0);
        assert_eq!(s.total(), 1);
    }

    #[test]
    fn record_incorrect_increments_incorrect_only() {
        let mut s = Stats::new();
        s.record(Keystroke::Incorrect);
        assert_eq!(s.correct(), 0);
        assert_eq!(s.incorrect(), 1);
        assert_eq!(s.total(), 1);
    }

    #[test]
    fn accuracy_reports_rounded_percent() {
        let mut s = Stats::new();
        for _ in 0..97 {
            s.record(Keystroke::Correct);
        }
        for _ in 0..3 {
            s.record(Keystroke::Incorrect);
        }
        assert_eq!(s.accuracy_percent(), 97);
    }

    #[test]
    fn accuracy_rounds_half_up() {
        // 1 correct / 3 total = 33.33% → rounds to 33.
        let mut s = Stats::new();
        s.record(Keystroke::Correct);
        s.record(Keystroke::Incorrect);
        s.record(Keystroke::Incorrect);
        assert_eq!(s.accuracy_percent(), 33);

        // 2 correct / 3 total = 66.66% → rounds to 67.
        let mut s = Stats::new();
        s.record(Keystroke::Correct);
        s.record(Keystroke::Correct);
        s.record(Keystroke::Incorrect);
        assert_eq!(s.accuracy_percent(), 67);
    }

    #[test]
    fn reset_clears_counters() {
        let mut s = Stats::new();
        s.record(Keystroke::Correct);
        s.record(Keystroke::Incorrect);
        s.reset();
        assert_eq!(s.total(), 0);
        assert_eq!(s.accuracy_percent(), 100);
    }

    #[test]
    fn wpm_is_zero_when_elapsed_is_zero() {
        let mut s = Stats::new();
        for _ in 0..100 {
            s.record(Keystroke::Correct);
        }
        assert_eq!(s.wpm(Duration::ZERO), 0);
    }

    #[test]
    fn wpm_is_zero_with_no_correct_keystrokes() {
        let mut s = Stats::new();
        for _ in 0..50 {
            s.record(Keystroke::Incorrect);
        }
        assert_eq!(s.wpm(Duration::from_secs(60)), 0);
    }

    #[test]
    fn wpm_uses_correct_chars_only_speedtyper_formula() {
        // 100 correct + 50 wrong over 60s → (100/5) / 1min = 20 wpm.
        let mut s = Stats::new();
        for _ in 0..100 {
            s.record(Keystroke::Correct);
        }
        for _ in 0..50 {
            s.record(Keystroke::Incorrect);
        }
        assert_eq!(s.wpm(Duration::from_secs(60)), 20);
    }

    #[test]
    fn wpm_scales_with_short_elapsed_windows() {
        // 50 correct chars in 10s → (50/5) / (10/60) = 60 wpm.
        let mut s = Stats::new();
        for _ in 0..50 {
            s.record(Keystroke::Correct);
        }
        assert_eq!(s.wpm(Duration::from_secs(10)), 60);
    }

    #[test]
    fn wpm_rounds_to_nearest_whole() {
        // 17 correct / 5 / (30/60) = 6.8 → rounds to 7.
        let mut s = Stats::new();
        for _ in 0..17 {
            s.record(Keystroke::Correct);
        }
        assert_eq!(s.wpm(Duration::from_secs(30)), 7);
    }
}
