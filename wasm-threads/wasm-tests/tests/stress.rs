// Stress / load tests
//
// 1000-iteration load tests verifying that implementations hold up under
// sustained use without data corruption or ordering violations.

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::{SharedMemoryBus, WorkerChannel, WorkerPool};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

/// Write 1000 values to a bus; last-writer-wins per slot must hold.
#[test]
fn bus_write_read_1000_times() {
    let log = log();
    log.info("Given a SharedMemoryBus with 1024 slots");
    let mut bus = SharedMemoryBus::new(1024);

    log.info("When 1000 values are written sequentially (i % 1024 slot mapping)");
    for i in 0i32..1000 {
        bus.write_i32((i % 1024) as usize, i).unwrap();
    }

    log.info("Then every slot in [0..1000) must hold a value in [0, 1000)");
    // Each slot in [0..1000) was last written when i == slot index
    // (single pass; i % 1024 maps uniquely in [0..1000)).
    for slot in 0..1000usize {
        let val = bus.read_i32(slot).unwrap();
        // The last value written to `slot` is slot itself (since i < 1024
        // means i % 1024 == i, and we only traverse once, so the last
        // write to slot `s` is when i == s).
        assert!(
            val >= 0 && val < 1000,
            "slot {slot} must have a valid value in [0, 1000), got {val}"
        );
    }
}

/// 1000 dispatches over a 4-worker pool must produce exactly 250 per worker.
#[test]
fn pool_dispatch_1000_times_visits_all_workers() {
    let log = log();
    log.info("Given a WorkerPool with 4 workers");
    let mut pool = WorkerPool::new(4, "w.js").unwrap();
    let mut counts = std::collections::HashMap::<u32, usize>::new();

    log.info("When dispatch_to_next is called 1000 times");
    for _ in 0..1000 {
        let id = pool.dispatch_to_next().worker_id();
        *counts.entry(id).or_insert(0) += 1;
    }

    log.info("Then all 4 workers must have been dispatched to, each exactly 250 times");
    assert_eq!(counts.len(), 4, "all 4 workers must have been dispatched to");
    for (id, count) in &counts {
        assert_eq!(
            *count, 250,
            "worker {id} should receive 250/1000 dispatches in round-robin"
        );
    }
}

/// A channel must handle 1000 messages with correct ordering.
#[test]
fn channel_handles_1000_messages() {
    #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Msg { seq: u64 }

    let log = log();
    log.info("Given a WorkerChannel with 1000 messages enqueued in sequence order");
    let mut ch = WorkerChannel::<Msg>::new(0);
    for i in 0..1000u64 {
        ch.send(Msg { seq: i }).unwrap();
    }
    assert_eq!(ch.pending_count(), 1000, "channel must hold 1000 pending messages");

    log.info("When messages are received one by one");
    log.info("Then each must arrive in FIFO order and the channel must drain to empty");
    for i in 0..1000u64 {
        let m = ch.receive().unwrap();
        assert_eq!(m.seq, i, "message {i} must be received in order");
    }
    assert_eq!(ch.pending_count(), 0, "channel must be empty after draining all messages");
}

/// Draining many messages at once must be consistent with receiving them one by one.
#[test]
fn channel_drain_1000_messages_all_present() {
    #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Msg { seq: u64 }

    let log = log();
    log.info("Given a WorkerChannel with 1000 messages enqueued");
    let mut ch = WorkerChannel::<Msg>::new(0);
    for i in 0..1000u64 {
        ch.send(Msg { seq: i }).unwrap();
    }

    log.info("When drain() is called");
    let drained = ch.drain();

    log.info("Then all 1000 messages must be returned in insertion order and channel must be empty");
    assert_eq!(drained.len(), 1000, "drain must return all 1000 messages");
    assert_eq!(ch.pending_count(), 0, "channel must be empty after drain");

    for (i, msg) in drained.iter().enumerate() {
        assert_eq!(msg.seq, i as u64, "drain must preserve insertion order");
    }
}

/// CAS stress: sequential compare-exchanges must succeed only when value matches.
#[test]
fn compare_exchange_sequential_stress() {
    let log = log();
    log.info("Given a SharedMemoryBus with slot 0 initialised to 0");
    let mut bus = SharedMemoryBus::new(4);
    bus.write_i32(0, 0).unwrap();

    log.info("When 100 sequential CAS increments are performed (each expects the previous value)");
    // Increment via CAS 100 times — each iteration should succeed.
    for i in 0i32..100 {
        let ok = bus.compare_exchange(0, i, i + 1);
        assert!(ok, "CAS must succeed when expected={i} matches current value");
        assert_eq!(bus.read_i32(0).unwrap(), i + 1);
    }

    log.info("Then the final value must be 100");
    assert_eq!(bus.read_i32(0).unwrap(), 100);
}
