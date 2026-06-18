use crate::worker::{Running, Terminated, Uninitialized, WasmWorker, WorkerError};

/// A fixed-size pool of running WASM workers with round-robin dispatch.
pub struct WorkerPool {
    workers: Vec<WasmWorker<Running>>,
    next_worker: usize,
    script_url: String,
}

impl WorkerPool {
    /// Spawn `size` workers all running the same `script_url`.
    pub fn new(size: usize, script_url: impl Into<String>) -> Result<Self, WorkerError> {
        let url: String = script_url.into();
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let w = WasmWorker::<Uninitialized>::new(url.clone(), id as u32)
                .start()
                .map_err(|e| WorkerError::CreationFailed(format!("worker {id}: {e}")))?;
            workers.push(w);
        }
        Ok(WorkerPool {
            workers,
            next_worker: 0,
            script_url: url,
        })
    }

    /// Return a reference to the next worker in the round-robin sequence
    /// and advance the internal cursor.
    pub fn dispatch_to_next(&mut self) -> &WasmWorker<Running> {
        let idx = self.next_worker;
        self.next_worker = (self.next_worker + 1) % self.workers.len();
        &self.workers[idx]
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }

    pub fn worker_ids(&self) -> Vec<u32> {
        self.workers.iter().map(|w| w.worker_id()).collect()
    }

    pub fn script_url(&self) -> &str {
        &self.script_url
    }

    /// Terminate every worker in the pool and return them in their
    /// `Terminated` state so callers can confirm shutdown.
    pub fn terminate_all(self) -> Vec<WasmWorker<Terminated>> {
        self.workers.into_iter().map(|w| w.terminate()).collect()
    }
}
