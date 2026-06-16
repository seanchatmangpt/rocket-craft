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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── WasmWorker state transitions ────────────────────────────────────────

    #[test]
    fn worker_starts_uninitialized_then_runs() {
        let w = WasmWorker::<Uninitialized>::new("worker.js", 0);
        let running = w.start().expect("should start");
        assert_eq!(running.worker_id(), 0);
        let _terminated = running.terminate();
    }

    #[test]
    fn worker_pause_resume_cycle() {
        let w = WasmWorker::<Uninitialized>::new("worker.js", 1)
            .start()
            .unwrap();
        let paused = w.pause();
        let running = paused.resume();
        let _done = running.terminate();
    }

    #[test]
    fn worker_script_url_preserved() {
        let url = "https://example.com/worker.js";
        let w = WasmWorker::<Uninitialized>::new(url, 7)
            .start()
            .unwrap();
        assert_eq!(w.script_url(), url);
    }

    #[test]
    fn terminated_worker_reports_terminated() {
        let t = WasmWorker::<Uninitialized>::new("w.js", 0)
            .start()
            .unwrap()
            .terminate();
        assert!(t.is_terminated());
    }

    // ── SharedMemoryBus ─────────────────────────────────────────────────────

    #[test]
    fn bus_write_read_roundtrip() {
        let mut bus = SharedMemoryBus::new(16);
        bus.write_i32(0, 42).unwrap();
        assert_eq!(bus.read_i32(0).unwrap(), 42);
    }

    #[test]
    fn bus_out_of_bounds_errors() {
        let bus = SharedMemoryBus::new(4);
        assert!(bus.read_i32(10).is_err());
    }

    #[test]
    fn bus_write_out_of_bounds_errors() {
        let mut bus = SharedMemoryBus::new(4);
        assert!(bus.write_i32(10, 99).is_err());
    }

    /// Falsification: verify computation depends on input, not a mocked constant.
    #[test]
    fn bus_different_values_at_different_offsets() {
        let mut bus = SharedMemoryBus::new(8);
        bus.write_i32(0, 100).unwrap();
        bus.write_i32(1, 200).unwrap();
        assert_ne!(bus.read_i32(0).unwrap(), bus.read_i32(1).unwrap());
    }

    #[test]
    fn bus_size_matches_requested() {
        let bus = SharedMemoryBus::new(32);
        assert_eq!(bus.size(), 32);
    }

    proptest! {
        #[test]
        fn bus_roundtrip_arbitrary_values(offset in 0usize..8, value in i32::MIN..i32::MAX) {
            let mut bus = SharedMemoryBus::new(16);
            bus.write_i32(offset, value).unwrap();
            prop_assert_eq!(bus.read_i32(offset).unwrap(), value);
        }

        #[test]
        fn compare_exchange_only_swaps_when_expected_matches(
            initial in 0i32..100,
            wrong   in 101i32..200,
            new_val in 201i32..300,
        ) {
            let mut bus = SharedMemoryBus::new(4);
            bus.write_i32(0, initial).unwrap();

            // Wrong expected → no swap
            let swapped = bus.compare_exchange(0, wrong, new_val);
            prop_assert!(!swapped);
            prop_assert_eq!(bus.read_i32(0).unwrap(), initial);

            // Correct expected → swaps
            let swapped = bus.compare_exchange(0, initial, new_val);
            prop_assert!(swapped);
            prop_assert_eq!(bus.read_i32(0).unwrap(), new_val);
        }
    }

    // ── WorkerChannel ───────────────────────────────────────────────────────

    #[test]
    fn channel_send_receive_typed_messages() {
        #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Ping {
            id: u32,
        }

        let mut ch = WorkerChannel::<Ping>::new(0);
        ch.send(Ping { id: 42 }).unwrap();
        ch.send(Ping { id: 99 }).unwrap();
        assert_eq!(ch.pending_count(), 2);
        assert_eq!(ch.receive(), Some(Ping { id: 42 }));
        assert_eq!(ch.pending_count(), 1);
    }

    #[test]
    fn channel_receive_empty_returns_none() {
        let mut ch = WorkerChannel::<u32>::new(0);
        assert_eq!(ch.receive(), None);
    }

    #[test]
    fn channel_drain_clears_queue() {
        let mut ch = WorkerChannel::<u32>::new(0);
        ch.send(1).unwrap();
        ch.send(2).unwrap();
        ch.send(3).unwrap();
        let drained = ch.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert_eq!(ch.pending_count(), 0);
    }

    // ── WorkerPool ──────────────────────────────────────────────────────────

    #[test]
    fn pool_size_matches_requested() {
        let pool = WorkerPool::new(4, "worker.js").unwrap();
        assert_eq!(pool.size(), 4);
        assert_eq!(pool.worker_ids().len(), 4);
    }

    #[test]
    fn pool_round_robin_cycles() {
        let mut pool = WorkerPool::new(3, "worker.js").unwrap();
        let id0 = pool.dispatch_to_next().worker_id();
        let id1 = pool.dispatch_to_next().worker_id();
        let id2 = pool.dispatch_to_next().worker_id();
        let id3 = pool.dispatch_to_next().worker_id(); // wraps around
        assert_ne!(id0, id1);
        assert_ne!(id1, id2);
        assert_eq!(id0, id3); // round-robin returns to first
    }

    #[test]
    fn pool_terminate_all_returns_terminated_workers() {
        let pool = WorkerPool::new(2, "worker.js").unwrap();
        let terminated = pool.terminate_all();
        assert_eq!(terminated.len(), 2);
        for t in &terminated {
            assert!(t.is_terminated());
        }
    }

    // ── ThreadingApproach ───────────────────────────────────────────────────

    #[test]
    fn separate_modules_does_not_require_coop_coep() {
        let a = ThreadingApproach::SeparateModules { worker_count: 2 };
        assert!(!a.requires_coop_coep());
    }

    #[test]
    fn shared_memory_requires_coop_coep() {
        let a = ThreadingApproach::SharedMemory {
            buffer_size_bytes: 1024,
        };
        assert!(a.requires_coop_coep());
    }

    #[test]
    fn hybrid_requires_coop_coep() {
        let a = ThreadingApproach::Hybrid {
            worker_count: 4,
            shared_buffer_size_bytes: 4096,
        };
        assert!(a.requires_coop_coep());
    }

    #[test]
    fn recommended_for_game_logic_is_separate_modules() {
        let a = ThreadingApproach::recommended_for_game_logic();
        matches!(a, ThreadingApproach::SeparateModules { worker_count: 2 });
        assert!(!a.requires_coop_coep());
    }

    #[test]
    fn shared_memory_worker_count_is_one() {
        let a = ThreadingApproach::SharedMemory {
            buffer_size_bytes: 512,
        };
        assert_eq!(a.worker_count(), 1);
    }

    proptest! {
        #[test]
        fn approach_worker_count_consistent(n in 1usize..64) {
            let a = ThreadingApproach::SeparateModules { worker_count: n };
            prop_assert_eq!(a.worker_count(), n);
        }

        #[test]
        fn hybrid_worker_count_consistent(n in 1usize..64, buf in 1usize..65536) {
            let a = ThreadingApproach::Hybrid {
                worker_count: n,
                shared_buffer_size_bytes: buf,
            };
            prop_assert_eq!(a.worker_count(), n);
            prop_assert!(a.requires_coop_coep());
        }
    }
}
