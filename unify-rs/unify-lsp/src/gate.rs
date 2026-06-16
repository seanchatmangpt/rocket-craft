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
