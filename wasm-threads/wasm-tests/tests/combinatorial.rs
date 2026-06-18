// Combinatorial proptest matrix across all subsystems

use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_core::{SharedMemoryBus, ThreadingApproach, WorkerPool};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

proptest! {
    /// Matrix: (worker_count, buffer_size, health, max_health, tick).
    /// Every dimension must affect the output — no shortcuts.
    #[test]
    fn combinatorial_system_state(
        worker_count in 1usize..8,
        buffer_size in 8usize..256,
        health in 0u32..1000,
        max_health in 1u32..1000,
        tick in 0u64..10000,
    ) {
        let log = log();
        log.info(&format!(
            "Given workers={worker_count} buf={buffer_size} hp={health}/{max_health} tick={tick}"
        ));
        let health = health.min(max_health);

        log.info("When a WorkerPool and SharedMemoryBus are created with these parameters");
        // WorkerPool: size must match request.
        let pool = WorkerPool::new(worker_count, "worker.js").unwrap();
        prop_assert_eq!(pool.size(), worker_count, "pool size must equal requested worker_count");

        // SharedMemoryBus: round-trip must be exact.
        let mut bus = SharedMemoryBus::new(buffer_size);
        bus.write_i32(0, health as i32).unwrap();
        bus.write_i32(1, max_health as i32).unwrap();
        // Write tick into slot 2 (clamped to i32 range; tick is u64 but fits
        // for the purpose of this test since it is bounded to 10000).
        bus.write_i32(2, tick as i32).unwrap();

        log.info("Then bus round-trips and health percentage invariants must hold");
        prop_assert_eq!(bus.read_i32(0).unwrap(), health as i32);
        prop_assert_eq!(bus.read_i32(1).unwrap(), max_health as i32);

        // Health percentage must be in [0.0, 1.0].
        let pct = health as f32 / max_health as f32;
        prop_assert!(pct >= 0.0 && pct <= 1.0, "health percentage must be in [0.0, 1.0]");

        // Two pools with the same count must have the same size.
        let pool2 = WorkerPool::new(worker_count, "worker.js").unwrap();
        prop_assert_eq!(pool.size(), pool2.size(), "same worker_count must produce same pool size");
    }

    /// Matrix: COOP/COEP requirements for each ThreadingApproach variant.
    #[test]
    fn approach_coop_coep_matrix(
        worker_count in 1usize..16,
        buffer_size in 1024usize..1_048_576,
    ) {
        let log = log();
        log.info(&format!("Given workers={worker_count} buf={buffer_size}"));
        let sep = ThreadingApproach::SeparateModules { worker_count };
        let shm = ThreadingApproach::SharedMemory { buffer_size_bytes: buffer_size };
        let hyb = ThreadingApproach::Hybrid {
            worker_count,
            shared_buffer_size_bytes: buffer_size,
        };

        log.info("When COOP/COEP flags are checked for all three approaches");
        prop_assert!(!sep.requires_coop_coep(), "SeparateModules must NOT require COOP/COEP");
        prop_assert!(shm.requires_coop_coep(), "SharedMemory MUST require COOP/COEP");
        prop_assert!(hyb.requires_coop_coep(), "Hybrid MUST require COOP/COEP");

        prop_assert_eq!(sep.worker_count(), worker_count);
        prop_assert_eq!(hyb.worker_count(), worker_count);

        log.info("Then SeparateModules and SharedMemory must differ on COOP/COEP (falsification)");
        // Falsification: SeparateModules and SharedMemory must have DIFFERENT flags.
        prop_assert_ne!(
            sep.requires_coop_coep(),
            shm.requires_coop_coep(),
            "SeparateModules and SharedMemory must differ on COOP/COEP"
        );
    }

    /// Bus arbitrary write/read roundtrip across all offsets and values.
    #[test]
    fn bus_roundtrip_arbitrary(
        size in 1usize..128,
        offset in 0usize..127,
        value in i32::MIN..i32::MAX,
    ) {
        let log = log();
        log.info(&format!("Given a bus of size={size}, writing value={value} at offset={offset}"));
        prop_assume!(offset < size);
        let mut bus = SharedMemoryBus::new(size);
        bus.write_i32(offset, value).unwrap();

        log.info("Then reading back must return exactly the value written");
        prop_assert_eq!(
            bus.read_i32(offset).unwrap(),
            value,
            "bus must return exactly the value that was written"
        );
    }

    /// Round-robin dispatch with arbitrary pool size and dispatch count.
    #[test]
    fn pool_round_robin_distributes_evenly(
        worker_count in 1usize..8,
        rounds in 1usize..4,
    ) {
        let log = log();
        log.info(&format!("Given a pool of {worker_count} workers dispatched {rounds} full rounds"));
        let mut pool = WorkerPool::new(worker_count, "w.js").unwrap();
        let dispatch_count = worker_count * rounds;
        let mut counts = std::collections::HashMap::<u32, usize>::new();
        for _ in 0..dispatch_count {
            let id = pool.dispatch_to_next().worker_id();
            *counts.entry(id).or_insert(0) += 1;
        }

        log.info("Then each worker must receive exactly 'rounds' dispatches");
        for (&id, &count) in &counts {
            prop_assert_eq!(
                count, rounds,
                "worker {} should receive exactly {} dispatches", id, rounds
            );
        }
    }
}
