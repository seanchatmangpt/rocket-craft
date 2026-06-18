use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_patterns::{EventBus, EventType};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum UiEvent {
    ButtonClicked(String),
    HealthChanged(u32),
    ScoreChanged(u64),
}

impl EventType for UiEvent {
    fn topic(&self) -> &'static str {
        match self {
            Self::ButtonClicked(_) => "button",
            Self::HealthChanged(_) => "health",
            Self::ScoreChanged(_) => "score",
        }
    }
}

#[test]
fn topic_subscriber_receives_only_matching_events() {
    let log = log();
    log.info("Given an EventBus with a subscriber on the 'health' topic");
    let received = Arc::new(Mutex::new(Vec::<u32>::new()));
    let received2 = received.clone();
    let mut bus = EventBus::<UiEvent>::new();
    bus.subscribe("health", move |e| {
        if let UiEvent::HealthChanged(hp) = e {
            received2.lock().unwrap().push(*hp);
        }
    });

    log.info("When health and non-health events are published");
    bus.publish(&UiEvent::HealthChanged(75));
    bus.publish(&UiEvent::HealthChanged(50));
    bus.publish(&UiEvent::ButtonClicked("start".to_string()));

    log.info("Then only health events are delivered to the subscriber");
    let v = received.lock().unwrap();
    assert_eq!(*v, vec![75, 50]);
}

#[test]
fn subscriber_does_not_fire_for_wrong_topic() {
    let log = log();
    log.info("Given an EventBus with a subscriber on the 'score' topic");
    let count = Arc::new(Mutex::new(0u32));
    let count2 = count.clone();
    let mut bus = EventBus::<UiEvent>::new();
    bus.subscribe("score", move |_| {
        *count2.lock().unwrap() += 1;
    });

    log.info("When events on other topics are published");
    bus.publish(&UiEvent::HealthChanged(50));
    bus.publish(&UiEvent::ButtonClicked("x".to_string()));

    log.info("Then the score subscriber callback count remains zero");
    assert_eq!(*count.lock().unwrap(), 0, "subscriber must not fire for wrong topic");
}

#[test]
fn wildcard_subscriber_receives_all_topics() {
    let log = log();
    log.info("Given an EventBus with a wildcard '*' subscriber");
    let count = Arc::new(Mutex::new(0u32));
    let count2 = count.clone();
    let mut bus = EventBus::<UiEvent>::new();
    bus.subscribe("*", move |_| {
        *count2.lock().unwrap() += 1;
    });

    log.info("When events on three different topics are published");
    bus.publish(&UiEvent::HealthChanged(50));
    bus.publish(&UiEvent::ScoreChanged(100));
    bus.publish(&UiEvent::ButtonClicked("x".to_string()));

    log.info("Then the wildcard subscriber fires for every event");
    assert_eq!(*count.lock().unwrap(), 3);
}

#[test]
fn topic_isolation_prevents_cross_topic_delivery() {
    let log = log();
    log.info("Given an EventBus with separate subscribers for 'health' and 'score'");
    let health_count = Arc::new(Mutex::new(0u32));
    let health_count2 = health_count.clone();
    let score_count = Arc::new(Mutex::new(0u32));
    let score_count2 = score_count.clone();
    let mut bus = EventBus::<UiEvent>::new();
    bus.subscribe("health", move |_| {
        *health_count2.lock().unwrap() += 1;
    });
    bus.subscribe("score", move |_| {
        *score_count2.lock().unwrap() += 1;
    });

    log.info("When a health event and a score event are published");
    bus.publish(&UiEvent::HealthChanged(80));
    bus.publish(&UiEvent::ScoreChanged(200));

    log.info("Then each subscriber receives exactly its own topic");
    assert_eq!(*health_count.lock().unwrap(), 1);
    assert_eq!(*score_count.lock().unwrap(), 1);
}

proptest! {
    #[test]
    fn event_bus_publish_count_monotonically_increases(n in 1usize..50) {
        let log = log();
        log.info("Given an EventBus");
        let mut bus = EventBus::<UiEvent>::new();

        log.info("When n events are published");
        for i in 0..n {
            bus.publish(&UiEvent::ScoreChanged(i as u64));
        }

        log.info("Then published_count equals n");
        prop_assert_eq!(bus.published_count(), n as u64);
    }
}
