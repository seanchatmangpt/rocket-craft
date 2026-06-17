// Scenario 3 — SharedMemoryBus as game-state sync channel

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::SharedMemoryBus;

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

/// Happy path: game logic writes health/max/entity_count; UI reads back.
#[test]
fn shared_memory_bus_simulates_atomic_sync() {
    let log = log();
    log.info("Given a SharedMemoryBus with 64 slots simulating game state sync");
    let mut bus = SharedMemoryBus::new(64);

    log.info("When the game-logic worker writes health, max_health, and entity_count");
    bus.write_i32(0, 75).unwrap();  // health
    bus.write_i32(1, 100).unwrap(); // max_health
    bus.write_i32(2, 5).unwrap();   // entity_count

    log.info("Then the UI worker must read back the exact values written");
    let health = bus.read_i32(0).unwrap();
    let max = bus.read_i32(1).unwrap();
    let entities = bus.read_i32(2).unwrap();

    assert_eq!(health, 75);
    assert_eq!(max, 100);
    assert_eq!(entities, 5);

    // Health percentage (as a UI worker would compute it).
    let pct = health as f32 / max as f32;
    assert!(
        (pct - 0.75).abs() < 0.001,
        "health percentage from raw ints must be 0.75"
    );
}

/// Falsification: bus slots must be independent — writing to one must not
/// overwrite or alias another.
#[test]
fn bus_slots_are_independent() {
    let log = log();
    log.info("Given a SharedMemoryBus with 3 distinct values at slots 0, 1, and 2");
    let mut bus = SharedMemoryBus::new(8);
    bus.write_i32(0, 111).unwrap();
    bus.write_i32(1, 222).unwrap();
    bus.write_i32(2, 333).unwrap();

    log.info("When the slots are read back");
    log.info("Then each slot must hold its own value — no aliasing between slots");
    assert_ne!(
        bus.read_i32(0).unwrap(),
        bus.read_i32(1).unwrap(),
        "slot 0 and slot 1 must be independent"
    );
    assert_ne!(
        bus.read_i32(1).unwrap(),
        bus.read_i32(2).unwrap(),
        "slot 1 and slot 2 must be independent"
    );
    assert_ne!(
        bus.read_i32(0).unwrap(),
        bus.read_i32(2).unwrap(),
        "slot 0 and slot 2 must be independent"
    );
}

/// Falsification: compare_exchange must only swap when the expected value
/// matches the current value.
#[test]
fn compare_exchange_is_conditional() {
    let log = log();
    log.info("Given a SharedMemoryBus with slot 0 set to 42");
    let mut bus = SharedMemoryBus::new(4);
    bus.write_i32(0, 42).unwrap();

    log.info("When compare_exchange is attempted with wrong expected values");
    // Wrong expected values — must NOT swap.
    for wrong in [0, 1, 41, 43, 100, -1, i32::MAX] {
        let swapped = bus.compare_exchange(0, wrong, 999);
        assert!(
            !swapped,
            "compare_exchange must NOT swap when expected={wrong} != actual=42"
        );
        assert_eq!(
            bus.read_i32(0).unwrap(),
            42,
            "value must remain 42 after failed CAS (wrong expected={wrong})"
        );
    }

    log.info("Then compare_exchange with the correct expected value must succeed");
    // Correct expected — MUST swap.
    let swapped = bus.compare_exchange(0, 42, 999);
    assert!(swapped, "compare_exchange MUST swap when expected==actual");
    assert_eq!(bus.read_i32(0).unwrap(), 999, "value must be 999 after successful CAS");
}

/// Out-of-bounds access must return an error, not silently succeed.
#[test]
fn bus_bounds_checking_is_real() {
    let log = log();
    log.info("Given a SharedMemoryBus with 4 slots");
    let bus = SharedMemoryBus::new(4);

    log.info("When reads are attempted at various offsets");
    log.info("Then in-bounds reads must succeed and out-of-bounds reads must error");
    assert!(bus.read_i32(0).is_ok(), "offset 0 on size-4 bus must be in bounds");
    assert!(bus.read_i32(3).is_ok(), "offset 3 on size-4 bus must be in bounds");
    assert!(bus.read_i32(4).is_err(), "offset == size must be out of bounds");
    assert!(bus.read_i32(5).is_err(), "offset > size must be out of bounds");
    assert!(bus.read_i32(100).is_err(), "large offset must be out of bounds");
}
