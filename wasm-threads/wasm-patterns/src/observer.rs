use std::collections::HashMap;

pub trait EventType: Clone + std::fmt::Debug + 'static {
    fn topic(&self) -> &'static str;
}

type BoxedCallback<E> = Box<dyn Fn(&E) + 'static>;

pub struct EventBus<E: EventType> {
    subscribers: HashMap<&'static str, Vec<BoxedCallback<E>>>,
    published_count: u64,
    subscriber_count: u64,
}

impl<E: EventType> EventBus<E> {
    pub fn new() -> Self { Self { subscribers: HashMap::new(), published_count: 0, subscriber_count: 0 } }

    pub fn subscribe<F: Fn(&E) + 'static>(&mut self, topic: &'static str, callback: F) {
        self.subscribers.entry(topic).or_default().push(Box::new(callback));
        self.subscriber_count += 1;
    }

    pub fn publish(&mut self, event: &E) {
        self.published_count += 1;
        let topic = event.topic();
        if let Some(subs) = self.subscribers.get(topic) {
            for cb in subs { cb(event); }
        }
        // wildcard "*" subscribers receive everything
        if let Some(subs) = self.subscribers.get("*") {
            for cb in subs { cb(event); }
        }
    }

    pub fn published_count(&self) -> u64 { self.published_count }
    pub fn subscriber_count(&self) -> u64 { self.subscriber_count }
    pub fn topic_subscriber_count(&self, topic: &'static str) -> usize {
        self.subscribers.get(topic).map(|v| v.len()).unwrap_or(0)
    }
    pub fn has_subscribers(&self, topic: &'static str) -> bool {
        self.subscribers.get(topic).map(|v| !v.is_empty()).unwrap_or(false)
    }
}

impl<E: EventType> Default for EventBus<E> {
    fn default() -> Self { Self::new() }
}
