//! BlueprintAdmissionGate — validate a Blueprint before serialization.

use blueprint_core::Blueprint;

/// Errors produced by the admission gate.
#[derive(Debug, Clone, PartialEq)]
pub enum GateError {
    EmptyName,
    NoGraphs,
    Custom(String),
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::EmptyName => write!(f, "Blueprint name must not be empty"),
            GateError::NoGraphs => write!(f, "Blueprint must have at least one graph"),
            GateError::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for GateError {}

/// Internal gate state — open means admissible, closed means blocked.
#[derive(Debug, Clone, PartialEq)]
pub enum GateState {
    Open,
    Closed,
}

/// A reusable admission gate that validates Blueprint objects.
pub struct BlueprintAdmissionGate {
    state: GateState,
    violations: Vec<String>,
}

impl Default for BlueprintAdmissionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintAdmissionGate {
    /// Create a new gate in the `Open` state (no violations yet).
    pub fn new() -> Self {
        Self {
            state: GateState::Open,
            violations: Vec::new(),
        }
    }

    /// Check a Blueprint is admissible: non-empty name, at least one graph.
    ///
    /// Returns `Ok(())` on success, `Err(violations)` on failure.
    pub fn admit(&mut self, bp: &Blueprint) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        if bp.name.trim().is_empty() {
            violations.push(GateError::EmptyName.to_string());
        }
        if bp.graphs.is_empty() {
            violations.push(GateError::NoGraphs.to_string());
        }

        if violations.is_empty() {
            self.state = GateState::Open;
            self.violations.clear();
            Ok(())
        } else {
            self.state = GateState::Closed;
            self.violations = violations.clone();
            Err(violations)
        }
    }

    /// Returns `true` when the gate is currently open (last `admit` passed, or
    /// newly constructed).
    pub fn is_open(&self) -> bool {
        self.state == GateState::Open
    }

    /// Consuming builder-style: call `admit` and return `self` for chaining.
    pub fn raise_for_blueprint(mut self, bp: &Blueprint) -> Self {
        let _ = self.admit(bp);
        self
    }

    /// Stateless convenience: validate and return `bp` on success or
    /// `Err(violations)` on failure.
    pub fn validate(bp: &Blueprint) -> Result<&Blueprint, Vec<String>> {
        let mut gate = BlueprintAdmissionGate::new();
        gate.admit(bp).map(|_| bp)
    }
}
