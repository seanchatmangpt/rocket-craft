pub trait DomainEvent: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static {
    fn event_type(&self) -> &'static str;
    fn sequence(&self) -> u64;
}

pub trait Aggregate: Default + Clone {
    type Event: DomainEvent;
    fn apply(&mut self, event: &Self::Event);
    fn version(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct EventLog<E: DomainEvent> {
    events: Vec<E>,
    version: u64,
}

impl<E: DomainEvent> EventLog<E> {
    pub fn new() -> Self { Self { events: Vec::new(), version: 0 } }
    pub fn append(&mut self, event: E) { self.version += 1; self.events.push(event); }
    pub fn events(&self) -> &[E] { &self.events }
    pub fn version(&self) -> u64 { self.version }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
    pub fn events_since(&self, version: u64) -> &[E] {
        let idx = version as usize;
        if idx >= self.events.len() { &[] } else { &self.events[idx..] }
    }
}

impl<E: DomainEvent> Default for EventLog<E> {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct Snapshot<A: Aggregate> {
    pub state: A,
    pub at_version: u64,
}

impl<A: Aggregate + Clone> Snapshot<A> {
    pub fn take(state: &A, at_version: u64) -> Self { Self { state: state.clone(), at_version } }
}

pub struct EventSourcedRepo<A: Aggregate> {
    log: EventLog<A::Event>,
    snapshot: Option<Snapshot<A>>,
    snapshot_interval: u64,
}

impl<A: Aggregate + Clone> EventSourcedRepo<A> {
    pub fn new(snapshot_interval: u64) -> Self {
        Self { log: EventLog::new(), snapshot: None, snapshot_interval }
    }
    pub fn append(&mut self, event: A::Event) {
        self.log.append(event);
        if self.snapshot_interval > 0 && self.log.version() % self.snapshot_interval == 0 {
            let state = self.reconstruct();
            self.snapshot = Some(Snapshot::take(&state, self.log.version()));
        }
    }
    pub fn reconstruct(&self) -> A {
        let (mut state, start_version) = match &self.snapshot {
            Some(snap) => (snap.state.clone(), snap.at_version),
            None => (A::default(), 0),
        };
        for event in self.log.events_since(start_version) {
            state.apply(event);
        }
        state
    }
    pub fn version(&self) -> u64 { self.log.version() }
    pub fn event_count(&self) -> usize { self.log.len() }
    pub fn has_snapshot(&self) -> bool { self.snapshot.is_some() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // ── Minimal fixture: counter aggregate ───────────────────────────────────

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum CounterEvent { Incremented(u64), Decremented(u64) }

    impl DomainEvent for CounterEvent {
        fn event_type(&self) -> &'static str {
            match self { CounterEvent::Incremented(_) => "incremented", CounterEvent::Decremented(_) => "decremented" }
        }
        fn sequence(&self) -> u64 { 0 }
    }

    #[derive(Debug, Clone, Default)]
    struct Counter { value: i64, version: u64 }
    impl Aggregate for Counter {
        type Event = CounterEvent;
        fn apply(&mut self, e: &CounterEvent) {
            match e {
                CounterEvent::Incremented(n) => self.value += *n as i64,
                CounterEvent::Decremented(n) => self.value -= *n as i64,
            }
            self.version += 1;
        }
        fn version(&self) -> u64 { self.version }
    }

    // ── EventLog ──────────────────────────────────────────────────────────────

    #[test]
    fn new_log_is_empty_at_version_zero() {
        let log: EventLog<CounterEvent> = EventLog::new();
        assert!(log.is_empty());
        assert_eq!(log.version(), 0);
    }

    #[test]
    fn append_increments_version_and_len() {
        let mut log: EventLog<CounterEvent> = EventLog::new();
        log.append(CounterEvent::Incremented(1));
        log.append(CounterEvent::Incremented(1));
        assert_eq!(log.len(), 2);
        assert_eq!(log.version(), 2);
    }

    #[test]
    fn events_since_returns_slice_from_index() {
        let mut log: EventLog<CounterEvent> = EventLog::new();
        log.append(CounterEvent::Incremented(1));
        log.append(CounterEvent::Incremented(2));
        log.append(CounterEvent::Incremented(3));
        let slice = log.events_since(1);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn events_since_past_end_returns_empty() {
        let mut log: EventLog<CounterEvent> = EventLog::new();
        log.append(CounterEvent::Incremented(1));
        assert_eq!(log.events_since(99).len(), 0);
    }

    // ── EventSourcedRepo ─────────────────────────────────────────────────────

    #[test]
    fn reconstruct_applies_all_events() {
        let mut repo: EventSourcedRepo<Counter> = EventSourcedRepo::new(0);
        repo.append(CounterEvent::Incremented(5));
        repo.append(CounterEvent::Incremented(3));
        repo.append(CounterEvent::Decremented(2));
        let state = repo.reconstruct();
        assert_eq!(state.value, 6);
    }

    #[test]
    fn snapshot_taken_at_interval() {
        let mut repo: EventSourcedRepo<Counter> = EventSourcedRepo::new(3);
        assert!(!repo.has_snapshot());
        repo.append(CounterEvent::Incremented(1));
        repo.append(CounterEvent::Incremented(1));
        repo.append(CounterEvent::Incremented(1)); // triggers snapshot at v3
        assert!(repo.has_snapshot());
    }

    #[test]
    fn reconstruct_after_snapshot_is_correct() {
        let mut repo: EventSourcedRepo<Counter> = EventSourcedRepo::new(2);
        repo.append(CounterEvent::Incremented(10));
        repo.append(CounterEvent::Incremented(10)); // snapshot at v2 (value=20)
        repo.append(CounterEvent::Decremented(5));
        let state = repo.reconstruct();
        assert_eq!(state.value, 15);
    }

    #[test]
    fn version_and_event_count_match_appends() {
        let mut repo: EventSourcedRepo<Counter> = EventSourcedRepo::new(0);
        repo.append(CounterEvent::Incremented(1));
        repo.append(CounterEvent::Incremented(1));
        assert_eq!(repo.version(), 2);
        assert_eq!(repo.event_count(), 2);
    }
}
