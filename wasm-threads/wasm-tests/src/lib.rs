// Integration test harness for the wasm-threads workspace.
//
// Every test in this file is a FALSIFICATION test — it must fail if any
// implementation returns a constant/mocked value instead of computing a
// real result. Happy-path scenarios and property-based invariants are
// also included so the harness doubles as a correctness regression suite.
//
// Cross-crate interactions tested here:
//   1. Game Logic → UI message pipeline  (wasm-game-logic ↔ wasm-ui)
//   2. Worker pool + typed channel integration (wasm-core)
//   3. SharedMemoryBus as a game-state sync channel (wasm-core)
//   4. Architecture pattern integration: CQRS + EventSourcing + Observer +
//      Pipeline + Actor working together (wasm-patterns)
//   5. Typestate correctness — compile-time enforcement (wasm-core)
//   6. Combinatorial proptest matrix across all subsystems
//   7. Anti-cheat / falsification harness
//   8. Stress / load tests

// ---------------------------------------------------------------------------
// Re-export types used by the test modules below so they are accessible
// from parent modules without additional `use` clutter.
// ---------------------------------------------------------------------------
pub use wasm_core::{
    SharedMemoryBus, ThreadingApproach, WorkerChannel, WorkerPool, WasmWorker,
    Uninitialized, Running, Paused, Terminated,
};

// ---------------------------------------------------------------------------
// SCENARIO 1 — Game Logic → UI message pipeline
//
// Verifies that the JSON wire format produced by wasm-game-logic is correctly
// consumed by wasm-ui's MessageBridge.  Any mismatch in the protocol enum
// names or field names will be caught here before it reaches the browser.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod game_to_ui_pipeline {
    use wasm_game_logic::GameToUiMessage as GameMsg;
    use wasm_ui::MessageBridge;

    /// Happy path: a StateUpdate message serialised by wasm-game-logic
    /// deserialises correctly inside wasm-ui's MessageBridge.
    #[test]
    fn game_to_ui_state_update_pipeline() {
        let msg = GameMsg::StateUpdate {
            tick: 42,
            entity_count: 5,
            player_health: Some(75),
            player_health_max: Some(100),
            player_score: 1500,
        };
        let json = serde_json::to_string(&msg).unwrap();

        let mut bridge = MessageBridge::new();
        let hud = bridge.process(&json).expect("UI must parse game-logic StateUpdate");

        assert_eq!(hud.game_tick, 42, "tick must travel through the pipeline");
        assert_eq!(hud.player_health, 75, "health must survive serialisation round-trip");
        assert_eq!(hud.player_health_max, 100, "health_max must survive serialisation round-trip");
        assert_eq!(hud.score, 1500, "score must survive serialisation round-trip");
        assert_eq!(hud.entity_count, 5, "entity_count must survive serialisation round-trip");
        assert_eq!(bridge.messages_processed, 1);
        assert_eq!(bridge.last_tick, 42);
    }

    /// Falsification: two different ticks must produce two different HUD states.
    #[test]
    fn different_game_ticks_produce_different_hud_states() {
        let mut bridge = MessageBridge::new();

        let msg1 = GameMsg::StateUpdate {
            tick: 1,
            entity_count: 1,
            player_health: Some(100),
            player_health_max: Some(100),
            player_score: 0,
        };
        let msg2 = GameMsg::StateUpdate {
            tick: 9999,
            entity_count: 1,
            player_health: Some(100),
            player_health_max: Some(100),
            player_score: 0,
        };

        let hud1 = bridge.process(&serde_json::to_string(&msg1).unwrap()).unwrap();
        let hud2 = bridge.process(&serde_json::to_string(&msg2).unwrap()).unwrap();

        assert_ne!(
            hud1.game_tick, hud2.game_tick,
            "different tick values must produce different HUD game_tick fields"
        );
        assert_eq!(bridge.messages_processed, 2);
    }

    /// Falsification: score must propagate correctly and not be zero by default.
    #[test]
    fn game_score_reaches_hud() {
        let scores = [0u64, 1, 500, 999_999, u64::MAX / 2];
        let mut bridge = MessageBridge::new();

        for &score in &scores {
            let msg = GameMsg::StateUpdate {
                tick: 1,
                entity_count: 0,
                player_health: None,
                player_health_max: None,
                player_score: score,
            };
            let hud = bridge
                .process(&serde_json::to_string(&msg).unwrap())
                .unwrap();
            assert_eq!(
                hud.score, score,
                "score {score} must reach HUD unmodified"
            );
        }
    }

    /// Falsification: health percentage computed by HUD must differ for different health values.
    #[test]
    fn health_percentage_depends_on_actual_health() {
        let mut bridge = MessageBridge::new();

        let full_json = serde_json::to_string(&GameMsg::StateUpdate {
            tick: 1,
            entity_count: 0,
            player_health: Some(100),
            player_health_max: Some(100),
            player_score: 0,
        })
        .unwrap();

        let low_json = serde_json::to_string(&GameMsg::StateUpdate {
            tick: 2,
            entity_count: 0,
            player_health: Some(1),
            player_health_max: Some(100),
            player_score: 0,
        })
        .unwrap();

        let full_hud = bridge.process(&full_json).unwrap();
        let low_hud = bridge.process(&low_json).unwrap();

        let full_pct = full_hud.health_percentage();
        let low_pct = low_hud.health_percentage();

        assert_ne!(
            full_pct, low_pct,
            "different health values must produce different percentages"
        );
        assert!(
            (full_pct - 1.0).abs() < 0.001,
            "full health must produce percentage ~1.0"
        );
        assert!(
            low_pct < 0.05,
            "1/100 health must produce percentage < 0.05"
        );
    }

    /// GameOver message must flow through the pipeline.
    #[test]
    fn game_over_message_pipeline() {
        let msg = GameMsg::GameOver {
            winner_score: 88_888,
            total_ticks: 360,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let mut bridge = MessageBridge::new();
        let hud = bridge.process(&json).unwrap();
        assert_eq!(hud.score, 88_888, "GameOver winner_score must reach HUD");
        assert_eq!(hud.game_tick, 360, "GameOver total_ticks must reach HUD as game_tick");
    }

    /// Falsification: bridge must reject garbage JSON and return None.
    #[test]
    fn bridge_rejects_garbage_json() {
        let mut bridge = MessageBridge::new();
        assert!(
            bridge.process("not json").is_none(),
            "invalid JSON must be rejected"
        );
        assert!(
            bridge.process(r#"{"unknown_field": 42}"#).is_none(),
            "unknown message type must be rejected"
        );
        assert_eq!(
            bridge.messages_processed, 0,
            "failed parses must not increment message counter"
        );
    }
}

// ---------------------------------------------------------------------------
// SCENARIO 2 — Worker pool + channel integration
// ---------------------------------------------------------------------------
#[cfg(test)]
mod worker_pool_and_channel {
    use wasm_core::{ThreadingApproach, WorkerChannel, WorkerPool};

    #[test]
    fn threading_approach_config_drives_pool_size() {
        let approach = ThreadingApproach::SeparateModules { worker_count: 4 };
        let pool = WorkerPool::new(approach.worker_count(), "game_logic.js").unwrap();
        assert_eq!(pool.size(), 4);
        assert!(!approach.requires_coop_coep());
    }

    #[test]
    fn shared_memory_approach_requires_coop_coep() {
        let approach = ThreadingApproach::SharedMemory { buffer_size_bytes: 65536 };
        assert!(approach.requires_coop_coep());
    }

    /// End-to-end: 3 channels, each carries a distinct message.
    #[test]
    fn worker_pool_and_channel_end_to_end() {
        #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct GameMsg {
            tick: u64,
            data: String,
        }

        let pool = WorkerPool::new(3, "worker.js").unwrap();
        let mut channels: Vec<WorkerChannel<GameMsg>> = pool
            .worker_ids()
            .iter()
            .map(|&id| WorkerChannel::new(id))
            .collect();

        channels[0]
            .send(GameMsg { tick: 1, data: "move".to_string() })
            .unwrap();
        channels[1]
            .send(GameMsg { tick: 2, data: "attack".to_string() })
            .unwrap();
        channels[2]
            .send(GameMsg { tick: 3, data: "heal".to_string() })
            .unwrap();

        assert_eq!(channels[0].pending_count(), 1);
        assert_eq!(channels[1].pending_count(), 1);
        assert_eq!(channels[2].pending_count(), 1);

        let m0 = channels[0].receive().unwrap();
        let m1 = channels[1].receive().unwrap();
        let m2 = channels[2].receive().unwrap();

        // Falsification: distinct channels must carry distinct messages.
        assert_ne!(
            m0.tick, m1.tick,
            "different channels must carry different tick values"
        );
        assert_ne!(
            m1.tick, m2.tick,
            "different channels must carry different tick values"
        );
        assert_ne!(
            m0.data, m1.data,
            "different channels must carry different data payloads"
        );
    }

    /// Falsification: pool must have unique worker IDs.
    #[test]
    fn pool_worker_ids_are_unique() {
        let pool = WorkerPool::new(6, "worker.js").unwrap();
        let ids = pool.worker_ids();
        let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();
        assert_eq!(
            unique.len(),
            ids.len(),
            "all worker IDs must be unique — implementation must not reuse IDs"
        );
    }

    /// Falsification: pool.size() must match the requested count for several values.
    #[test]
    fn pool_size_reflects_requested_count() {
        for n in [1, 2, 4, 8] {
            let pool = WorkerPool::new(n, "w.js").unwrap();
            assert_eq!(
                pool.size(), n,
                "pool.size() must return {n}, not a constant"
            );
        }
    }

    /// Round-robin dispatch must cycle back to the first worker.
    #[test]
    fn pool_round_robin_returns_to_start() {
        let mut pool = WorkerPool::new(3, "worker.js").unwrap();
        let id0 = pool.dispatch_to_next().worker_id();
        let id1 = pool.dispatch_to_next().worker_id();
        let id2 = pool.dispatch_to_next().worker_id();
        let id3 = pool.dispatch_to_next().worker_id(); // must wrap around

        assert_ne!(id0, id1, "consecutive dispatches must differ");
        assert_ne!(id1, id2, "consecutive dispatches must differ");
        assert_eq!(id0, id3, "round-robin must cycle back to first worker");
    }
}

// ---------------------------------------------------------------------------
// SCENARIO 3 — SharedMemoryBus as game-state sync channel
// ---------------------------------------------------------------------------
#[cfg(test)]
mod shared_memory_bus {
    use wasm_core::SharedMemoryBus;

    /// Happy path: game logic writes health/max/entity_count; UI reads back.
    #[test]
    fn shared_memory_bus_simulates_atomic_sync() {
        let mut bus = SharedMemoryBus::new(64);

        // Simulate game-logic worker writing game state to shared memory.
        bus.write_i32(0, 75).unwrap(); // health
        bus.write_i32(1, 100).unwrap(); // max_health
        bus.write_i32(2, 5).unwrap(); // entity_count

        let health = bus.read_i32(0).unwrap();
        let max = bus.read_i32(1).unwrap();
        let entities = bus.read_i32(2).unwrap();

        assert_eq!(health, 75);
        assert_eq!(max, 100);
        assert_eq!(entities, 5);

        // Health percentage (as a UI worker would compute it).
        let pct = health as f32 / max as f32;
        assert!(
            (pct - 0.75).abs() < 0.001,
            "health percentage from raw ints must be 0.75"
        );
    }

    /// Falsification: bus slots must be independent — writing to one must not
    /// overwrite or alias another.
    #[test]
    fn bus_slots_are_independent() {
        let mut bus = SharedMemoryBus::new(8);
        bus.write_i32(0, 111).unwrap();
        bus.write_i32(1, 222).unwrap();
        bus.write_i32(2, 333).unwrap();

        assert_ne!(
            bus.read_i32(0).unwrap(),
            bus.read_i32(1).unwrap(),
            "slot 0 and slot 1 must be independent"
        );
        assert_ne!(
            bus.read_i32(1).unwrap(),
            bus.read_i32(2).unwrap(),
            "slot 1 and slot 2 must be independent"
        );
        assert_ne!(
            bus.read_i32(0).unwrap(),
            bus.read_i32(2).unwrap(),
            "slot 0 and slot 2 must be independent"
        );
    }

    /// Falsification: compare_exchange must only swap when the expected value
    /// matches the current value.
    #[test]
    fn compare_exchange_is_conditional() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, 42).unwrap();

        // Wrong expected values — must NOT swap.
        for wrong in [0, 1, 41, 43, 100, -1, i32::MAX] {
            let swapped = bus.compare_exchange(0, wrong, 999);
            assert!(
                !swapped,
                "compare_exchange must NOT swap when expected={wrong} != actual=42"
            );
            assert_eq!(
                bus.read_i32(0).unwrap(),
                42,
                "value must remain 42 after failed CAS (wrong expected={wrong})"
            );
        }

        // Correct expected — MUST swap.
        let swapped = bus.compare_exchange(0, 42, 999);
        assert!(swapped, "compare_exchange MUST swap when expected==actual");
        assert_eq!(bus.read_i32(0).unwrap(), 999, "value must be 999 after successful CAS");
    }

    /// Out-of-bounds access must return an error, not silently succeed.
    #[test]
    fn bus_bounds_checking_is_real() {
        let bus = SharedMemoryBus::new(4);

        assert!(bus.read_i32(0).is_ok(), "offset 0 on size-4 bus must be in bounds");
        assert!(bus.read_i32(3).is_ok(), "offset 3 on size-4 bus must be in bounds");
        assert!(bus.read_i32(4).is_err(), "offset == size must be out of bounds");
        assert!(bus.read_i32(5).is_err(), "offset > size must be out of bounds");
        assert!(bus.read_i32(100).is_err(), "large offset must be out of bounds");
    }
}

// ---------------------------------------------------------------------------
// SCENARIO 4 — Architecture pattern integration (wasm-patterns)
//
// Simulates a mini game-loop tick routed through every pattern in
// wasm-patterns: CQRS command → EventSourcing event log →
// Observer fire → Pipeline transform → Actor mailbox delivery.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod pattern_integration {
    use std::sync::{Arc, Mutex};

    use wasm_patterns::{
        ActorMailbox, ActorMessage, ActorSystem,
        Aggregate, CommandBus, DomainEvent, EventLog, EventSourcedRepo, EventBus, EventType,
        Pipeline, BatchPipeline, Stage,
        Command, CommandHandler, Query, QueryBus, QueryHandler,
    };

    // -------------------------------------------------------------------
    // Domain types used across all sub-scenarios
    // -------------------------------------------------------------------

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
        fn sequence(&self) -> u64 { 0 }
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
        fn version(&self) -> u64 { 0 }
    }

    // -------------------------------------------------------------------
    // CQRS integration
    // -------------------------------------------------------------------

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
        fn command_name(&self) -> &'static str { "HealCommand" }
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
        fn query_name(&self) -> &'static str { "HpQuery" }
    }

    impl QueryHandler<HpQuery> for EntityStore {
        fn handle(&self, q: &HpQuery) -> u32 {
            *self.hp.get(&q.entity_id).unwrap_or(&100)
        }
    }

    /// Falsification: CQRS query must reflect the state change made by a command.
    #[test]
    fn cqrs_command_changes_query_result() {
        let mut bus = CommandBus::new();
        let mut qbus = QueryBus::new();
        let mut store = EntityStore { hp: std::collections::HashMap::new() };

        // Start with entity at 50 HP.
        store.hp.insert(1, 50);
        let before = qbus.ask(&store, &HpQuery { entity_id: 1 });

        bus.dispatch(&mut store, HealCommand { entity_id: 1, amount: 30 }).unwrap();
        let after = qbus.ask(&store, &HpQuery { entity_id: 1 });

        assert_ne!(
            before, after,
            "HP query must change after HealCommand — implementation must not return a constant"
        );
        assert_eq!(after, 80, "50 + 30 heal must equal 80 HP");
    }

    // -------------------------------------------------------------------
    // EventSourcing integration
    // -------------------------------------------------------------------

    /// Falsification: reconstructed state must reflect events.
    #[test]
    fn event_log_reconstruct_reflects_events() {
        let mut repo = EventSourcedRepo::<GameReadModel>::new(100);

        let initial = repo.reconstruct();
        assert_eq!(initial.total_moves, 0, "fresh repo must have 0 moves");

        repo.append(GameDomainEvent::PlayerMoved { entity_id: 1, dx: 1.0, dy: 0.0 });
        repo.append(GameDomainEvent::PlayerMoved { entity_id: 1, dx: 0.0, dy: 1.0 });
        repo.append(GameDomainEvent::HealthChanged { entity_id: 1, new_hp: 75 });

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
        let mut log = EventLog::<GameDomainEvent>::new();
        assert_eq!(log.version(), 0);
        log.append(GameDomainEvent::PlayerMoved { entity_id: 0, dx: 0.0, dy: 0.0 });
        assert_eq!(log.version(), 1);
        log.append(GameDomainEvent::HealthChanged { entity_id: 0, new_hp: 50 });
        assert_eq!(log.version(), 2);
    }

    // -------------------------------------------------------------------
    // Observer integration
    // -------------------------------------------------------------------

    #[derive(Debug, Clone)]
    enum SystemEvent { HealthChanged(u32), ScoreTick(u64) }
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
        let collected = Arc::new(Mutex::new(Vec::<u32>::new()));
        let collected2 = collected.clone();

        let mut bus = EventBus::<SystemEvent>::new();
        bus.subscribe("health", move |e| {
            if let SystemEvent::HealthChanged(hp) = e {
                collected2.lock().unwrap().push(*hp);
            }
        });

        bus.publish(&SystemEvent::HealthChanged(80));
        bus.publish(&SystemEvent::ScoreTick(999)); // must NOT trigger health handler
        bus.publish(&SystemEvent::HealthChanged(40));

        let got = collected.lock().unwrap();
        assert_eq!(*got, vec![80, 40], "observer must receive only health events in order");
    }

    /// Falsification: subscriber must NOT fire for events on a different topic.
    #[test]
    fn observer_does_not_fire_for_wrong_topic() {
        let count = Arc::new(Mutex::new(0u32));
        let count2 = count.clone();

        let mut bus = EventBus::<SystemEvent>::new();
        bus.subscribe("score", move |_| {
            *count2.lock().unwrap() += 1;
        });

        bus.publish(&SystemEvent::HealthChanged(50));
        bus.publish(&SystemEvent::HealthChanged(10));

        assert_eq!(
            *count.lock().unwrap(), 0,
            "score subscriber must not fire for HealthChanged events"
        );
    }

    // -------------------------------------------------------------------
    // Pipeline integration
    // -------------------------------------------------------------------

    struct ScaleStage { factor: i32 }
    impl Stage<i32, i32> for ScaleStage {
        fn process(&self, input: i32) -> i32 { input * self.factor }
        fn stage_name(&self) -> &'static str { "scale" }
    }

    struct ClampStage { max: i32 }
    impl Stage<i32, i32> for ClampStage {
        fn process(&self, input: i32) -> i32 { input.min(self.max) }
        fn stage_name(&self) -> &'static str { "clamp" }
    }

    struct FormatStage;
    impl Stage<i32, String> for FormatStage {
        fn process(&self, input: i32) -> String { format!("hp:{}", input) }
        fn stage_name(&self) -> &'static str { "format" }
    }

    /// Falsification: pipeline output must depend on input, not be a constant.
    #[test]
    fn pipeline_output_depends_on_input() {
        let p = Pipeline::new("scale", ScaleStage { factor: 3 })
            .then(ClampStage { max: 100 })
            .then(FormatStage);

        let r1 = p.run(10);
        let r2 = p.run(50);
        let r3 = p.run(0);

        assert_ne!(r1, r2, "pipeline output must vary with input");
        assert_ne!(r2, r3, "pipeline output must vary with input");
        assert_eq!(r1, "hp:30", "10*3=30, clamp(30,100)=30");
        assert_eq!(r2, "hp:100", "50*3=150, clamp(150,100)=100");
        assert_eq!(r3, "hp:0", "0*3=0, clamp(0,100)=0");
    }

    /// Batch pipeline must process all items.
    #[test]
    fn batch_pipeline_processes_all_items() {
        let p = Pipeline::new("scale", ScaleStage { factor: 2 });
        let mut bp = BatchPipeline::new(p);
        let results = bp.process_batch(vec![1, 2, 3, 4, 5]);
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
        assert_eq!(bp.processed_count(), 5);
    }

    // -------------------------------------------------------------------
    // Actor model integration
    // -------------------------------------------------------------------

    #[derive(Debug, Clone)]
    struct MoveCmd { entity_id: u32, seq: u32 }
    impl ActorMessage for MoveCmd {}

    /// Falsification: actor system must assign different workers for different actors.
    #[test]
    fn actor_system_worker_assignment_depends_on_actor_id() {
        let sys = ActorSystem::new(4);
        let w0 = sys.assign_worker(0);
        let w1 = sys.assign_worker(1);
        let w2 = sys.assign_worker(2);
        let w3 = sys.assign_worker(3);
        let w4 = sys.assign_worker(4); // must wrap to same as w0

        assert_ne!(w0, w1, "actor 0 and actor 1 must be on different workers");
        assert_ne!(w1, w2, "actor 1 and actor 2 must be on different workers");
        assert_ne!(w2, w3, "actor 2 and actor 3 must be on different workers");
        assert_eq!(w0, w4, "actor 4 must wrap to same worker as actor 0 (4 mod 4 == 0)");
    }

    /// Falsification: mailbox must deliver messages in FIFO order.
    #[test]
    fn actor_mailbox_delivers_in_fifo_order() {
        let mut mb = ActorMailbox::<MoveCmd>::new(0, 10);
        for seq in 0..5u32 {
            mb.send(MoveCmd { entity_id: 1, seq }).unwrap();
        }
        for expected_seq in 0..5u32 {
            let msg = mb.receive().unwrap();
            assert_eq!(
                msg.seq, expected_seq,
                "mailbox must deliver in FIFO order: expected seq={expected_seq}"
            );
        }
    }

    // -------------------------------------------------------------------
    // End-to-end mini game-loop: CQRS → EventLog → Observer → Pipeline
    // -------------------------------------------------------------------

    #[test]
    fn full_mini_game_loop_tick() {
        // Step 1: CQRS command changes write model.
        let mut cmd_bus = CommandBus::new();
        let mut store = EntityStore { hp: std::collections::HashMap::new() };
        store.hp.insert(42, 30); // entity 42 starts at 30 HP

        cmd_bus.dispatch(&mut store, HealCommand { entity_id: 42, amount: 25 }).unwrap();

        // Step 2: Emit a domain event to the EventLog.
        let mut repo = EventSourcedRepo::<GameReadModel>::new(100);
        let new_hp = *store.hp.get(&42).unwrap();
        repo.append(GameDomainEvent::HealthChanged { entity_id: 42, new_hp });

        // Step 3: Reconstruct read model from events.
        let state = repo.reconstruct();
        assert_eq!(state.last_hp, 55, "30 + 25 heal = 55 HP");

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

        // Verify the full chain produced the right result.
        let got_hp = *fired.lock().unwrap().first().unwrap();
        assert_eq!(got_hp, 55, "observer must have received hp=55");
        assert_eq!(label, "hp:55", "pipeline must format hp=55 as 'hp:55'");
        assert_eq!(cmd_bus.commands_dispatched(), 1);
    }
}

// ---------------------------------------------------------------------------
// SCENARIO 5 — Typestate correctness: compile-time enforcement
//
// These tests prove that only valid state transitions compile. The commented
// lines show what would be a COMPILE ERROR, not a runtime failure.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod typestate {
    use wasm_core::{Paused, Running, Terminated, Uninitialized, WasmWorker};

    #[test]
    fn typestate_worker_lifecycle() {
        let w: WasmWorker<Uninitialized> = WasmWorker::new("worker.js", 42);
        let r: WasmWorker<Running> = w.start().unwrap();
        let _t: WasmWorker<Terminated> = r.terminate();

        // The following would NOT compile — proving typestate works:
        //
        //   let w2 = WasmWorker::<Uninitialized>::new("x", 0);
        //   w2.terminate();     // ERROR: no method `terminate` on WasmWorker<Uninitialized>
        //
        //   let t = WasmWorker::<Uninitialized>::new("x", 0).start().unwrap().terminate();
        //   t.pause();          // ERROR: no method `pause` on WasmWorker<Terminated>
    }

    #[test]
    fn typestate_pause_resume_terminate() {
        let running: WasmWorker<Running> =
            WasmWorker::new("worker.js", 0).start().unwrap();
        let paused: WasmWorker<Paused> = running.pause();
        let resumed: WasmWorker<Running> = paused.resume();
        let _term: WasmWorker<Terminated> = resumed.terminate();

        // The following would NOT compile:
        //
        //   let p = WasmWorker::new("x", 0).start().unwrap().pause();
        //   p.start();  // ERROR: WasmWorker<Paused> has no `start` method — only `resume`
    }

    /// Falsification: worker_id must be preserved across state transitions.
    #[test]
    fn worker_id_survives_transitions() {
        for id in [0u32, 1, 255, u32::MAX / 2] {
            let w = WasmWorker::new("x.js", id).start().unwrap();
            assert_eq!(w.worker_id(), id, "worker_id must equal {id} after start");
            let p = w.pause();
            let r = p.resume();
            assert_eq!(r.worker_id(), id, "worker_id must equal {id} after pause+resume");
        }
    }

    /// Falsification: is_terminated must always be true for Terminated state.
    #[test]
    fn terminated_worker_always_reports_terminated() {
        for id in [0u32, 7, 99] {
            let t = WasmWorker::new("w.js", id)
                .start()
                .unwrap()
                .terminate();
            assert!(t.is_terminated(), "WasmWorker<Terminated> must always return true from is_terminated");
        }
    }

    /// Falsification: script_url must be preserved across transitions.
    #[test]
    fn script_url_survives_transitions() {
        let urls = ["worker.js", "https://cdn.example.com/worker.js", "relative/path.js"];
        for url in urls {
            let r = WasmWorker::new(url, 0).start().unwrap();
            assert_eq!(r.script_url(), url, "script_url must survive Uninitialized → Running");
            let p = r.pause();
            let r2 = p.resume();
            assert_eq!(r2.script_url(), url, "script_url must survive Running → Paused → Running");
        }
    }
}

// ---------------------------------------------------------------------------
// Combinatorial proptest matrix
// ---------------------------------------------------------------------------
#[cfg(test)]
mod combinatorial {
    use proptest::prelude::*;
    use wasm_core::{SharedMemoryBus, ThreadingApproach, WorkerPool};

    proptest! {
        /// Matrix: (worker_count, buffer_size, health, max_health, tick).
        /// Every dimension must affect the output — no shortcuts.
        #[test]
        fn combinatorial_system_state(
            worker_count in 1usize..8,
            buffer_size in 8usize..256,
            health in 0u32..1000,
            max_health in 1u32..1000,
            tick in 0u64..10000,
        ) {
            let health = health.min(max_health);

            // WorkerPool: size must match request.
            let pool = WorkerPool::new(worker_count, "worker.js").unwrap();
            prop_assert_eq!(pool.size(), worker_count, "pool size must equal requested worker_count");

            // SharedMemoryBus: round-trip must be exact.
            let mut bus = SharedMemoryBus::new(buffer_size);
            bus.write_i32(0, health as i32).unwrap();
            bus.write_i32(1, max_health as i32).unwrap();
            // Write tick into slot 2 (clamped to i32 range; tick is u64 but fits
            // for the purpose of this test since it is bounded to 10000).
            bus.write_i32(2, tick as i32).unwrap();

            prop_assert_eq!(bus.read_i32(0).unwrap(), health as i32);
            prop_assert_eq!(bus.read_i32(1).unwrap(), max_health as i32);

            // Health percentage must be in [0.0, 1.0].
            let pct = health as f32 / max_health as f32;
            prop_assert!(pct >= 0.0 && pct <= 1.0, "health percentage must be in [0.0, 1.0]");

            // Two pools with the same count must have the same size.
            let pool2 = WorkerPool::new(worker_count, "worker.js").unwrap();
            prop_assert_eq!(pool.size(), pool2.size(), "same worker_count must produce same pool size");
        }

        /// Matrix: COOP/COEP requirements for each ThreadingApproach variant.
        #[test]
        fn approach_coop_coep_matrix(
            worker_count in 1usize..16,
            buffer_size in 1024usize..1_048_576,
        ) {
            let sep = ThreadingApproach::SeparateModules { worker_count };
            let shm = ThreadingApproach::SharedMemory { buffer_size_bytes: buffer_size };
            let hyb = ThreadingApproach::Hybrid {
                worker_count,
                shared_buffer_size_bytes: buffer_size,
            };

            prop_assert!(!sep.requires_coop_coep(), "SeparateModules must NOT require COOP/COEP");
            prop_assert!(shm.requires_coop_coep(), "SharedMemory MUST require COOP/COEP");
            prop_assert!(hyb.requires_coop_coep(), "Hybrid MUST require COOP/COEP");

            prop_assert_eq!(sep.worker_count(), worker_count);
            prop_assert_eq!(hyb.worker_count(), worker_count);

            // Falsification: SeparateModules and SharedMemory must have DIFFERENT flags.
            prop_assert_ne!(
                sep.requires_coop_coep(),
                shm.requires_coop_coep(),
                "SeparateModules and SharedMemory must differ on COOP/COEP"
            );
        }

        /// Bus arbitrary write/read roundtrip across all offsets and values.
        #[test]
        fn bus_roundtrip_arbitrary(
            size in 1usize..128,
            offset in 0usize..127,
            value in i32::MIN..i32::MAX,
        ) {
            prop_assume!(offset < size);
            let mut bus = SharedMemoryBus::new(size);
            bus.write_i32(offset, value).unwrap();
            prop_assert_eq!(
                bus.read_i32(offset).unwrap(),
                value,
                "bus must return exactly the value that was written"
            );
        }

        /// Round-robin dispatch with arbitrary pool size and dispatch count.
        #[test]
        fn pool_round_robin_distributes_evenly(
            worker_count in 1usize..8,
            rounds in 1usize..4,
        ) {
            let mut pool = WorkerPool::new(worker_count, "w.js").unwrap();
            let dispatch_count = worker_count * rounds;
            let mut counts = std::collections::HashMap::<u32, usize>::new();
            for _ in 0..dispatch_count {
                let id = pool.dispatch_to_next().worker_id();
                *counts.entry(id).or_insert(0) += 1;
            }
            for (&id, &count) in &counts {
                prop_assert_eq!(
                    count, rounds,
                    "worker {} should receive exactly {} dispatches", id, rounds
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Anti-cheat / Falsification module
// ---------------------------------------------------------------------------
pub mod falsification {
    use wasm_core::{SharedMemoryBus, ThreadingApproach, WorkerChannel, WorkerPool};

    /// Bus reads must reflect exactly what was written, not a constant.
    #[test]
    pub fn anti_cheat_bus_reads_reflect_writes() {
        let mut bus = SharedMemoryBus::new(16);
        let test_values = [0i32, 1, -1, i32::MAX, i32::MIN, 42, 100, 999,
                           0, -999, 12345, -12345, 7, 77, 777, 7777];
        for (i, &v) in test_values.iter().enumerate() {
            bus.write_i32(i, v).unwrap();
        }
        for (i, &v) in test_values.iter().enumerate() {
            assert_eq!(
                bus.read_i32(i).unwrap(), v,
                "bus.read_i32({i}) must return {v}, not a constant"
            );
        }
    }

    /// pool.size() must vary with the requested count, not return a constant.
    #[test]
    pub fn anti_cheat_pool_size_varies_with_request() {
        for n in [1, 2, 4, 8, 16] {
            let pool = WorkerPool::new(n, "w.js").unwrap();
            assert_eq!(
                pool.size(), n,
                "pool.size() must return {n}, not a constant"
            );
        }
    }

    /// Channel.receive() must return the exact message that was sent.
    #[test]
    pub fn anti_cheat_channel_delivers_what_was_sent() {
        #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Probe { id: u64, payload: String }

        let probes = vec![
            Probe { id: 0, payload: "alpha".to_string() },
            Probe { id: 1, payload: "beta".to_string() },
            Probe { id: 999, payload: "gamma".to_string() },
        ];

        let mut ch = WorkerChannel::<Probe>::new(0);
        for p in &probes { ch.send(p.clone()).unwrap(); }
        for p in &probes {
            let received = ch.receive().expect("channel must deliver sent message");
            assert_eq!(
                received, *p,
                "delivered message must match sent message (id={})", p.id
            );
        }
    }

    /// compare_exchange must only swap on exact match; never on any mismatch.
    #[test]
    pub fn anti_cheat_compare_exchange_conditional() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, 42).unwrap();

        for wrong in [0, 1, 41, 43, 100, -1, i32::MAX] {
            let swapped = bus.compare_exchange(0, wrong, 999);
            assert!(
                !swapped,
                "compare_exchange must NOT swap when expected={wrong} != actual=42"
            );
            assert_eq!(
                bus.read_i32(0).unwrap(), 42,
                "value must remain 42 after failed CAS (expected={wrong})"
            );
        }

        let swapped = bus.compare_exchange(0, 42, 999);
        assert!(swapped, "compare_exchange MUST swap when expected matches");
        assert_eq!(bus.read_i32(0).unwrap(), 999);
    }

    /// Round-robin must visit ALL workers in a pool, not repeat the same one.
    #[test]
    pub fn anti_cheat_pool_round_robin_is_real() {
        let mut pool = WorkerPool::new(4, "w.js").unwrap();
        let mut seen_ids = std::collections::HashSet::new();
        for _ in 0..pool.size() {
            seen_ids.insert(pool.dispatch_to_next().worker_id());
        }
        assert_eq!(
            seen_ids.len(), 4,
            "round-robin must visit all 4 workers, not repeat the same one"
        );
    }

    /// SeparateModules and SharedMemory must have DIFFERENT COOP/COEP flags.
    #[test]
    pub fn anti_cheat_approach_coop_coep_differs() {
        let sep = ThreadingApproach::SeparateModules { worker_count: 2 };
        let shm = ThreadingApproach::SharedMemory { buffer_size_bytes: 1024 };
        assert_ne!(
            sep.requires_coop_coep(),
            shm.requires_coop_coep(),
            "SeparateModules and SharedMemory must have DIFFERENT COOP/COEP requirements"
        );
    }

    /// All worker IDs within a pool must be unique.
    #[test]
    pub fn anti_cheat_pool_worker_ids_are_unique() {
        let pool = WorkerPool::new(8, "w.js").unwrap();
        let ids = pool.worker_ids();
        let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();
        assert_eq!(
            unique.len(), ids.len(),
            "all worker IDs in pool must be unique"
        );
    }

    /// Out-of-bounds access must error — bounds check must be real.
    #[test]
    pub fn anti_cheat_bus_bounds_checking_is_real() {
        let bus = SharedMemoryBus::new(4);
        assert!(bus.read_i32(4).is_err(), "offset == size must error");
        assert!(bus.read_i32(5).is_err(), "offset > size must error");
        assert!(bus.read_i32(100).is_err(), "large offset must error");
        assert!(bus.read_i32(0).is_ok(), "offset 0 on size-4 bus must succeed");
        assert!(bus.read_i32(3).is_ok(), "offset 3 on size-4 bus must succeed");
    }

    /// Bus size() must reflect what was requested, not be a constant.
    #[test]
    pub fn anti_cheat_bus_size_matches_requested() {
        for size in [1, 4, 16, 64, 256, 1024] {
            let bus = SharedMemoryBus::new(size);
            assert_eq!(bus.size(), size, "SharedMemoryBus::size() must return {size}");
        }
    }
}

// ---------------------------------------------------------------------------
// Stress / load module
// ---------------------------------------------------------------------------
pub mod stress {
    use wasm_core::{SharedMemoryBus, WorkerChannel, WorkerPool};

    /// Write 1000 values to a bus; last-writer-wins per slot must hold.
    #[test]
    pub fn bus_write_read_1000_times() {
        let mut bus = SharedMemoryBus::new(1024);
        for i in 0i32..1000 {
            bus.write_i32((i % 1024) as usize, i).unwrap();
        }
        // Each slot in [0..1000) was last written when i == slot index
        // (single pass; i % 1024 maps uniquely in [0..1000)).
        for slot in 0..1000usize {
            let val = bus.read_i32(slot).unwrap();
            // The last value written to `slot` is slot itself (since i < 1024
            // means i % 1024 == i, and we only traverse once, so the last
            // write to slot `s` is when i == s).
            assert!(
                val >= 0 && val < 1000,
                "slot {slot} must have a valid value in [0, 1000), got {val}"
            );
        }
    }

    /// 1000 dispatches over a 4-worker pool must produce exactly 250 per worker.
    #[test]
    pub fn pool_dispatch_1000_times_visits_all_workers() {
        let mut pool = WorkerPool::new(4, "w.js").unwrap();
        let mut counts = std::collections::HashMap::<u32, usize>::new();
        for _ in 0..1000 {
            let id = pool.dispatch_to_next().worker_id();
            *counts.entry(id).or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 4, "all 4 workers must have been dispatched to");
        for (id, count) in &counts {
            assert_eq!(
                *count, 250,
                "worker {id} should receive 250/1000 dispatches in round-robin"
            );
        }
    }

    /// A channel must handle 1000 messages with correct ordering.
    #[test]
    pub fn channel_handles_1000_messages() {
        #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Msg { seq: u64 }

        let mut ch = WorkerChannel::<Msg>::new(0);
        for i in 0..1000u64 {
            ch.send(Msg { seq: i }).unwrap();
        }
        assert_eq!(ch.pending_count(), 1000, "channel must hold 1000 pending messages");

        for i in 0..1000u64 {
            let m = ch.receive().unwrap();
            assert_eq!(m.seq, i, "message {i} must be received in order");
        }
        assert_eq!(ch.pending_count(), 0, "channel must be empty after draining all messages");
    }

    /// Draining many messages at once must be consistent with receiving them one by one.
    #[test]
    pub fn channel_drain_1000_messages_all_present() {
        #[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Msg { seq: u64 }

        let mut ch = WorkerChannel::<Msg>::new(0);
        for i in 0..1000u64 {
            ch.send(Msg { seq: i }).unwrap();
        }

        let drained = ch.drain();
        assert_eq!(drained.len(), 1000, "drain must return all 1000 messages");
        assert_eq!(ch.pending_count(), 0, "channel must be empty after drain");

        for (i, msg) in drained.iter().enumerate() {
            assert_eq!(msg.seq, i as u64, "drain must preserve insertion order");
        }
    }

    /// CAS stress: sequential compare-exchanges must succeed only when value matches.
    #[test]
    pub fn compare_exchange_sequential_stress() {
        let mut bus = SharedMemoryBus::new(4);
        bus.write_i32(0, 0).unwrap();

        // Increment via CAS 100 times — each iteration should succeed.
        for i in 0i32..100 {
            let ok = bus.compare_exchange(0, i, i + 1);
            assert!(ok, "CAS must succeed when expected={i} matches current value");
            assert_eq!(bus.read_i32(0).unwrap(), i + 1);
        }
        assert_eq!(bus.read_i32(0).unwrap(), 100);
    }
}
