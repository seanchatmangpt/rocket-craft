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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::{Arc, Mutex};

    // ========== ACTOR MODEL ==========

    #[derive(Debug, Clone)]
    struct PingMsg { value: u32 }
    impl ActorMessage for PingMsg {}

    #[test]
    fn mailbox_send_receive_fifo() {
        let mut mb = ActorMailbox::<PingMsg>::new(0, 10);
        mb.send(PingMsg { value: 1 }).unwrap();
        mb.send(PingMsg { value: 2 }).unwrap();
        mb.send(PingMsg { value: 3 }).unwrap();
        assert_eq!(mb.receive().unwrap().value, 1);  // FIFO
        assert_eq!(mb.receive().unwrap().value, 2);
        assert_eq!(mb.receive().unwrap().value, 3);
        assert!(mb.receive().is_none());
    }

    #[test]
    fn mailbox_capacity_enforced() {
        let mut mb = ActorMailbox::<PingMsg>::new(0, 2);
        mb.send(PingMsg { value: 1 }).unwrap();
        mb.send(PingMsg { value: 2 }).unwrap();
        assert!(mb.send(PingMsg { value: 3 }).is_err());
        assert!(mb.is_full());
    }

    // Falsification: worker assignment depends on actor_id
    #[test]
    fn actor_system_assigns_different_workers_for_different_ids() {
        let sys = ActorSystem::new(4);
        let w0 = sys.assign_worker(0);
        let w1 = sys.assign_worker(1);
        let w4 = sys.assign_worker(4);
        assert_ne!(w0, w1, "different actors should map to different workers");
        assert_eq!(w0, w4, "actor_id mod worker_count should cycle");
    }

    proptest! {
        #[test]
        fn mailbox_pending_count_matches_sent(n in 1usize..10) {
            let mut mb = ActorMailbox::<PingMsg>::new(0, 100);
            for i in 0..n { mb.send(PingMsg { value: i as u32 }).unwrap(); }
            prop_assert_eq!(mb.pending(), n);
        }

        #[test]
        fn actor_worker_assignment_is_deterministic(actor_id in 0u32..1000, workers in 1usize..16) {
            let sys = ActorSystem::new(workers);
            let w1 = sys.assign_worker(actor_id);
            let w2 = sys.assign_worker(actor_id);
            prop_assert_eq!(w1, w2, "same actor_id must always map to same worker");
            prop_assert!(w1 < workers, "worker index must be within range");
        }
    }

    // ========== EVENT SOURCING ==========

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    enum GameEvent {
        PlayerSpawned { id: u32, x: f32, y: f32 },
        PlayerMoved { id: u32, dx: f32, dy: f32 },
        PlayerDied { id: u32 },
    }

    impl DomainEvent for GameEvent {
        fn event_type(&self) -> &'static str {
            match self { Self::PlayerSpawned { .. } => "PlayerSpawned", Self::PlayerMoved { .. } => "PlayerMoved", Self::PlayerDied { .. } => "PlayerDied" }
        }
        fn sequence(&self) -> u64 { 0 }
    }

    #[derive(Debug, Clone, Default)]
    struct GameAggregate { player_count: u32, total_moves: u32 }
    impl Aggregate for GameAggregate {
        type Event = GameEvent;
        fn apply(&mut self, event: &GameEvent) {
            match event {
                GameEvent::PlayerSpawned { .. } => self.player_count += 1,
                GameEvent::PlayerMoved { .. } => self.total_moves += 1,
                GameEvent::PlayerDied { .. } => { if self.player_count > 0 { self.player_count -= 1; } }
            }
        }
        fn version(&self) -> u64 { 0 }
    }

    #[test]
    fn event_sourcing_reconstruct_from_events() {
        let mut repo = EventSourcedRepo::<GameAggregate>::new(100);
        repo.append(GameEvent::PlayerSpawned { id: 1, x: 0.0, y: 0.0 });
        repo.append(GameEvent::PlayerSpawned { id: 2, x: 10.0, y: 0.0 });
        repo.append(GameEvent::PlayerMoved { id: 1, dx: 1.0, dy: 0.0 });
        repo.append(GameEvent::PlayerDied { id: 2 });
        let state = repo.reconstruct();
        assert_eq!(state.player_count, 1);
        assert_eq!(state.total_moves, 1);
    }

    // Falsification: reconstruction must depend on events, not return default
    #[test]
    fn reconstruction_reflects_all_events() {
        let mut repo = EventSourcedRepo::<GameAggregate>::new(100);
        let empty = repo.reconstruct();
        assert_eq!(empty.player_count, 0);  // default is zero
        repo.append(GameEvent::PlayerSpawned { id: 1, x: 0.0, y: 0.0 });
        let after = repo.reconstruct();
        assert_ne!(empty.player_count, after.player_count, "reconstruction must change after events");
    }

    #[test]
    fn snapshot_reduces_replay_work() {
        let mut repo = EventSourcedRepo::<GameAggregate>::new(3); // snapshot every 3
        for i in 0..6u32 {
            repo.append(GameEvent::PlayerSpawned { id: i, x: 0.0, y: 0.0 });
        }
        assert!(repo.has_snapshot());
        let state = repo.reconstruct();
        assert_eq!(state.player_count, 6);
    }

    proptest! {
        #[test]
        fn event_log_version_monotonically_increases(n in 1usize..50) {
            let mut log = EventLog::<GameEvent>::new();
            for i in 0..n {
                log.append(GameEvent::PlayerSpawned { id: i as u32, x: 0.0, y: 0.0 });
                prop_assert_eq!(log.version(), (i + 1) as u64);
            }
        }

        #[test]
        fn reconstruction_is_deterministic(spawns in 1u32..20, moves in 0u32..20) {
            let mut repo = EventSourcedRepo::<GameAggregate>::new(100);
            for i in 0..spawns { repo.append(GameEvent::PlayerSpawned { id: i, x: 0.0, y: 0.0 }); }
            for i in 0..moves { repo.append(GameEvent::PlayerMoved { id: 0, dx: i as f32, dy: 0.0 }); }
            let s1 = repo.reconstruct();
            let s2 = repo.reconstruct();
            prop_assert_eq!(s1.player_count, s2.player_count);
            prop_assert_eq!(s1.total_moves, s2.total_moves);
        }
    }

    // ========== CQRS ==========

    #[derive(Debug)]
    struct DamageCommand { target_id: u32, amount: u32 }
    #[derive(Debug)]
    struct CqrsError(String);
    impl std::fmt::Display for CqrsError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.0) } }

    impl Command for DamageCommand {
        type Error = CqrsError;
        fn command_name(&self) -> &'static str { "DamageCommand" }
    }

    struct HealthWriteModel { health: std::collections::HashMap<u32, u32> }
    impl CommandHandler<DamageCommand> for HealthWriteModel {
        fn handle(&mut self, cmd: DamageCommand) -> Result<(), CqrsError> {
            let hp = self.health.entry(cmd.target_id).or_insert(100);
            *hp = hp.saturating_sub(cmd.amount);
            Ok(())
        }
    }

    struct HealthQuery { target_id: u32 }
    impl Query for HealthQuery {
        type Result = u32;
        fn query_name(&self) -> &'static str { "HealthQuery" }
    }

    impl QueryHandler<HealthQuery> for HealthWriteModel {
        fn handle(&self, query: &HealthQuery) -> u32 {
            *self.health.get(&query.target_id).unwrap_or(&100)
        }
    }

    #[test]
    fn cqrs_command_then_query() {
        let mut bus = CommandBus::new();
        let mut qbus = QueryBus::new();
        let mut model = HealthWriteModel { health: std::collections::HashMap::new() };
        bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 30 }).unwrap();
        let hp = qbus.ask(&model, &HealthQuery { target_id: 1 });
        assert_eq!(hp, 70);
    }

    // Falsification: query result depends on prior commands
    #[test]
    fn query_result_changes_after_command() {
        let mut bus = CommandBus::new();
        let mut qbus = QueryBus::new();
        let mut model = HealthWriteModel { health: std::collections::HashMap::new() };
        let before = qbus.ask(&model, &HealthQuery { target_id: 1 });
        bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 50 }).unwrap();
        let after = qbus.ask(&model, &HealthQuery { target_id: 1 });
        assert_ne!(before, after, "query must reflect state changes from commands");
    }

    #[test]
    fn command_bus_counts_dispatches() {
        let mut bus = CommandBus::new();
        let mut model = HealthWriteModel { health: std::collections::HashMap::new() };
        for _ in 0..5 {
            bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 1 }).unwrap();
        }
        assert_eq!(bus.commands_dispatched(), 5);
        assert_eq!(bus.commands_failed(), 0);
        assert_eq!(bus.success_rate(), 1.0);
    }

    // ========== OBSERVER ==========

    #[derive(Debug, Clone)]
    enum UiEvent { ButtonClicked(String), HealthChanged(u32), ScoreChanged(u64) }
    impl EventType for UiEvent {
        fn topic(&self) -> &'static str {
            match self { Self::ButtonClicked(_) => "button", Self::HealthChanged(_) => "health", Self::ScoreChanged(_) => "score" }
        }
    }

    #[test]
    fn observer_receives_subscribed_events() {
        let received = Arc::new(Mutex::new(Vec::<u32>::new()));
        let received2 = received.clone();
        let mut bus = EventBus::<UiEvent>::new();
        bus.subscribe("health", move |e| {
            if let UiEvent::HealthChanged(hp) = e { received2.lock().unwrap().push(*hp); }
        });
        bus.publish(&UiEvent::HealthChanged(75));
        bus.publish(&UiEvent::HealthChanged(50));
        bus.publish(&UiEvent::ButtonClicked("start".to_string())); // should NOT trigger health subscriber
        let v = received.lock().unwrap();
        assert_eq!(*v, vec![75, 50]);
    }

    // Falsification: subscriber only fires for its topic
    #[test]
    fn observer_does_not_fire_for_wrong_topic() {
        let count = Arc::new(Mutex::new(0u32));
        let count2 = count.clone();
        let mut bus = EventBus::<UiEvent>::new();
        bus.subscribe("score", move |_| { *count2.lock().unwrap() += 1; });
        bus.publish(&UiEvent::HealthChanged(50)); // wrong topic
        bus.publish(&UiEvent::ButtonClicked("x".to_string())); // wrong topic
        assert_eq!(*count.lock().unwrap(), 0, "subscriber must not fire for wrong topic");
    }

    #[test]
    fn wildcard_subscriber_receives_all() {
        let count = Arc::new(Mutex::new(0u32));
        let count2 = count.clone();
        let mut bus = EventBus::<UiEvent>::new();
        bus.subscribe("*", move |_| { *count2.lock().unwrap() += 1; });
        bus.publish(&UiEvent::HealthChanged(50));
        bus.publish(&UiEvent::ScoreChanged(100));
        bus.publish(&UiEvent::ButtonClicked("x".to_string()));
        assert_eq!(*count.lock().unwrap(), 3);
    }

    proptest! {
        #[test]
        fn event_bus_publish_count_monotonic(n in 1usize..50) {
            let mut bus = EventBus::<UiEvent>::new();
            for i in 0..n { bus.publish(&UiEvent::ScoreChanged(i as u64)); }
            prop_assert_eq!(bus.published_count(), n as u64);
        }
    }

    // ========== PIPELINE ==========

    struct DoubleStage;
    impl Stage<i32, i32> for DoubleStage {
        fn process(&self, input: i32) -> i32 { input * 2 }
        fn stage_name(&self) -> &'static str { "double" }
    }

    struct AddTenStage;
    impl Stage<i32, i32> for AddTenStage {
        fn process(&self, input: i32) -> i32 { input + 10 }
        fn stage_name(&self) -> &'static str { "add_ten" }
    }

    struct ToStringStage;
    impl Stage<i32, String> for ToStringStage {
        fn process(&self, input: i32) -> String { format!("value:{}", input) }
        fn stage_name(&self) -> &'static str { "to_string" }
    }

    #[test]
    fn pipeline_single_stage() {
        let p = Pipeline::new("double", DoubleStage);
        assert_eq!(p.run(5), 10);
        assert_eq!(p.stages_count(), 1);
    }

    #[test]
    fn pipeline_composed_stages() {
        let p = Pipeline::new("double", DoubleStage).then(AddTenStage).then(ToStringStage);
        assert_eq!(p.run(5), "value:20");  // (5*2)+10 = 20
        assert_eq!(p.stages_count(), 3);
    }

    // Falsification: pipeline output depends on input
    #[test]
    fn pipeline_output_varies_with_input() {
        let p = Pipeline::new("double", DoubleStage).then(AddTenStage);
        let r1 = p.run(1);
        let r2 = p.run(100);
        assert_ne!(r1, r2, "pipeline output must depend on input");
    }

    #[test]
    fn batch_pipeline_processes_all() {
        let p = Pipeline::new("double", DoubleStage);
        let mut bp = BatchPipeline::new(p);
        let results = bp.process_batch(vec![1, 2, 3, 4, 5]);
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
        assert_eq!(bp.processed_count(), 5);
    }

    proptest! {
        #[test]
        fn pipeline_double_is_always_even(input in 0i32..1000) {
            let p = Pipeline::new("double", DoubleStage);
            prop_assert_eq!(p.run(input) % 2, 0);
        }

        #[test]
        fn pipeline_composition_is_associative_in_result(input in -100i32..100) {
            // (double then add10) == explicit formula
            let p = Pipeline::new("double", DoubleStage).then(AddTenStage);
            let expected = input * 2 + 10;
            prop_assert_eq!(p.run(input), expected);
        }
    }
}
