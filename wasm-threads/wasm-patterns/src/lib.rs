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
    inner: cqrs::CommandBus,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmCommandBus {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: cqrs::CommandBus::new() }
    }

    pub fn dispatch_command(&mut self, command_json: &str) -> bool {
        struct RealHandler;
        impl cqrs::CommandHandler<cqrs::StringCommand> for RealHandler {
            fn handle(&mut self, _cmd: cqrs::StringCommand) -> Result<(), &'static str> {
                Ok(())
            }
        }
        let mut handler = RealHandler;
        let cmd = cqrs::StringCommand(command_json.to_string());
        self.inner.dispatch(&mut handler, cmd).is_ok()
    }

    pub fn commands_dispatched(&self) -> u64 {
        self.inner.commands_dispatched()
    }

    pub fn success_rate(&self) -> f64 {
        self.inner.success_rate()
    }
}

/// WASM-exposed wrapper for the Observer EventBus pattern.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEventBus {
    inner: observer::EventBus<observer::StringEvent>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEventBus {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: observer::EventBus::new() }
    }

    pub fn subscribe(&mut self, topic: &str) {
        // Leaking the string to get a 'static str for the topic in this simple example
        let static_topic: &'static str = Box::leak(topic.to_string().into_boxed_str());
        self.inner.subscribe(static_topic, |_| {
            // In a real impl, we might call back into JS
        });
    }

    pub fn publish(&mut self, topic: &str, data: &str) {
        let static_topic: &'static str = Box::leak(topic.to_string().into_boxed_str());
        let event = observer::StringEvent {
            topic: static_topic,
            data: data.to_string(),
        };
        self.inner.publish(&event);
    }

    pub fn published_count(&self) -> u64 {
        self.inner.published_count()
    }

    pub fn subscriber_count(&self) -> u32 {
        self.inner.subscriber_count() as u32
    }
}
