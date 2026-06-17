pub mod actor;
pub mod cqrs;
pub mod event_sourcing;
pub mod observer;
pub mod pipeline;

pub use actor::{ActorError, ActorMailbox, ActorMessage, ActorSystem};
pub use cqrs::{Command, CommandBus, CommandHandler, Query, QueryBus, QueryHandler, ReadModel};
pub use event_sourcing::{Aggregate, DomainEvent, EventLog, EventSourcedRepo, Snapshot};
pub use observer::{EventBus, EventType};
pub use pipeline::{BatchPipeline, Pipeline, Stage};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASM-exposed wrapper for the Actor pattern.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmActorSystem {
    inner: actor::ActorSystem,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmActorSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(worker_count: usize) -> Self {
        Self {
            inner: actor::ActorSystem::new(worker_count),
        }
    }

    pub fn spawn_actor_id(&mut self) -> u32 {
        self.inner.spawn_actor_id()
    }

    pub fn worker_count(&self) -> usize {
        self.inner.worker_count()
    }

    pub fn assign_worker(&self, actor_id: u32) -> usize {
        self.inner.assign_worker(actor_id)
    }
}

/// WASM-exposed wrapper for the CQRS CommandBus pattern.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmCommandBus {
    dispatched: u64,
    succeeded: u64,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmCommandBus {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { dispatched: 0, succeeded: 0 }
    }

    pub fn dispatch_command(&mut self, _command_json: &str) -> bool {
        self.dispatched += 1;
        self.succeeded += 1;
        true
    }

    pub fn commands_dispatched(&self) -> u64 {
        self.dispatched
    }

    pub fn success_rate(&self) -> f64 {
        if self.dispatched == 0 { 0.0 } else { self.succeeded as f64 / self.dispatched as f64 }
    }
}

/// WASM-exposed wrapper for the Observer EventBus pattern.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEventBus {
    published: u64,
    subscriber_count: u32,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEventBus {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { published: 0, subscriber_count: 0 }
    }

    pub fn subscribe(&mut self) -> u32 {
        let id = self.subscriber_count;
        self.subscriber_count += 1;
        id
    }

    pub fn publish(&mut self, _event_json: &str) {
        self.published += 1;
    }

    pub fn published_count(&self) -> u64 {
        self.published
    }

    pub fn subscriber_count(&self) -> u32 {
        self.subscriber_count
    }
}
