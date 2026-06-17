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
            js_worker: None,
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
            js_worker: None,
        }
    }

    /// Transition Paused → Terminated.
    pub fn terminate(self) -> WasmWorker<Terminated> {
        WasmWorker {
            script_url: self.script_url,
            worker_id: self.worker_id,
            _state: PhantomData,
            #[cfg(target_arch = "wasm32")]
            js_worker: None,
        }
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
