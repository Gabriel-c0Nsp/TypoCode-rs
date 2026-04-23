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
}
