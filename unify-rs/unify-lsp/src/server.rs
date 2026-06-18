use lsp_max_protocol::{ConformanceVector, SnapshotId};

/// Rocket Craft LSP server built on lsp-max.
///
/// Wire-up checklist:
/// - `impl lsp_max::LanguageServer for UnifyLspServer` — initialize, initialized,
///   shutdown, did_open, did_change, hover, completion
/// - Call `compositor.update_snapshot(uri, version, text)` on did_change
/// - Produce diagnostics via the `diagnostic` module and push them through the
///   lsp-max client's `publish_diagnostics()` method
/// - Register capabilities from `capability::CapabilitySet` in `initialize` response
/// - Mirror diagnostics into `lsp_max::REGISTRY` for AutonomicMesh observation
pub struct UnifyLspServer {
    _conformance: ConformanceVector,
}

impl UnifyLspServer {
    pub fn new() -> Self {
        Self {
            _conformance: ConformanceVector::default(),
        }
    }

    pub fn conformance_vector(&self) -> &ConformanceVector {
        &self._conformance
    }

    pub fn snapshot_id_for(uri: &str, version: i32) -> SnapshotId {
        SnapshotId(format!("{}@{}", uri, version))
    }
}

impl Default for UnifyLspServer {
    fn default() -> Self {
        Self::new()
    }
}
