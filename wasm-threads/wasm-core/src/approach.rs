use serde::{Deserialize, Serialize};

/// The three browser WASM multi-threading approaches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreadingApproach {
    /// Multiple independent WASM modules, each in a separate Web Worker.
    /// No shared memory; communicate via postMessage.
    SeparateModules { worker_count: usize },

    /// Single WASM module spawns threads sharing a SharedArrayBuffer.
    /// Requires COOP/COEP headers. Uses WebAssembly.Memory { shared: true }.
    SharedMemory { buffer_size_bytes: usize },

    /// Hybrid: separate modules with a shared memory segment for hot data.
    Hybrid {
        worker_count: usize,
        shared_buffer_size_bytes: usize,
    },
}

impl ThreadingApproach {
    /// Returns `true` for any approach that requires the
    /// `Cross-Origin-Opener-Policy: same-origin` and
    /// `Cross-Origin-Embedder-Policy: require-corp` response headers.
    pub fn requires_coop_coep(&self) -> bool {
        matches!(self, Self::SharedMemory { .. } | Self::Hybrid { .. })
    }

    /// Total number of Web Workers used by this approach.
    pub fn worker_count(&self) -> usize {
        match self {
            Self::SeparateModules { worker_count } => *worker_count,
            Self::SharedMemory { .. } => 1,
            Self::Hybrid { worker_count, .. } => *worker_count,
        }
    }

    /// The default recommended approach for a game-logic + UI split.
    /// Two separate modules avoids SharedArrayBuffer header requirements
    /// while still keeping game logic off the main thread.
    pub fn recommended_for_game_logic() -> Self {
        Self::SeparateModules { worker_count: 2 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_modules_does_not_require_coop_coep() {
        let a = ThreadingApproach::SeparateModules { worker_count: 2 };
        assert!(!a.requires_coop_coep());
    }

    #[test]
    fn shared_memory_requires_coop_coep() {
        let a = ThreadingApproach::SharedMemory { buffer_size_bytes: 1024 };
        assert!(a.requires_coop_coep());
    }

    #[test]
    fn hybrid_requires_coop_coep() {
        let a = ThreadingApproach::Hybrid { worker_count: 3, shared_buffer_size_bytes: 512 };
        assert!(a.requires_coop_coep());
    }

    #[test]
    fn separate_modules_worker_count() {
        let a = ThreadingApproach::SeparateModules { worker_count: 4 };
        assert_eq!(a.worker_count(), 4);
    }

    #[test]
    fn shared_memory_worker_count_is_one() {
        let a = ThreadingApproach::SharedMemory { buffer_size_bytes: 0 };
        assert_eq!(a.worker_count(), 1);
    }

    #[test]
    fn hybrid_worker_count_returns_worker_field() {
        let a = ThreadingApproach::Hybrid { worker_count: 5, shared_buffer_size_bytes: 0 };
        assert_eq!(a.worker_count(), 5);
    }

    #[test]
    fn recommended_for_game_logic_is_separate_modules_two_workers() {
        match ThreadingApproach::recommended_for_game_logic() {
            ThreadingApproach::SeparateModules { worker_count: 2 } => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn threading_approach_serializes() {
        let a = ThreadingApproach::SharedMemory { buffer_size_bytes: 4096 };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("4096"));
    }
}
