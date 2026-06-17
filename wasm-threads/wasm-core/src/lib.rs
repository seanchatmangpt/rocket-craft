pub mod approach;
pub mod channel;
pub mod memory;
pub mod pool;
pub mod worker;

pub use approach::ThreadingApproach;
pub use channel::{ChannelError, WorkerChannel};
pub use memory::{BusError, SharedMemoryBus};
pub use pool::WorkerPool;
pub use worker::{Paused, Running, Terminated, Uninitialized, WasmWorker, WorkerError};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_panic_hook() {
    // Enabled via the optional `console_error_panic_hook` feature.
    // Not wired up here to avoid adding an extra dep; callers can set it
    // directly if they add the crate.
}

/// JS-exposed context for managing the threading approach.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmThreadContext {
    approach: ThreadingApproach,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmThreadContext {
    #[wasm_bindgen(constructor)]
    pub fn new_separate_modules(worker_count: usize) -> Self {
        Self {
            approach: ThreadingApproach::SeparateModules { worker_count },
        }
    }

    pub fn requires_coop_coep(&self) -> bool {
        self.approach.requires_coop_coep()
    }

    pub fn worker_count(&self) -> usize {
        self.approach.worker_count()
    }

    pub fn approach_name(&self) -> String {
        format!("{:?}", self.approach)
    }
}

