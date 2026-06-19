/// State of the ANDON gate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AndonState {
    /// Gate is open — operations may proceed.
    Open,
    /// Gate is raised due to `reason` — operations are blocked.
    Raised(String),
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Conformance-driven gate that blocks LSP shell actions.
pub struct AndonGate {
    state: AndonState,
    /// History of (state, timestamp_ms) transitions.
    history: Vec<(AndonState, u64)>,
}

impl AndonGate {
    /// Create a new gate in the `Open` state.
    pub fn new() -> Self {
        Self {
            state: AndonState::Open,
            history: Vec::new(),
        }
    }

    /// Raise the gate with a reason, blocking further operations.
    pub fn raise(&mut self, reason: impl Into<String>) {
        let new_state = AndonState::Raised(reason.into());
        self.history.push((new_state.clone(), now_ms()));
        self.state = new_state;
    }

    /// Lower the gate, allowing operations to proceed again.
    pub fn lower(&mut self) {
        self.history.push((AndonState::Open, now_ms()));
        self.state = AndonState::Open;
    }

    /// Returns `true` if the gate is currently `Open`.
    pub fn is_open(&self) -> bool {
        self.state == AndonState::Open
    }

    /// Current state of the gate.
    pub fn state(&self) -> &AndonState {
        &self.state
    }

    /// Returns `Ok(())` when open, or `Err(reason)` when raised.
    pub fn check(&self) -> Result<(), String> {
        match &self.state {
            AndonState::Open => Ok(()),
            AndonState::Raised(reason) => Err(reason.clone()),
        }
    }

    /// Ordered history of state transitions with timestamps.
    pub fn history(&self) -> &[(AndonState, u64)] {
        &self.history
    }

    /// Total number of state-change events recorded.
    pub fn event_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for AndonGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gate_is_open() {
        let g = AndonGate::new();
        assert!(g.is_open());
        assert_eq!(g.check(), Ok(()));
        assert_eq!(g.event_count(), 0);
    }

    #[test]
    fn raise_blocks_operations() {
        let mut g = AndonGate::new();
        g.raise("conformance failure");
        assert!(!g.is_open());
        assert_eq!(g.check(), Err("conformance failure".into()));
    }

    #[test]
    fn raise_records_history() {
        let mut g = AndonGate::new();
        g.raise("reason");
        assert_eq!(g.event_count(), 1);
    }

    #[test]
    fn lower_opens_gate_after_raise() {
        let mut g = AndonGate::new();
        g.raise("bad");
        g.lower();
        assert!(g.is_open());
        assert_eq!(g.check(), Ok(()));
    }

    #[test]
    fn lower_records_history() {
        let mut g = AndonGate::new();
        g.raise("bad");
        g.lower();
        assert_eq!(g.event_count(), 2);
        assert_eq!(g.history()[1].0, AndonState::Open);
    }

    #[test]
    fn multiple_raises_accumulate_history() {
        let mut g = AndonGate::new();
        g.raise("a");
        g.raise("b");
        assert_eq!(g.event_count(), 2);
        if let AndonState::Raised(r) = &g.state() {
            assert_eq!(r, "b");
        } else { panic!("wrong state"); }
    }

    #[test]
    fn default_equals_new() {
        let g: AndonGate = Default::default();
        assert!(g.is_open());
    }
}
