use std::collections::HashMap;

pub trait EventType: Clone + std::fmt::Debug + 'static {
    fn topic(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct StringEvent {
    pub topic: &'static str,
    pub data: String,
}

impl EventType for StringEvent {
    fn topic(&self) -> &'static str {
        self.topic
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn make_event(topic: &'static str, data: &str) -> StringEvent {
        StringEvent { topic, data: data.into() }
    }

    #[test]
    fn new_bus_starts_empty() {
        let bus: EventBus<StringEvent> = EventBus::new();
        assert_eq!(bus.published_count(), 0);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn subscribe_increments_subscriber_count() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        bus.subscribe("a", |_| {});
        bus.subscribe("a", |_| {});
        bus.subscribe("b", |_| {});
        assert_eq!(bus.subscriber_count(), 3);
    }

    #[test]
    fn publish_increments_published_count() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        bus.publish(&make_event("x", ""));
        bus.publish(&make_event("y", ""));
        assert_eq!(bus.published_count(), 2);
    }

    #[test]
    fn subscriber_receives_matching_event() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        let received = Rc::new(Cell::new(0u32));
        let r = Rc::clone(&received);
        bus.subscribe("ping", move |_| { r.set(r.get() + 1); });
        bus.publish(&make_event("ping", "hi"));
        assert_eq!(received.get(), 1);
    }

    #[test]
    fn subscriber_does_not_receive_different_topic() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        let received = Rc::new(Cell::new(false));
        let r = Rc::clone(&received);
        bus.subscribe("ping", move |_| { r.set(true); });
        bus.publish(&make_event("pong", "hi"));
        assert!(!received.get());
    }

    #[test]
    fn wildcard_subscriber_receives_all_topics() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        let count = Rc::new(Cell::new(0u32));
        let c = Rc::clone(&count);
        bus.subscribe("*", move |_| { c.set(c.get() + 1); });
        bus.publish(&make_event("a", ""));
        bus.publish(&make_event("b", ""));
        bus.publish(&make_event("c", ""));
        assert_eq!(count.get(), 3);
    }

    #[test]
    fn has_subscribers_reflects_subscription_state() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        assert!(!bus.has_subscribers("topic"));
        bus.subscribe("topic", |_| {});
        assert!(bus.has_subscribers("topic"));
    }

    #[test]
    fn topic_subscriber_count_is_per_topic() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        bus.subscribe("a", |_| {});
        bus.subscribe("a", |_| {});
        bus.subscribe("b", |_| {});
        assert_eq!(bus.topic_subscriber_count("a"), 2);
        assert_eq!(bus.topic_subscriber_count("b"), 1);
        assert_eq!(bus.topic_subscriber_count("c"), 0);
    }

    #[test]
    fn publish_with_no_subscribers_does_not_panic() {
        let mut bus: EventBus<StringEvent> = EventBus::new();
        bus.publish(&make_event("orphan", "data")); // must not panic
        assert_eq!(bus.published_count(), 1);
    }
}
