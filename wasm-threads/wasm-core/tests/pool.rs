use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::WorkerPool;

fn make_logger() -> Logger {
    let mut logger = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    logger.add_sink(Box::new(sink));
    logger
}

#[test]
fn pool_size_matches_requested() {
    let log = make_logger();
    log.info("Given a WorkerPool constructed with 4 workers and 'worker.js'");
    let pool = WorkerPool::new(4, "worker.js").unwrap();

    log.info("When we inspect size() and worker_ids()");
    log.info("Then size() is 4 and worker_ids() has 4 entries");
    assert_eq!(pool.size(), 4);
    assert_eq!(pool.worker_ids().len(), 4);
}

#[test]
fn pool_round_robin_cycles() {
    let log = make_logger();
    log.info("Given a WorkerPool with 3 workers");
    let mut pool = WorkerPool::new(3, "worker.js").unwrap();

    log.info("When we dispatch_to_next() four times");
    let id0 = pool.dispatch_to_next().worker_id();
    let id1 = pool.dispatch_to_next().worker_id();
    let id2 = pool.dispatch_to_next().worker_id();
    let id3 = pool.dispatch_to_next().worker_id();

    log.info("Then ids 0,1,2 are all distinct and the 4th dispatch wraps back to id0");
    assert_ne!(id0, id1);
    assert_ne!(id1, id2);
    assert_eq!(id0, id3);
}

#[test]
fn pool_terminate_all_returns_terminated_workers() {
    let log = make_logger();
    log.info("Given a WorkerPool with 2 workers");
    let pool = WorkerPool::new(2, "worker.js").unwrap();

    log.info("When we call terminate_all()");
    let terminated = pool.terminate_all();

    log.info("Then 2 workers are returned and each reports is_terminated() == true");
    assert_eq!(terminated.len(), 2);
    for t in &terminated {
        assert!(t.is_terminated());
    }
}
