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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_new_creates_correct_size() {
        let pool = WorkerPool::new(3, "worker.js").unwrap();
        assert_eq!(pool.size(), 3);
    }

    #[test]
    fn pool_script_url_matches_input() {
        let pool = WorkerPool::new(2, "game-worker.js").unwrap();
        assert_eq!(pool.script_url(), "game-worker.js");
    }

    #[test]
    fn worker_ids_are_sequential() {
        let pool = WorkerPool::new(3, "w.js").unwrap();
        let ids = pool.worker_ids();
        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn dispatch_round_robins() {
        let mut pool = WorkerPool::new(3, "w.js").unwrap();
        let id0 = pool.dispatch_to_next().worker_id();
        let id1 = pool.dispatch_to_next().worker_id();
        let id2 = pool.dispatch_to_next().worker_id();
        let id3 = pool.dispatch_to_next().worker_id(); // wraps back to 0
        assert_eq!([id0, id1, id2, id3], [0, 1, 2, 0]);
    }

    #[test]
    fn terminate_all_returns_correct_count() {
        let pool = WorkerPool::new(4, "w.js").unwrap();
        let terminated = pool.terminate_all();
        assert_eq!(terminated.len(), 4);
    }
}
