// Anti-cheat / Falsification tests
//
// These 8 tests verify that the implementations compute real results
// and do not return constants or mocked values.

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::{SharedMemoryBus, ThreadingApproach, WorkerChannel, WorkerPool};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

/// Bus reads must reflect exactly what was written, not a constant.
#[test]
fn anti_cheat_bus_reads_reflect_writes_not_a_constant() {
    let log = log();
    log.info("Given a SharedMemoryBus with 16 slots");
    let mut bus = SharedMemoryBus::new(16);

    log.info("When 16 distinct values are written to 16 distinct offsets");
    let test_values = [0i32, 1, -1, i32::MAX, i32::MIN, 42, 100, 999,
                       0, -999, 12345, -12345, 7, 77, 777, 7777];
    for (i, &v) in test_values.iter().enumerate() {
        bus.write_i32(i, v).unwrap();
    }

    log.info("Then each read returns exactly what was written — not a constant");
    for (i, &v) in test_values.iter().enumerate() {
        assert_eq!(
            bus.read_i32(i).unwrap(), v,
            "bus.read_i32({i}) must return {v}, not a constant"
        );
    }
}

/// pool.size() must vary with the requested count, not return a constant.
#[test]
fn anti_cheat_pool_size_varies_with_request() {
    let log = log();
    log.info("Given pools created with sizes 1, 2, 4, 8, and 16");

    log.info("When pool.size() is called on each pool");
    for n in [1, 2, 4, 8, 16] {
        let pool = WorkerPool::new(n, "w.js").unwrap();

        log.info(&format!("Then pool.size() must return {n}, not a constant"));
        assert_eq!(
            pool.size(), n,
            "pool.size() must return {n}, not a constant"
        );
    }
}

/// Channel.receive() must return the exact message that was sent.
#[test]
fn anti_cheat_channel_delivers_what_was_sent() {
    #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Probe { id: u64, payload: String }

    let log = log();
    log.info("Given a WorkerChannel with 3 distinct probe messages queued");
    let probes = vec![
        Probe { id: 0, payload: "alpha".to_string() },
        Probe { id: 1, payload: "beta".to_string() },
        Probe { id: 999, payload: "gamma".to_string() },
    ];

    let mut ch = WorkerChannel::<Probe>::new(0);
    for p in &probes { ch.send(p.clone()).unwrap(); }

    log.info("When messages are received one by one");
    log.info("Then each received message must match exactly what was sent");
    for p in &probes {
        let received = ch.receive().expect("channel must deliver sent message");
        assert_eq!(
            received, *p,
            "delivered message must match sent message (id={})", p.id
        );
    }
}

/// compare_exchange must only swap on exact match; never on any mismatch.
#[test]
fn anti_cheat_compare_exchange_conditional() {
    let log = log();
    log.info("Given a SharedMemoryBus with slot 0 set to 42");
    let mut bus = SharedMemoryBus::new(4);
    bus.write_i32(0, 42).unwrap();

    log.info("When compare_exchange is called with wrong expected values");
    for wrong in [0, 1, 41, 43, 100, -1, i32::MAX] {
        let swapped = bus.compare_exchange(0, wrong, 999);
        assert!(
            !swapped,
            "compare_exchange must NOT swap when expected={wrong} != actual=42"
        );
        assert_eq!(
            bus.read_i32(0).unwrap(), 42,
            "value must remain 42 after failed CAS (expected={wrong})"
        );
    }

    log.info("Then compare_exchange with the correct expected value must swap");
    let swapped = bus.compare_exchange(0, 42, 999);
    assert!(swapped, "compare_exchange MUST swap when expected matches");
    assert_eq!(bus.read_i32(0).unwrap(), 999);
}

/// Round-robin must visit ALL workers in a pool, not repeat the same one.
#[test]
fn anti_cheat_pool_round_robin_is_real() {
    let log = log();
    log.info("Given a pool of 4 workers");
    let mut pool = WorkerPool::new(4, "w.js").unwrap();
    let mut seen_ids = std::collections::HashSet::new();

    log.info("When dispatch_to_next is called once per worker");
    for _ in 0..pool.size() {
        seen_ids.insert(pool.dispatch_to_next().worker_id());
    }

    log.info("Then all 4 workers must have been visited — round-robin must be real");
    assert_eq!(
        seen_ids.len(), 4,
        "round-robin must visit all 4 workers, not repeat the same one"
    );
}

/// SeparateModules and SharedMemory must have DIFFERENT COOP/COEP flags.
#[test]
fn anti_cheat_approach_coop_coep_differs() {
    let log = log();
    log.info("Given SeparateModules and SharedMemory threading approaches");
    let sep = ThreadingApproach::SeparateModules { worker_count: 2 };
    let shm = ThreadingApproach::SharedMemory { buffer_size_bytes: 1024 };

    log.info("When COOP/COEP requirements are compared");
    log.info("Then they must differ — not both return true or both return false");
    assert_ne!(
        sep.requires_coop_coep(),
        shm.requires_coop_coep(),
        "SeparateModules and SharedMemory must have DIFFERENT COOP/COEP requirements"
    );
}

/// All worker IDs within a pool must be unique.
#[test]
fn anti_cheat_pool_worker_ids_are_unique() {
    let log = log();
    log.info("Given a pool of 8 workers");
    let pool = WorkerPool::new(8, "w.js").unwrap();

    log.info("When all worker IDs are collected");
    let ids = pool.worker_ids();
    let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();

    log.info("Then all 8 worker IDs must be unique — no ID reuse allowed");
    assert_eq!(
        unique.len(), ids.len(),
        "all worker IDs in pool must be unique"
    );
}

/// Out-of-bounds access must error — bounds check must be real.
#[test]
fn anti_cheat_bus_bounds_checking_is_real() {
    let log = log();
    log.info("Given a SharedMemoryBus with 4 slots");
    let bus = SharedMemoryBus::new(4);

    log.info("When reads are attempted at both valid and invalid offsets");
    log.info("Then out-of-bounds reads must error and in-bounds reads must succeed");
    assert!(bus.read_i32(4).is_err(), "offset == size must error");
    assert!(bus.read_i32(5).is_err(), "offset > size must error");
    assert!(bus.read_i32(100).is_err(), "large offset must error");
    assert!(bus.read_i32(0).is_ok(), "offset 0 on size-4 bus must succeed");
    assert!(bus.read_i32(3).is_ok(), "offset 3 on size-4 bus must succeed");
}

/// Bus size() must reflect what was requested, not be a constant.
#[test]
fn anti_cheat_bus_size_matches_requested() {
    let log = log();
    log.info("Given SharedMemoryBuses created with sizes 1, 4, 16, 64, 256, and 1024");

    log.info("When bus.size() is called on each");
    for size in [1, 4, 16, 64, 256, 1024] {
        let bus = SharedMemoryBus::new(size);

        log.info(&format!("Then bus.size() must return {size}"));
        assert_eq!(bus.size(), size, "SharedMemoryBus::size() must return {size}");
    }
}
