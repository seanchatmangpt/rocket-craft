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
