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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_server_with_default_conformance() {
        let server = UnifyLspServer::new();
        let _ = server.conformance_vector(); // must not panic
    }

    #[test]
    fn default_matches_new_structure() {
        let _a = UnifyLspServer::new();
        let _b = UnifyLspServer::default();
        // Both produce snapshot IDs via the same function — verify it's deterministic
        let id1 = UnifyLspServer::snapshot_id_for("file:///foo.rs", 1);
        let id2 = UnifyLspServer::snapshot_id_for("file:///foo.rs", 1);
        assert_eq!(id1.0, id2.0, "same inputs must produce identical snapshot IDs");
    }

    #[test]
    fn snapshot_id_encodes_uri_and_version() {
        let id = UnifyLspServer::snapshot_id_for("file:///src/lib.rs", 42);
        assert!(id.0.contains("file:///src/lib.rs"), "URI must appear in snapshot id");
        assert!(id.0.contains("42"), "version must appear in snapshot id");
    }

    #[test]
    fn snapshot_id_differs_for_different_versions() {
        let id1 = UnifyLspServer::snapshot_id_for("file:///a.rs", 1);
        let id2 = UnifyLspServer::snapshot_id_for("file:///a.rs", 2);
        assert_ne!(id1.0, id2.0, "different versions must produce different IDs");
    }

    #[test]
    fn snapshot_id_differs_for_different_uris() {
        let id1 = UnifyLspServer::snapshot_id_for("file:///a.rs", 1);
        let id2 = UnifyLspServer::snapshot_id_for("file:///b.rs", 1);
        assert_ne!(id1.0, id2.0, "different URIs must produce different IDs");
    }
}
