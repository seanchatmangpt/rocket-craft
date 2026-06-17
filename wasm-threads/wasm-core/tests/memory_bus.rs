use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_core::SharedMemoryBus;

fn make_logger() -> Logger {
    let mut logger = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    logger.add_sink(Box::new(sink));
    logger
}

#[test]
fn bus_write_read_roundtrip() {
    let mut log = make_logger();
    log.info("Given a SharedMemoryBus of size 16");
    let mut bus = SharedMemoryBus::new(16);

    log.info("When we write 42 at offset 0");
    bus.write_i32(0, 42).unwrap();

    log.info("Then reading offset 0 returns 42");
    assert_eq!(bus.read_i32(0).unwrap(), 42);
}

#[test]
fn bus_out_of_bounds_read_errors() {
    let mut log = make_logger();
    log.info("Given a SharedMemoryBus of size 4");
    let bus = SharedMemoryBus::new(4);

    log.info("When we read at offset 10 (beyond size)");
    log.info("Then read_i32 returns an error");
    assert!(bus.read_i32(10).is_err());
}

#[test]
fn bus_out_of_bounds_write_errors() {
    let mut log = make_logger();
    log.info("Given a SharedMemoryBus of size 4");
    let mut bus = SharedMemoryBus::new(4);

    log.info("When we write at offset 10 (beyond size)");
    log.info("Then write_i32 returns an error");
    assert!(bus.write_i32(10, 99).is_err());
}

#[test]
fn bus_different_values_at_different_offsets() {
    let mut log = make_logger();
    log.info("Given a SharedMemoryBus of size 8");
    let mut bus = SharedMemoryBus::new(8);

    log.info("When we write 100 at offset 0 and 200 at offset 1");
    bus.write_i32(0, 100).unwrap();
    bus.write_i32(1, 200).unwrap();

    log.info("Then reading offset 0 and offset 1 yields distinct values");
    assert_ne!(bus.read_i32(0).unwrap(), bus.read_i32(1).unwrap());
}

#[test]
fn bus_size_matches_requested() {
    let mut log = make_logger();
    log.info("Given a SharedMemoryBus constructed with size 32");

    log.info("When we inspect its size()");
    let bus = SharedMemoryBus::new(32);

    log.info("Then size() returns 32");
    assert_eq!(bus.size(), 32);
}

proptest! {
    #[test]
    fn bus_roundtrip_arbitrary_values(offset in 0usize..8, value in i32::MIN..i32::MAX) {
        let mut log = make_logger();
        log.info(&format!("Given offset={offset} value={value}"));
        let mut bus = SharedMemoryBus::new(16);
        bus.write_i32(offset, value).unwrap();
        log.info("Then read must return what was written");
        prop_assert_eq!(bus.read_i32(offset).unwrap(), value);
    }

    #[test]
    fn compare_exchange_only_swaps_when_expected_matches(
        initial in 0i32..100,
        wrong   in 101i32..200,
        new_val in 201i32..300,
    ) {
        let mut log = make_logger();
        log.info(&format!("Given initial={initial} wrong={wrong} new_val={new_val}"));
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, initial).unwrap();

        log.info("When compare_exchange is called with wrong expected value");
        let swapped = bus.compare_exchange(0, wrong, new_val);
        prop_assert!(!swapped);
        prop_assert_eq!(bus.read_i32(0).unwrap(), initial);

        log.info("When compare_exchange is called with correct expected value");
        let swapped = bus.compare_exchange(0, initial, new_val);
        prop_assert!(swapped);
        prop_assert_eq!(bus.read_i32(0).unwrap(), new_val);
    }
}
