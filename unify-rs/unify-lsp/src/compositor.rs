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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_server(name: &str) -> ServerEntry {
        ServerEntry { name: name.into(), language: "rust".into(), weight: 1.0 }
    }

    #[test]
    fn new_compositor_has_no_servers() {
        let c = CompositorState::new();
        assert_eq!(c.server_count(), 0);
    }

    #[test]
    fn add_server_increments_count() {
        let mut c = CompositorState::new();
        c.add_server(make_server("rust-analyzer"));
        c.add_server(make_server("anti-llm"));
        assert_eq!(c.server_count(), 2);
    }

    #[test]
    fn health_initial_gate_open_and_healthy() {
        let c = CompositorState::new();
        let h = c.health();
        assert!(h.gate_open);
        assert_eq!(h.error_count, 0);
        assert!(h.healthy);
    }

    #[test]
    fn raised_andon_makes_health_unhealthy() {
        let mut c = CompositorState::new();
        c.raise_andon("test reason");
        let h = c.health();
        assert!(!h.gate_open);
        assert!(!h.healthy);
    }

    #[test]
    fn lower_andon_after_raise_restores_healthy() {
        let mut c = CompositorState::new();
        c.raise_andon("r");
        c.lower_andon();
        let h = c.health();
        assert!(h.gate_open);
        assert!(h.healthy);
    }

    #[test]
    fn health_server_count_matches() {
        let mut c = CompositorState::new();
        c.add_server(make_server("s1"));
        c.add_server(make_server("s2"));
        c.add_server(make_server("s3"));
        assert_eq!(c.health().server_count, 3);
    }

    #[test]
    fn compositor_health_serializes() {
        let h = CompositorHealth { server_count: 2, error_count: 0, gate_open: true, healthy: true };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("server_count"));
        assert!(json.contains("healthy"));
    }
}
