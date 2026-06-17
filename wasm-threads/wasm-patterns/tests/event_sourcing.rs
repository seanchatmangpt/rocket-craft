use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_patterns::{Aggregate, DomainEvent, EventLog, EventSourcedRepo};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum GameEvent {
    PlayerSpawned { id: u32, x: f32, y: f32 },
    PlayerMoved { id: u32, dx: f32, dy: f32 },
    PlayerDied { id: u32 },
}

impl DomainEvent for GameEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::PlayerSpawned { .. } => "PlayerSpawned",
            Self::PlayerMoved { .. } => "PlayerMoved",
            Self::PlayerDied { .. } => "PlayerDied",
        }
    }
    fn sequence(&self) -> u64 {
        0
    }
}

#[derive(Debug, Clone, Default)]
struct GameAggregate {
    player_count: u32,
    total_moves: u32,
}

impl Aggregate for GameAggregate {
    type Event = GameEvent;
    fn apply(&mut self, event: &GameEvent) {
        match event {
            GameEvent::PlayerSpawned { .. } => self.player_count += 1,
            GameEvent::PlayerMoved { .. } => self.total_moves += 1,
            GameEvent::PlayerDied { .. } => {
                if self.player_count > 0 {
                    self.player_count -= 1;
                }
            }
        }
    }
    fn version(&self) -> u64 {
        0
    }
}

#[test]
fn event_log_appends_and_tracks_version() {
    let log = log();
    log.info("Given an empty EventLog");
    let mut el = EventLog::<GameEvent>::new();

    log.info("When a PlayerSpawned event is appended");
    el.append(GameEvent::PlayerSpawned { id: 1, x: 0.0, y: 0.0 });

    log.info("Then the log version is 1 and len is 1");
    assert_eq!(el.version(), 1);
    assert_eq!(el.len(), 1);
}

#[test]
fn state_reconstruction_reflects_all_appended_events() {
    let log = log();
    log.info("Given an EventSourcedRepo with no events");
    let mut repo = EventSourcedRepo::<GameAggregate>::new(100);
    let empty = repo.reconstruct();
    assert_eq!(empty.player_count, 0);

    log.info("When events are appended");
    repo.append(GameEvent::PlayerSpawned { id: 1, x: 0.0, y: 0.0 });
    repo.append(GameEvent::PlayerSpawned { id: 2, x: 10.0, y: 0.0 });
    repo.append(GameEvent::PlayerMoved { id: 1, dx: 1.0, dy: 0.0 });
    repo.append(GameEvent::PlayerDied { id: 2 });

    log.info("Then reconstruction must reflect those events — not return the default state");
    let state = repo.reconstruct();
    assert_eq!(state.player_count, 1);
    assert_eq!(state.total_moves, 1);
    assert_ne!(empty.player_count, state.player_count, "reconstruction must change after events");
}

#[test]
fn snapshot_triggers_at_configured_interval_and_replay_succeeds() {
    let log = log();
    log.info("Given an EventSourcedRepo with snapshot interval of 3");
    let mut repo = EventSourcedRepo::<GameAggregate>::new(3);

    log.info("When 6 PlayerSpawned events are appended");
    for i in 0..6u32 {
        repo.append(GameEvent::PlayerSpawned { id: i, x: 0.0, y: 0.0 });
    }

    log.info("Then a snapshot exists and reconstruction returns the full count");
    assert!(repo.has_snapshot());
    let state = repo.reconstruct();
    assert_eq!(state.player_count, 6);
}

proptest! {
    #[test]
    fn event_log_version_monotonically_increases(n in 1usize..50) {
        let log = log();
        log.info("Given an EventLog");
        let mut el = EventLog::<GameEvent>::new();

        log.info("When n events are appended one by one");
        for i in 0..n {
            el.append(GameEvent::PlayerSpawned { id: i as u32, x: 0.0, y: 0.0 });
            prop_assert_eq!(el.version(), (i + 1) as u64);
        }

        log.info("Then version increments monotonically with each append");
    }

    #[test]
    fn reconstruction_is_deterministic(spawns in 1u32..20, moves in 0u32..20) {
        let log = log();
        log.info("Given an EventSourcedRepo with spawn and move events");
        let mut repo = EventSourcedRepo::<GameAggregate>::new(100);
        for i in 0..spawns {
            repo.append(GameEvent::PlayerSpawned { id: i, x: 0.0, y: 0.0 });
        }
        for i in 0..moves {
            repo.append(GameEvent::PlayerMoved { id: 0, dx: i as f32, dy: 0.0 });
        }

        log.info("When reconstruct is called twice");
        let s1 = repo.reconstruct();
        let s2 = repo.reconstruct();

        log.info("Then both reconstructions produce identical state");
        prop_assert_eq!(s1.player_count, s2.player_count);
        prop_assert_eq!(s1.total_moves, s2.total_moves);
    }
}
