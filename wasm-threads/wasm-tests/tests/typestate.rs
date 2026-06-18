// Scenario 5 — Typestate correctness: compile-time enforcement
//
// These tests prove that only valid state transitions compile. The commented
// lines show what would be a COMPILE ERROR, not a runtime failure.

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::{Paused, Running, Terminated, Uninitialized, WasmWorker};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn typestate_worker_lifecycle() {
    let log = log();
    log.info("Given a WasmWorker in Uninitialized state");
    let w: WasmWorker<Uninitialized> = WasmWorker::new("worker.js", 42);

    log.info("When the worker is started and then terminated");
    let r: WasmWorker<Running> = w.start().unwrap();
    let _t: WasmWorker<Terminated> = r.terminate();

    log.info("Then the typestate transitions compile — illegal transitions do not");
    // The following would NOT compile — proving typestate works:
    //
    //   let w2 = WasmWorker::<Uninitialized>::new("x", 0);
    //   w2.terminate();     // ERROR: no method `terminate` on WasmWorker<Uninitialized>
    //
    //   let t = WasmWorker::<Uninitialized>::new("x", 0).start().unwrap().terminate();
    //   t.pause();          // ERROR: no method `pause` on WasmWorker<Terminated>
}

#[test]
fn typestate_pause_resume_terminate() {
    let log = log();
    log.info("Given a WasmWorker in Running state");
    let running: WasmWorker<Running> =
        WasmWorker::new("worker.js", 0).start().unwrap();

    log.info("When it is paused, resumed, and then terminated");
    let paused: WasmWorker<Paused> = running.pause();
    let resumed: WasmWorker<Running> = paused.resume();
    let _term: WasmWorker<Terminated> = resumed.terminate();

    log.info("Then the full pause-resume-terminate lifecycle compiles correctly");
    // The following would NOT compile:
    //
    //   let p = WasmWorker::new("x", 0).start().unwrap().pause();
    //   p.start();  // ERROR: WasmWorker<Paused> has no `start` method — only `resume`
}

/// Falsification: worker_id must be preserved across state transitions.
#[test]
fn worker_id_survives_transitions() {
    let log = log();
    log.info("Given WasmWorkers with various IDs");

    for id in [0u32, 1, 255, u32::MAX / 2] {
        log.info(&format!("When worker id={id} goes through start → pause → resume"));
        let w = WasmWorker::new("x.js", id).start().unwrap();
        assert_eq!(w.worker_id(), id, "worker_id must equal {id} after start");
        let p = w.pause();
        let r = p.resume();

        log.info(&format!("Then worker_id must still be {id} after all transitions"));
        assert_eq!(r.worker_id(), id, "worker_id must equal {id} after pause+resume");
    }
}

/// Falsification: is_terminated must always be true for Terminated state.
#[test]
fn terminated_worker_always_reports_terminated() {
    let log = log();
    log.info("Given WasmWorkers with IDs 0, 7, and 99 that are started then terminated");

    for id in [0u32, 7, 99] {
        let t = WasmWorker::new("w.js", id)
            .start()
            .unwrap()
            .terminate();

        log.info(&format!("Then WasmWorker<Terminated> with id={id} must report is_terminated=true"));
        assert!(t.is_terminated(), "WasmWorker<Terminated> must always return true from is_terminated");
    }
}

/// Falsification: script_url must be preserved across transitions.
#[test]
fn script_url_survives_transitions() {
    let log = log();
    log.info("Given WasmWorkers constructed with various script URLs");
    let urls = ["worker.js", "https://cdn.example.com/worker.js", "relative/path.js"];

    for url in urls {
        log.info(&format!("When worker with url='{url}' goes through start → pause → resume"));
        let r = WasmWorker::new(url, 0).start().unwrap();
        assert_eq!(r.script_url(), url, "script_url must survive Uninitialized → Running");
        let p = r.pause();
        let r2 = p.resume();

        log.info(&format!("Then script_url must still be '{url}' after all transitions"));
        assert_eq!(r2.script_url(), url, "script_url must survive Running → Paused → Running");
    }
}
