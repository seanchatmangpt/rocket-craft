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
