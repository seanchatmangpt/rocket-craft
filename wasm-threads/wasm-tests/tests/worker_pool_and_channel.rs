// Scenario 2 — Worker pool + channel integration

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::{ThreadingApproach, WorkerChannel, WorkerPool};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn threading_approach_config_drives_pool_size() {
    let log = log();
    log.info("Given a SeparateModules approach with 4 workers");
    let approach = ThreadingApproach::SeparateModules { worker_count: 4 };

    log.info("When a pool is created from the approach's worker_count");
    let pool = WorkerPool::new(approach.worker_count(), "game_logic.js").unwrap();

    log.info("Then pool size must equal 4 and COOP/COEP must not be required");
    assert_eq!(pool.size(), 4);
    assert!(!approach.requires_coop_coep());
}

#[test]
fn shared_memory_approach_requires_coop_coep() {
    let log = log();
    log.info("Given a SharedMemory threading approach");
    let approach = ThreadingApproach::SharedMemory { buffer_size_bytes: 65536 };

    log.info("When checking COOP/COEP requirements");
    log.info("Then SharedMemory must require COOP/COEP");
    assert!(approach.requires_coop_coep());
}

/// End-to-end: 3 channels, each carries a distinct message.
#[test]
fn worker_pool_and_channel_end_to_end() {
    #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct GameMsg {
        tick: u64,
        data: String,
    }

    let log = log();
    log.info("Given a pool with 3 workers and a channel per worker");
    let pool = WorkerPool::new(3, "worker.js").unwrap();
    let mut channels: Vec<WorkerChannel<GameMsg>> = pool
        .worker_ids()
        .iter()
        .map(|&id| WorkerChannel::new(id))
        .collect();

    log.info("When distinct messages are sent to each channel");
    channels[0]
        .send(GameMsg { tick: 1, data: "move".to_string() })
        .unwrap();
    channels[1]
        .send(GameMsg { tick: 2, data: "attack".to_string() })
        .unwrap();
    channels[2]
        .send(GameMsg { tick: 3, data: "heal".to_string() })
        .unwrap();

    assert_eq!(channels[0].pending_count(), 1);
    assert_eq!(channels[1].pending_count(), 1);
    assert_eq!(channels[2].pending_count(), 1);

    let m0 = channels[0].receive().unwrap();
    let m1 = channels[1].receive().unwrap();
    let m2 = channels[2].receive().unwrap();

    log.info("Then distinct channels must carry distinct messages (falsification)");
    assert_ne!(
        m0.tick, m1.tick,
        "different channels must carry different tick values"
    );
    assert_ne!(
        m1.tick, m2.tick,
        "different channels must carry different tick values"
    );
    assert_ne!(
        m0.data, m1.data,
        "different channels must carry different data payloads"
    );
}

/// Falsification: pool must have unique worker IDs.
#[test]
fn pool_worker_ids_are_unique() {
    let log = log();
    log.info("Given a pool of 6 workers");
    let pool = WorkerPool::new(6, "worker.js").unwrap();

    log.info("When the worker IDs are collected into a set");
    let ids = pool.worker_ids();
    let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();

    log.info("Then all worker IDs must be unique — implementation must not reuse IDs");
    assert_eq!(
        unique.len(),
        ids.len(),
        "all worker IDs must be unique — implementation must not reuse IDs"
    );
}

/// Falsification: pool.size() must match the requested count for several values.
#[test]
fn pool_size_reflects_requested_count() {
    let log = log();
    log.info("Given pools created with sizes 1, 2, 4, and 8");

    log.info("When pool.size() is called on each");
    for n in [1, 2, 4, 8] {
        let pool = WorkerPool::new(n, "w.js").unwrap();

        log.info(&format!("Then pool.size() must return {n}, not a constant"));
        assert_eq!(
            pool.size(), n,
            "pool.size() must return {n}, not a constant"
        );
    }
}

/// Round-robin dispatch must cycle back to the first worker.
#[test]
fn pool_round_robin_returns_to_start() {
    let log = log();
    log.info("Given a pool with 3 workers");
    let mut pool = WorkerPool::new(3, "worker.js").unwrap();

    log.info("When 4 consecutive dispatches are made");
    let id0 = pool.dispatch_to_next().worker_id();
    let id1 = pool.dispatch_to_next().worker_id();
    let id2 = pool.dispatch_to_next().worker_id();
    let id3 = pool.dispatch_to_next().worker_id(); // must wrap around

    log.info("Then the 4th dispatch must wrap back to the first worker");
    assert_ne!(id0, id1, "consecutive dispatches must differ");
    assert_ne!(id1, id2, "consecutive dispatches must differ");
    assert_eq!(id0, id3, "round-robin must cycle back to first worker");
}
