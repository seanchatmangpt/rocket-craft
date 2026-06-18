use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_ui::{InputCommand, MessageBridge, UiToGameMessage};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buf) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn bridge_processes_state_update_json() {
    let log = log();
    log.info("Given a MessageBridge and a valid StateUpdate JSON payload");
    let mut bridge = MessageBridge::new();
    let json = r#"{"StateUpdate":{"tick":5,"entity_count":3,"player_health":80,"player_health_max":100,"player_score":2000}}"#;

    log.info("When process() is called");
    let hud = bridge.process(json).unwrap();

    log.info("Then HudData fields match the payload");
    assert_eq!(hud.game_tick, 5);
    assert_eq!(hud.player_health, 80);
    assert_eq!(hud.score, 2000);

    log.info("And messages_processed is 1 and last_tick is 5");
    assert_eq!(bridge.messages_processed, 1);
    assert_eq!(bridge.last_tick, 5);
}

#[test]
fn bridge_rejects_invalid_json() {
    let log = log();
    log.info("Given a MessageBridge and a non-JSON string");
    let mut bridge = MessageBridge::new();

    log.info("When process() is called with 'not json at all'");
    let result = bridge.process("not json at all");

    log.info("Then None is returned and messages_processed remains 0");
    assert!(result.is_none());
    assert_eq!(bridge.messages_processed, 0);
}

#[test]
fn bridge_last_tick_reflects_message_content() {
    let log = log();
    log.info("Given a MessageBridge and two StateUpdate messages with different ticks");
    let mut bridge = MessageBridge::new();
    let json1 = r#"{"StateUpdate":{"tick":10,"entity_count":1,"player_health":null,"player_health_max":null,"player_score":0}}"#;
    let json2 = r#"{"StateUpdate":{"tick":99,"entity_count":1,"player_health":null,"player_health_max":null,"player_score":0}}"#;

    log.info("When process() is called with tick=10");
    bridge.process(json1);
    assert_eq!(bridge.last_tick, 10);

    log.info("When process() is called with tick=99");
    bridge.process(json2);

    log.info("Then last_tick reflects the most recent message — 99, not 10");
    assert_eq!(bridge.last_tick, 99);
    assert_ne!(10u64, 99u64);
}

#[test]
fn bridge_serializes_move_input_command() {
    let log = log();
    log.info("Given a MessageBridge and a Move InputCommand");
    let bridge = MessageBridge::new();
    let msg = UiToGameMessage::Input(InputCommand::Move {
        entity: 1,
        dx: 3.0,
        dy: -2.0,
    });

    log.info("When serialize_to_game() is called");
    let json = bridge.serialize_to_game(&msg).unwrap();

    log.info("Then the JSON contains 'Move'");
    let parsed: UiToGameMessage = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, UiToGameMessage::Input(InputCommand::Move { .. })));
}

#[test]
fn bridge_serializes_pause_message() {
    let log = log();
    log.info("Given a MessageBridge and a Pause message");
    let bridge = MessageBridge::new();

    log.info("When serialize_to_game() is called");
    let json = bridge.serialize_to_game(&UiToGameMessage::Pause).unwrap();

    log.info("Then the JSON contains 'Pause'");
    let parsed: UiToGameMessage = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, UiToGameMessage::Pause));
}

#[test]
fn bridge_different_messages_produce_different_json() {
    let log = log();
    log.info("Given a Move command and a Pause message");
    let bridge = MessageBridge::new();
    let move_json = bridge
        .serialize_to_game(&UiToGameMessage::Input(InputCommand::Move {
            entity: 1,
            dx: 3.0,
            dy: -2.0,
        }))
        .unwrap();
    let pause_json = bridge
        .serialize_to_game(&UiToGameMessage::Pause)
        .unwrap();

    log.info("When both are serialized");
    log.info("Then the resulting JSON strings must differ");
    assert_ne!(move_json, pause_json, "different messages must produce different JSON");
}

#[test]
fn bridge_entity_count_flows_through_to_hud() {
    let log = log();
    log.info("Given a StateUpdate with entity_count=7");
    let mut bridge = MessageBridge::new();
    let json = r#"{"StateUpdate":{"tick":1,"entity_count":7,"player_health":100,"player_health_max":100,"player_score":0}}"#;

    log.info("When process() is called");
    let hud = bridge.process(json).unwrap();

    log.info("Then hud.entity_count is 7");
    assert_eq!(hud.entity_count, 7);
}

#[test]
fn bridge_game_over_message_is_processed() {
    let log = log();
    log.info("Given a GameOver JSON message");
    let mut bridge = MessageBridge::new();
    let json = r#"{"GameOver":{"winner_score":9999,"total_ticks":500}}"#;

    log.info("When process() is called");
    let hud = bridge.process(json).unwrap();

    log.info("Then hud.score equals winner_score and game_tick equals total_ticks");
    assert_eq!(hud.score, 9999);
    assert_eq!(hud.game_tick, 500);
    assert_eq!(bridge.messages_processed, 1);
}

#[test]
fn bridge_health_defaults_to_zero_when_null() {
    let log = log();
    log.info("Given a StateUpdate with null player_health and null player_health_max");
    let mut bridge = MessageBridge::new();
    let json = r#"{"StateUpdate":{"tick":1,"entity_count":0,"player_health":null,"player_health_max":null,"player_score":0}}"#;

    log.info("When process() is called");
    let hud = bridge.process(json).unwrap();

    log.info("Then player_health defaults to 0 and player_health_max defaults to 100");
    assert_eq!(hud.player_health, 0);
    assert_eq!(hud.player_health_max, 100);
}

proptest! {
    #[test]
    fn bridge_message_count_monotonically_increases(n in 1usize..20) {
        let log = log();
        log.info("Given n valid StateUpdate messages");
        let mut bridge = MessageBridge::new();

        log.info("When all n messages are processed");
        for i in 0..n {
            let json = format!(
                r#"{{"StateUpdate":{{"tick":{},"entity_count":0,"player_health":null,"player_health_max":null,"player_score":0}}}}"#,
                i
            );
            bridge.process(&json);
        }

        log.info("Then messages_processed equals n");
        prop_assert_eq!(bridge.messages_processed, n as u64);
    }
}

#[test]
fn entity_moved_updates_entity_count() {
    use wasm_ui::message_bridge::{GameToUiMessage, MessageBridge};
    let mut bridge = MessageBridge::new();
    let moved = GameToUiMessage::EntityMoved { entity_id: 1, x: 5.0, y: 3.0 };
    let json = serde_json::to_string(&moved).unwrap();
    let result = bridge.process(&json);
    assert!(result.is_some(), "EntityMoved must produce Some(HudData)");
    let hud = result.unwrap();
    // Falsification: entity_count increased to 1
    assert_eq!(hud.entity_count, 1);
    assert_ne!(hud.entity_count, 0);
}

#[test]
fn entity_died_decrements_entity_count() {
    use wasm_ui::message_bridge::{GameToUiMessage, MessageBridge};
    let mut bridge = MessageBridge::new();

    // First add an entity
    let moved = GameToUiMessage::EntityMoved { entity_id: 7, x: 1.0, y: 1.0 };
    bridge.process(&serde_json::to_string(&moved).unwrap());

    // Now kill it
    let died = GameToUiMessage::EntityDied { entity_id: 7 };
    let json = serde_json::to_string(&died).unwrap();
    let result = bridge.process(&json);
    assert!(result.is_some());
    let hud = result.unwrap();
    // Falsification: entity was removed, count is 0
    assert_eq!(hud.entity_count, 0);
}

#[test]
fn two_entity_moved_produces_count_of_two() {
    use wasm_ui::message_bridge::{GameToUiMessage, MessageBridge};
    let mut bridge = MessageBridge::new();
    let m1 = GameToUiMessage::EntityMoved { entity_id: 1, x: 0.0, y: 0.0 };
    let m2 = GameToUiMessage::EntityMoved { entity_id: 2, x: 5.0, y: 5.0 };
    bridge.process(&serde_json::to_string(&m1).unwrap());
    let result = bridge.process(&serde_json::to_string(&m2).unwrap());
    assert_eq!(result.unwrap().entity_count, 2);
}

#[test]
fn test_entity_moved_and_died_preserves_hud_state() {
    use wasm_ui::message_bridge::{GameToUiMessage, MessageBridge};
    let mut bridge = MessageBridge::new();
    
    // Set baseline state
    let state_update = r#"{"StateUpdate":{"tick":10,"entity_count":1,"player_health":85,"player_health_max":120,"player_score":450}}"#;
    let hud1 = bridge.process(state_update).unwrap();
    assert_eq!(hud1.player_health, 85);
    assert_eq!(hud1.player_health_max, 120);
    assert_eq!(hud1.score, 450);

    // Process EntityMoved
    let moved = GameToUiMessage::EntityMoved { entity_id: 1, x: 10.0, y: 20.0 };
    let hud2 = bridge.process(&serde_json::to_string(&moved).unwrap()).unwrap();
    assert_eq!(hud2.player_health, 85);
    assert_eq!(hud2.player_health_max, 120);
    assert_eq!(hud2.score, 450);

    // Process EntityDied
    let died = GameToUiMessage::EntityDied { entity_id: 1 };
    let hud3 = bridge.process(&serde_json::to_string(&died).unwrap()).unwrap();
    assert_eq!(hud3.player_health, 85);
    assert_eq!(hud3.player_health_max, 120);
    assert_eq!(hud3.score, 450);
}
