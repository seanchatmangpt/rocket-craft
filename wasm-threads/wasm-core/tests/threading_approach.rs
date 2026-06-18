use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_core::ThreadingApproach;

fn make_logger() -> Logger {
    let mut logger = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    logger.add_sink(Box::new(sink));
    logger
}

#[test]
fn separate_modules_does_not_require_coop_coep() {
    let log = make_logger();
    log.info("Given ThreadingApproach::SeparateModules with worker_count 2");
    let a = ThreadingApproach::SeparateModules { worker_count: 2 };

    log.info("When we check requires_coop_coep()");
    log.info("Then it returns false — no shared memory headers needed");
    assert!(!a.requires_coop_coep());
}

#[test]
fn shared_memory_requires_coop_coep() {
    let log = make_logger();
    log.info("Given ThreadingApproach::SharedMemory with buffer_size_bytes 1024");
    let a = ThreadingApproach::SharedMemory {
        buffer_size_bytes: 1024,
    };

    log.info("When we check requires_coop_coep()");
    log.info("Then it returns true — SharedArrayBuffer requires COOP/COEP");
    assert!(a.requires_coop_coep());
}

#[test]
fn hybrid_requires_coop_coep() {
    let log = make_logger();
    log.info("Given ThreadingApproach::Hybrid with worker_count 4 and shared_buffer_size_bytes 4096");
    let a = ThreadingApproach::Hybrid {
        worker_count: 4,
        shared_buffer_size_bytes: 4096,
    };

    log.info("When we check requires_coop_coep()");
    log.info("Then it returns true — hybrid approach shares memory");
    assert!(a.requires_coop_coep());
}

#[test]
fn recommended_for_game_logic_is_separate_modules() {
    let log = make_logger();
    log.info("Given the recommended_for_game_logic() factory");

    log.info("When we call it");
    let a = ThreadingApproach::recommended_for_game_logic();

    log.info("Then it returns SeparateModules (no COOP/COEP required)");
    matches!(a, ThreadingApproach::SeparateModules { worker_count: 2 });
    assert!(!a.requires_coop_coep());
}

#[test]
fn shared_memory_worker_count_is_one() {
    let log = make_logger();
    log.info("Given ThreadingApproach::SharedMemory with buffer_size_bytes 512");
    let a = ThreadingApproach::SharedMemory {
        buffer_size_bytes: 512,
    };

    log.info("When we call worker_count()");
    log.info("Then it returns 1 — only one WASM module runs in SharedMemory mode");
    assert_eq!(a.worker_count(), 1);
}

proptest! {
    #[test]
    fn approach_worker_count_consistent(n in 1usize..64) {
        let log = make_logger();
        log.info(&format!("Given SeparateModules with worker_count={n}"));
        let a = ThreadingApproach::SeparateModules { worker_count: n };
        log.info("Then worker_count() returns n");
        prop_assert_eq!(a.worker_count(), n);
    }

    #[test]
    fn hybrid_worker_count_consistent(n in 1usize..64, buf in 1usize..65536) {
        let log = make_logger();
        log.info(&format!("Given Hybrid with worker_count={n} shared_buffer_size_bytes={buf}"));
        let a = ThreadingApproach::Hybrid {
            worker_count: n,
            shared_buffer_size_bytes: buf,
        };
        log.info("Then worker_count() returns n and requires_coop_coep() is true");
        prop_assert_eq!(a.worker_count(), n);
        prop_assert!(a.requires_coop_coep());
    }
}
