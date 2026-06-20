use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::{Uninitialized, WasmWorker};

fn make_logger() -> Logger {
    let mut logger = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    logger.add_sink(Box::new(sink));
    logger
}

#[test]
fn worker_starts_uninitialized_then_runs() {
    let log = make_logger();
    log.info("Given a WasmWorker<Uninitialized> with script url 'worker.js' and id 0");
    let w = WasmWorker::<Uninitialized>::new("worker.js", 0);

    log.info("When we call start()");
    let running = w.start().expect("should start");

    log.info("Then the worker id is 0 and we can terminate it");
    assert_eq!(running.worker_id(), 0);
    let _terminated = running.terminate();
}

#[test]
fn worker_pause_resume_cycle() {
    let log = make_logger();
    log.info("Given a running WasmWorker with id 1");
    let w = WasmWorker::<Uninitialized>::new("worker.js", 1)
        .start()
        .unwrap();

    log.info("When we pause and then resume the worker");
    let paused = w.pause();
    let running = paused.resume();

    log.info("Then the worker can be terminated without error");
    let _completed = running.terminate();
}

#[test]
fn worker_script_url_preserved() {
    let log = make_logger();
    let url = "https://example.com/worker.js";
    log.info(&format!("Given a WasmWorker with script url '{url}'"));
    let w = WasmWorker::<Uninitialized>::new(url, 7)
        .start()
        .unwrap();

    log.info("When we inspect the running worker");
    log.info("Then script_url() returns the original url");
    assert_eq!(w.script_url(), url);
}

#[test]
fn terminated_worker_reports_terminated() {
    let log = make_logger();
    log.info("Given a WasmWorker that has been started and terminated");
    let t = WasmWorker::<Uninitialized>::new("w.js", 0)
        .start()
        .unwrap()
        .terminate();

    log.info("When we check is_terminated()");
    log.info("Then it returns true");
    assert!(t.is_terminated());
}

#[test]
fn worker_start_transitions_to_running() {
    use wasm_core::worker::{WasmWorker};
    let w = WasmWorker::new("test.js", 1);
    let running = w.start().expect("start should succeed on native");
    assert_eq!(running.worker_id(), 1);
    // Falsification: worker_id is 1 not 0
    assert_ne!(running.worker_id(), 0);
}

#[test]
fn worker_terminate_from_running() {
    use wasm_core::worker::{WasmWorker};
    let w = WasmWorker::new("test.js", 5);
    let running = w.start().unwrap();
    let terminated = running.terminate();
    assert!(terminated.is_terminated());
}

#[test]
fn different_worker_ids_are_different() {
    use wasm_core::worker::{WasmWorker};
    let w1 = WasmWorker::new("a.js", 1);
    let w2 = WasmWorker::new("b.js", 2);
    let r1 = w1.start().unwrap();
    let r2 = w2.start().unwrap();
    // Falsification: worker IDs differ
    assert_ne!(r1.worker_id(), r2.worker_id());
}
