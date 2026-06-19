use std::marker::PhantomData;

// Zero-sized typestate markers — consistent with repo's Machine<Law, Phase> pattern.
pub struct Uninitialized;
pub struct Running;
pub struct Paused;
pub struct Terminated;

/// A WASM Web Worker abstraction using the PhantomData typestate pattern.
/// Illegal state transitions are rejected at compile time because the
/// required `impl` block simply does not exist for that state.
pub struct WasmWorker<S> {
    script_url: String,
    worker_id: u32,
    _state: PhantomData<S>,
    #[cfg(target_arch = "wasm32")]
    js_worker: Option<web_sys::Worker>,
}

impl WasmWorker<Uninitialized> {
    pub fn new(script_url: impl Into<String>, worker_id: u32) -> Self {
        WasmWorker {
            script_url: script_url.into(),
            worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: None,
        }
    }

    /// Transition from Uninitialized → Running.
    /// On wasm32 this spawns the actual Web Worker via web_sys::Worker::new().
    pub fn start(self) -> Result<WasmWorker<Running>, WorkerError> {
        #[cfg(target_arch = "wasm32")]
        {
            let js_worker = web_sys::Worker::new(&self.script_url)
                .map_err(|e| WorkerError::CreationFailed(format!("{:?}", e)))?;
            return Ok(WasmWorker {
                script_url: self.script_url,
                worker_id: self.worker_id,
                _state: PhantomData,
                js_worker: Some(js_worker),
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        Ok(WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
        })
    }
}

impl WasmWorker<Running> {
    /// Transition Running → Paused.
    pub fn pause(self) -> WasmWorker<Paused> {
        WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: self.js_worker,
        }
    }

    /// Transition Running → Terminated.
    pub fn terminate(self) -> WasmWorker<Terminated> {
        #[cfg(target_arch = "wasm32")]
        if let Some(ref w) = self.js_worker {
            w.terminate();
        }
        WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: None,
        }
    }

    pub fn worker_id(&self) -> u32 {
        self.worker_id
    }

    pub fn script_url(&self) -> &str {
        &self.script_url
    }

    /// Return a reference to the underlying JS Worker (wasm32 only).
    #[cfg(target_arch = "wasm32")]
    pub fn js_worker(&self) -> Option<&web_sys::Worker> {
        self.js_worker.as_ref()
    }
}

impl WasmWorker<Paused> {
    /// Transition Paused → Running.
    pub fn resume(self) -> WasmWorker<Running> {
        WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: self.js_worker,
        }
    }

    /// Transition Paused → Terminated.
    pub fn terminate(self) -> WasmWorker<Terminated> {
        #[cfg(target_arch = "wasm32")]
        if let Some(ref w) = self.js_worker {
            w.terminate();
        }
        WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: None,
        }
    }

    /// Return a reference to the underlying JS Worker (wasm32 only).
    #[cfg(target_arch = "wasm32")]
    pub fn js_worker(&self) -> Option<&web_sys::Worker> {
        self.js_worker.as_ref()
    }
}

impl WasmWorker<Terminated> {
    pub fn is_terminated(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// WorkerError
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum WorkerError {
    CreationFailed(String),
    SendFailed(String),
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerError::CreationFailed(msg) => write!(f, "Failed to create worker: {}", msg),
            WorkerError::SendFailed(msg) => write!(f, "Message send failed: {}", msg),
        }
    }
}

impl std::error::Error for WorkerError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ── WasmWorker typestate transitions (native target) ──────────────────────

    #[test]
    fn new_creates_uninitialized_worker() {
        let w = WasmWorker::<Uninitialized>::new("worker.js", 1);
        assert_eq!(w.script_url, "worker.js");
        assert_eq!(w.worker_id, 1);
    }

    #[test]
    fn start_transitions_to_running() {
        let w = WasmWorker::<Uninitialized>::new("worker.js", 7)
            .start()
            .expect("start must succeed on native target");
        assert_eq!(w.worker_id(), 7);
        assert_eq!(w.script_url(), "worker.js");
    }

    #[test]
    fn running_pause_resume_roundtrip() {
        let running = WasmWorker::<Uninitialized>::new("w.js", 2).start().unwrap();
        let paused = running.pause();
        let resumed = paused.resume();
        assert_eq!(resumed.worker_id(), 2);
    }

    #[test]
    fn running_terminate_is_terminated() {
        let running = WasmWorker::<Uninitialized>::new("w.js", 3).start().unwrap();
        let terminated = running.terminate();
        assert!(terminated.is_terminated());
    }

    #[test]
    fn paused_terminate_is_terminated() {
        let running = WasmWorker::<Uninitialized>::new("w.js", 4).start().unwrap();
        let terminated = running.pause().terminate();
        assert!(terminated.is_terminated());
    }

    // ── WorkerError display ───────────────────────────────────────────────────

    #[test]
    fn worker_error_creation_failed_display() {
        let e = WorkerError::CreationFailed("no DOM".into());
        assert!(e.to_string().contains("Failed to create worker"));
        assert!(e.to_string().contains("no DOM"));
    }

    #[test]
    fn worker_error_send_failed_display() {
        let e = WorkerError::SendFailed("channel closed".into());
        assert!(e.to_string().contains("send failed"));
    }
}
