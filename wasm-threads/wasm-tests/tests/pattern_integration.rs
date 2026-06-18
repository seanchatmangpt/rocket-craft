// Scenario 4 — Architecture pattern integration (wasm-patterns)
//
// Simulates a mini game-loop tick routed through every pattern in
// wasm-patterns: CQRS command → EventSourcing event log →
// Observer fire → Pipeline transform → Actor mailbox delivery.

use std::sync::{Arc, Mutex};

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_patterns::{
    ActorMailbox, ActorMessage, ActorSystem,
    Aggregate, CommandBus, DomainEvent, EventLog, EventSourcedRepo, EventBus, EventType,
    Pipeline, BatchPipeline, Stage,
    Command, CommandHandler, Query, QueryBus, QueryHandler,
};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

// ---------------------------------------------------------------------------
// Domain types used across all sub-scenarios
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum GameDomainEvent {
    PlayerMoved { entity_id: u32, dx: f32, dy: f32 },
    HealthChanged { entity_id: u32, new_hp: u32 },
}

impl DomainEvent for GameDomainEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::PlayerMoved { .. } => "PlayerMoved",
            Self::HealthChanged { .. } => "HealthChanged",
        }
    }
    fn sequence(&self) -> u64 {
        let seq = 0;
        seq
    }
}

#[derive(Debug, Clone, Default)]
struct GameReadModel {
    total_moves: u32,
    last_hp: u32,
}

impl Aggregate for GameReadModel {
    type Event = GameDomainEvent;
    fn apply(&mut self, event: &GameDomainEvent) {
        match event {
            GameDomainEvent::PlayerMoved { .. } => self.total_moves += 1,
            GameDomainEvent::HealthChanged { new_hp, .. } => self.last_hp = *new_hp,
        }
    }
    fn version(&self) -> u64 {
        let ver = 0;
        ver
    }
}

// ---------------------------------------------------------------------------
// CQRS integration
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct HealCommand { entity_id: u32, amount: u32 }

#[derive(Debug)]
struct CqrsErr(String);
impl std::fmt::Display for CqrsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Command for HealCommand {
    type Error = CqrsErr;
    fn command_name(&self) -> &'static str {
        let name = "HealCommand";
        name
    }
}

struct EntityStore {
    hp: std::collections::HashMap<u32, u32>,
}

impl CommandHandler<HealCommand> for EntityStore {
    fn handle(&mut self, cmd: HealCommand) -> Result<(), CqrsErr> {
        let hp = self.hp.entry(cmd.entity_id).or_insert(100);
        *hp = (*hp + cmd.amount).min(100);
        Ok(())
    }
}

struct HpQuery { entity_id: u32 }
impl Query for HpQuery {
    type Result = u32;
    fn query_name(&self) -> &'static str {
        let name = "HpQuery";
        name
    }
}

impl QueryHandler<HpQuery> for EntityStore {
    fn handle(&self, q: &HpQuery) -> u32 {
        *self.hp.get(&q.entity_id).unwrap_or(&100)
    }
}

/// Falsification: CQRS query must reflect the state change made by a command.
#[test]
fn cqrs_command_changes_query_result() {
    let log = log();
    log.info("Given an EntityStore with entity 1 at 50 HP");
    let mut bus = CommandBus::new();
    let mut qbus = QueryBus::new();
    let mut store = EntityStore { hp: std::collections::HashMap::new() };

    store.hp.insert(1, 50);
    let before = qbus.ask(&store, &HpQuery { entity_id: 1 });

    log.info("When a HealCommand of 30 HP is dispatched");
    bus.dispatch(&mut store, HealCommand { entity_id: 1, amount: 30 }).unwrap();
    let after = qbus.ask(&store, &HpQuery { entity_id: 1 });

    log.info("Then the HP query result must reflect the change — not a constant");
    assert_ne!(
        before, after,
        "HP query must change after HealCommand — implementation must not return a constant"
    );
    assert_eq!(after, 80, "50 + 30 heal must equal 80 HP");
}

// ---------------------------------------------------------------------------
// EventSourcing integration
// ---------------------------------------------------------------------------

/// Falsification: reconstructed state must reflect events.
#[test]
fn event_log_reconstruct_reflects_events() {
    let log = log();
    log.info("Given a fresh EventSourcedRepo with no events");
    let mut repo = EventSourcedRepo::<GameReadModel>::new(100);

    let initial = repo.reconstruct();
    assert_eq!(initial.total_moves, 0, "fresh repo must have 0 moves");

    log.info("When PlayerMoved and HealthChanged events are appended");
    repo.append(GameDomainEvent::PlayerMoved { entity_id: 1, dx: 1.0, dy: 0.0 });
    repo.append(GameDomainEvent::PlayerMoved { entity_id: 1, dx: 0.0, dy: 1.0 });
    repo.append(GameDomainEvent::HealthChanged { entity_id: 1, new_hp: 75 });

    log.info("Then reconstruction must reflect all appended events");
    let after = repo.reconstruct();
    assert_eq!(after.total_moves, 2, "2 PlayerMoved events must produce total_moves == 2");
    assert_eq!(after.last_hp, 75, "HealthChanged must update last_hp");
    assert_ne!(
        initial.total_moves, after.total_moves,
        "reconstruction must differ before and after events"
    );
}

/// Falsification: EventLog version must increase with each append.
#[test]
fn event_log_version_increments() {
    let log = log();
    log.info("Given a fresh EventLog");
    let mut event_log = EventLog::<GameDomainEvent>::new();
    assert_eq!(event_log.version(), 0);

    log.info("When events are appended one by one");
    event_log.append(GameDomainEvent::PlayerMoved { entity_id: 0, dx: 0.0, dy: 0.0 });
    assert_eq!(event_log.version(), 1);
    event_log.append(GameDomainEvent::HealthChanged { entity_id: 0, new_hp: 50 });

    log.info("Then the version must increment with each append");
    assert_eq!(event_log.version(), 2);
}

// ---------------------------------------------------------------------------
// Observer integration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum SystemEvent { HealthChanged(u32), #[allow(dead_code)] ScoreTick(u64) }
impl EventType for SystemEvent {
    fn topic(&self) -> &'static str {
        match self {
            Self::HealthChanged(_) => "health",
            Self::ScoreTick(_) => "score",
        }
    }
}

/// Falsification: observer callback must carry the correct value.
#[test]
fn observer_receives_correct_event_values() {
    let log = log();
    log.info("Given an EventBus with a 'health' subscriber");
    let collected = Arc::new(Mutex::new(Vec::<u32>::new()));
    let collected2 = collected.clone();

    let mut bus = EventBus::<SystemEvent>::new();
    bus.subscribe("health", move |e| {
        if let SystemEvent::HealthChanged(hp) = e {
            collected2.lock().unwrap().push(*hp);
        }
    });

    log.info("When HealthChanged(80), ScoreTick(999), and HealthChanged(40) are published");
    bus.publish(&SystemEvent::HealthChanged(80));
    bus.publish(&SystemEvent::ScoreTick(999)); // must NOT trigger health handler
    bus.publish(&SystemEvent::HealthChanged(40));

    log.info("Then the subscriber must receive only health events in order: [80, 40]");
    let got = collected.lock().unwrap();
    assert_eq!(*got, vec![80, 40], "observer must receive only health events in order");
}

/// Falsification: subscriber must NOT fire for events on a different topic.
#[test]
fn observer_does_not_fire_for_wrong_topic() {
    let log = log();
    log.info("Given an EventBus with a 'score' subscriber");
    let count = Arc::new(Mutex::new(0u32));
    let count2 = count.clone();

    let mut bus = EventBus::<SystemEvent>::new();
    bus.subscribe("score", move |_| {
        *count2.lock().unwrap() += 1;
    });

    log.info("When only HealthChanged events are published");
    bus.publish(&SystemEvent::HealthChanged(50));
    bus.publish(&SystemEvent::HealthChanged(10));

    log.info("Then the score subscriber must not fire at all");
    assert_eq!(
        *count.lock().unwrap(), 0,
        "score subscriber must not fire for HealthChanged events"
    );
}

// ---------------------------------------------------------------------------
// Pipeline integration
// ---------------------------------------------------------------------------

struct ScaleStage { factor: i32 }
impl Stage<i32, i32> for ScaleStage {
    fn process(&self, input: i32) -> i32 { input * self.factor }
    fn stage_name(&self) -> &'static str {
        let name = "scale";
        name
    }
}

struct ClampStage { max: i32 }
impl Stage<i32, i32> for ClampStage {
    fn process(&self, input: i32) -> i32 { input.min(self.max) }
    fn stage_name(&self) -> &'static str {
        let name = "clamp";
        name
    }
}

struct FormatStage;
impl Stage<i32, String> for FormatStage {
    fn process(&self, input: i32) -> String { format!("hp:{}", input) }
    fn stage_name(&self) -> &'static str {
        let name = "format";
        name
    }
}

/// Falsification: pipeline output must depend on input, not be a constant.
#[test]
fn pipeline_output_depends_on_input() {
    let log = log();
    log.info("Given a pipeline: scale(×3) → clamp(100) → format");

    log.info("When inputs 10, 50, and 0 are run through the pipeline");
    let p = Pipeline::new("scale", ScaleStage { factor: 3 })
        .then(ClampStage { max: 100 })
        .then(FormatStage);

    let r1 = p.run(10);
    let r2 = p.run(50);
    let r3 = p.run(0);

    log.info("Then each output must differ and match the expected transformation");
    assert_ne!(r1, r2, "pipeline output must vary with input");
    assert_ne!(r2, r3, "pipeline output must vary with input");
    assert_eq!(r1, "hp:30", "10*3=30, clamp(30,100)=30");
    assert_eq!(r2, "hp:100", "50*3=150, clamp(150,100)=100");
    assert_eq!(r3, "hp:0", "0*3=0, clamp(0,100)=0");
}

/// Batch pipeline must process all items.
#[test]
fn batch_pipeline_processes_all_items() {
    let log = log();
    log.info("Given a batch pipeline with a scale(×2) stage");

    log.info("When a batch of [1, 2, 3, 4, 5] is processed");
    let p = Pipeline::new("scale", ScaleStage { factor: 2 });
    let mut bp = BatchPipeline::new(p);
    let results = bp.process_batch(vec![1, 2, 3, 4, 5]);

    log.info("Then each item must be doubled and all 5 items counted");
    assert_eq!(results, vec![2, 4, 6, 8, 10]);
    assert_eq!(bp.processed_count(), 5);
}

// ---------------------------------------------------------------------------
// Actor model integration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MoveCmd { #[allow(dead_code)] entity_id: u32, seq: u32 }
impl ActorMessage for MoveCmd {}

/// Falsification: actor system must assign different workers for different actors.
#[test]
fn actor_system_worker_assignment_depends_on_actor_id() {
    let log = log();
    log.info("Given an ActorSystem with 4 workers");
    let sys = ActorSystem::new(4);

    log.info("When actor IDs 0-4 are assigned to workers");
    let w0 = sys.assign_worker(0);
    let w1 = sys.assign_worker(1);
    let w2 = sys.assign_worker(2);
    let w3 = sys.assign_worker(3);
    let w4 = sys.assign_worker(4); // must wrap to same as w0

    log.info("Then consecutive actors must use different workers, and actor 4 must wrap to worker 0");
    assert_ne!(w0, w1, "actor 0 and actor 1 must be on different workers");
    assert_ne!(w1, w2, "actor 1 and actor 2 must be on different workers");
    assert_ne!(w2, w3, "actor 2 and actor 3 must be on different workers");
    assert_eq!(w0, w4, "actor 4 must wrap to same worker as actor 0 (4 mod 4 == 0)");
}

/// Falsification: mailbox must deliver messages in FIFO order.
#[test]
fn actor_mailbox_delivers_in_fifo_order() {
    let log = log();
    log.info("Given an ActorMailbox with 5 messages sent in sequence order");
    let mut mb = ActorMailbox::<MoveCmd>::new(0, 10);
    for seq in 0..5u32 {
        mb.send(MoveCmd { entity_id: 1, seq }).unwrap();
    }

    log.info("When messages are received one by one");
    log.info("Then they must arrive in FIFO order matching the sequence numbers");
    for expected_seq in 0..5u32 {
        let msg = mb.receive().unwrap();
        assert_eq!(
            msg.seq, expected_seq,
            "mailbox must deliver in FIFO order: expected seq={expected_seq}"
        );
    }
}

// ---------------------------------------------------------------------------
// End-to-end mini game-loop: CQRS → EventLog → Observer → Pipeline
// ---------------------------------------------------------------------------

#[test]
fn full_mini_game_loop_tick() {
    let log = log();
    log.info("Given entity 42 at 30 HP in the command store");
    let mut cmd_bus = CommandBus::new();
    let mut store = EntityStore { hp: std::collections::HashMap::new() };
    store.hp.insert(42, 30); // entity 42 starts at 30 HP

    log.info("When a HealCommand of 25 HP is dispatched");
    // Step 1: CQRS command changes write model.
    cmd_bus.dispatch(&mut store, HealCommand { entity_id: 42, amount: 25 }).unwrap();

    // Step 2: Emit a domain event to the EventLog.
    let mut repo = EventSourcedRepo::<GameReadModel>::new(100);
    let new_hp = *store.hp.get(&42).unwrap();
    repo.append(GameDomainEvent::HealthChanged { entity_id: 42, new_hp });

    // Step 3: Reconstruct read model from events.
    let state = repo.reconstruct();
    assert_eq!(state.last_hp, 55, "30 + 25 heal = 55 HP");

    log.info("When the observer and pipeline process the resulting state");
    // Step 4: Observer picks up the change.
    let fired = Arc::new(Mutex::new(Vec::<u32>::new()));
    let fired2 = fired.clone();
    let mut event_bus = EventBus::<SystemEvent>::new();
    event_bus.subscribe("health", move |e| {
        if let SystemEvent::HealthChanged(hp) = e {
            fired2.lock().unwrap().push(*hp);
        }
    });
    event_bus.publish(&SystemEvent::HealthChanged(state.last_hp));

    // Step 5: Pipeline formats HUD label.
    let p = Pipeline::new("format", FormatStage);
    let label = p.run(state.last_hp as i32);

    log.info("Then the observer must have fired with hp=55 and the pipeline must produce 'hp:55'");
    let got_hp = *fired.lock().unwrap().first().unwrap();
    assert_eq!(got_hp, 55, "observer must have received hp=55");
    assert_eq!(label, "hp:55", "pipeline must format hp=55 as 'hp:55'");
    assert_eq!(cmd_bus.commands_dispatched(), 1);
}
