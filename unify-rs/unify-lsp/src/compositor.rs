use crate::diagnostic::DiagnosticSet;
use crate::gate::AndonGate;

/// A named language server participating in the compositor.
pub struct ServerEntry {
    pub name: String,
    pub language: String,
    /// Weight used for quorum debounce (0.0–1.0 typical).
    pub weight: f64,
}

/// Aggregated health report from the compositor.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompositorHealth {
    pub server_count: usize,
    pub error_count: usize,
    pub gate_open: bool,
    /// Healthy when gate is open and no errors are present.
    pub healthy: bool,
}

/// Multi-server fan-out hub: merges diagnostics and tracks gate state.
pub struct CompositorState {
    servers: Vec<ServerEntry>,
    diagnostics: DiagnosticSet,
    gate: AndonGate,
}

impl CompositorState {
    /// Create a compositor with no servers.
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            diagnostics: DiagnosticSet::new(),
            gate: AndonGate::new(),
        }
    }

    /// Register a language server with the compositor.
    pub fn add_server(&mut self, entry: ServerEntry) {
        self.servers.push(entry);
    }

    /// Merge diagnostics from another set into this compositor's set.
    pub fn merge_diagnostics(&mut self, other: DiagnosticSet) {
        self.diagnostics.merge(other);
    }

    /// Raise the ANDON gate, blocking shell actions.
    pub fn raise_andon(&mut self, reason: impl Into<String>) {
        self.gate.raise(reason);
    }

    /// Lower the ANDON gate, permitting operations.
    pub fn lower_andon(&mut self) {
        self.gate.lower();
    }

    /// Compute the current health of the compositor.
    pub fn health(&self) -> CompositorHealth {
        let server_count = self.servers.len();
        let error_count = self.diagnostics.error_count();
        let gate_open = self.gate.is_open();
        let healthy = gate_open && error_count == 0;
        CompositorHealth {
            server_count,
            error_count,
            gate_open,
            healthy,
        }
    }

    /// Number of registered servers.
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }
}

impl Default for CompositorState {
    fn default() -> Self {
        Self::new()
    }
}
