use serde::{de::DeserializeOwned, Serialize};
use std::collections::VecDeque;

/// Typed message channel that simulates `postMessage` between workers.
///
/// On the native target the channel is a simple `VecDeque` queue so that
/// logic can be unit-tested without a browser. On WASM the same API would
/// be backed by `web_sys::Worker::post_message` / the `message` event
/// listener, but that path requires an actual Worker context.
pub struct WorkerChannel<M> {
    queue: VecDeque<M>,
    worker_id: u32,
}

impl<M: Serialize + DeserializeOwned + Clone> WorkerChannel<M> {
    pub fn new(worker_id: u32) -> Self {
        WorkerChannel {
            queue: VecDeque::new(),
            worker_id,
        }
    }

    /// Enqueue a message for delivery to the associated worker.
    pub fn send(&mut self, msg: M) -> Result<(), ChannelError> {
        // Validate round-trip serialisation so that the same code path
        // would work when backed by postMessage (which JSON-serialises).
        let _ = serde_json::to_string(&msg).map_err(|e| {
            ChannelError::SerializationFailed(e.to_string())
        })?;
        self.queue.push_back(msg);
        Ok(())
    }

    /// Dequeue the oldest pending message, or `None` if the queue is empty.
    pub fn receive(&mut self) -> Option<M> {
        self.queue.pop_front()
    }

    /// Number of messages waiting to be consumed.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Drain all pending messages into a `Vec`, oldest first.
    pub fn drain(&mut self) -> Vec<M> {
        self.queue.drain(..).collect()
    }

    pub fn worker_id(&self) -> u32 {
        self.worker_id
    }
}

// ---------------------------------------------------------------------------
// ChannelError
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ChannelError {
    SerializationFailed(String),
    WorkerNotFound(u32),
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelError::SerializationFailed(msg) => {
                write!(f, "serialization failed: {}", msg)
            }
            ChannelError::WorkerNotFound(id) => {
                write!(f, "worker {} not found", id)
            }
        }
    }
}

impl std::error::Error for ChannelError {}
