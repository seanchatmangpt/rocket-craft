//! StateMaximalist — records every state transition during a test so you can
//! assert on the full history (inspired by un-test-utils StateMaximalist).

/// Tracks all state transitions in a test by storing labelled snapshots.
pub struct StateMaximalist<T: Clone + std::fmt::Debug> {
    history: Vec<(String, T)>,
}

impl<T: Clone + std::fmt::Debug> StateMaximalist<T> {
    /// Create an empty tracker.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Snapshot the current state under the given label.
    pub fn record(&mut self, label: impl Into<String>, state: &T) {
        self.history.push((label.into(), state.clone()));
    }

    /// Full ordered history of `(label, snapshot)` pairs.
    pub fn history(&self) -> &[(String, T)] {
        &self.history
    }

    /// The most-recently recorded entry, or `None` if nothing was recorded.
    pub fn last(&self) -> Option<&(String, T)> {
        self.history.last()
    }

    /// Number of recorded snapshots.
    pub fn count(&self) -> usize {
        self.history.len()
    }

    /// All labels in order.
    pub fn labels(&self) -> Vec<&str> {
        self.history.iter().map(|(l, _)| l.as_str()).collect()
    }

    /// Panic if the recorded labels do not exactly match `expected_labels` in order.
    pub fn assert_sequence(&self, expected_labels: &[&str]) {
        let actual: Vec<&str> = self.labels();
        assert_eq!(
            actual, expected_labels,
            "state sequence mismatch: expected {:?}, got {:?}",
            expected_labels, actual
        );
    }

    /// Panic if every recorded state snapshot is equal to every other one
    /// (i.e. no state change was observed).
    pub fn assert_state_changed(&self)
    where
        T: PartialEq,
    {
        assert!(
            self.history.len() >= 2,
            "assert_state_changed: need at least 2 snapshots, got {}",
            self.history.len()
        );
        let first = &self.history[0].1;
        let all_same = self.history.iter().all(|(_, s)| s == first);
        assert!(
            !all_same,
            "assert_state_changed: all {} snapshots are equal to {:?}",
            self.history.len(),
            first
        );
    }
}

impl<T: Clone + std::fmt::Debug> Default for StateMaximalist<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_history_have_correct_labels() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("step-1", &10);
        tracker.record("step-2", &20);
        tracker.record("step-3", &30);

        assert_eq!(tracker.count(), 3);
        assert_eq!(tracker.labels(), vec!["step-1", "step-2", "step-3"]);
    }

    #[test]
    fn last_returns_most_recent_snapshot() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("a", &1);
        tracker.record("b", &99);
        assert_eq!(tracker.last(), Some(&("b".to_string(), 99)));
    }

    #[test]
    fn assert_sequence_passes_with_correct_labels() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("given", &0);
        tracker.record("when", &1);
        tracker.record("then", &2);
        tracker.assert_sequence(&["given", "when", "then"]);
    }

    #[test]
    #[should_panic(expected = "state sequence mismatch")]
    fn assert_sequence_panics_with_wrong_labels() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("a", &1);
        tracker.assert_sequence(&["b"]);
    }

    #[test]
    #[should_panic(expected = "assert_state_changed")]
    fn assert_state_changed_panics_if_all_same() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("x", &5);
        tracker.record("y", &5);
        tracker.assert_state_changed();
    }

    #[test]
    fn assert_state_changed_passes_when_states_differ() {
        let mut tracker: StateMaximalist<i32> = StateMaximalist::new();
        tracker.record("before", &0);
        tracker.record("after", &1);
        tracker.assert_state_changed(); // should not panic
    }
}
