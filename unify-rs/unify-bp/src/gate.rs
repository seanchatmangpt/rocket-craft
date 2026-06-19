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

#[cfg(test)]
mod tests {
    use super::*;
    use blueprint_core::Blueprint;

    fn valid_bp() -> Blueprint {
        Blueprint::new("MyActor", "Actor")
    }

    fn empty_name_bp() -> Blueprint {
        Blueprint::new("", "Actor")
    }

    // ── BlueprintAdmissionGate::admit ─────────────────────────────────────────

    #[test]
    fn admit_passes_valid_blueprint() {
        let mut gate = BlueprintAdmissionGate::new();
        assert!(gate.admit(&valid_bp()).is_ok());
    }

    #[test]
    fn admit_fails_on_empty_name() {
        let mut gate = BlueprintAdmissionGate::new();
        let err = gate.admit(&empty_name_bp()).unwrap_err();
        assert!(err.iter().any(|v| v.contains("empty")));
    }

    #[test]
    fn admit_updates_state_to_closed_on_failure() {
        let mut gate = BlueprintAdmissionGate::new();
        let _ = gate.admit(&empty_name_bp());
        assert!(!gate.is_open());
    }

    #[test]
    fn admit_updates_state_to_open_on_success() {
        let mut gate = BlueprintAdmissionGate::new();
        let _ = gate.admit(&empty_name_bp()); // close it first
        let _ = gate.admit(&valid_bp());       // re-open
        assert!(gate.is_open());
    }

    // ── BlueprintAdmissionGate::validate ─────────────────────────────────────

    #[test]
    fn validate_returns_ok_for_valid_blueprint() {
        assert!(BlueprintAdmissionGate::validate(&valid_bp()).is_ok());
    }

    #[test]
    fn validate_returns_err_for_empty_name() {
        assert!(BlueprintAdmissionGate::validate(&empty_name_bp()).is_err());
    }

    // ── GateState / is_open ───────────────────────────────────────────────────

    #[test]
    fn new_gate_is_open() {
        let gate = BlueprintAdmissionGate::new();
        assert!(gate.is_open());
    }
}
