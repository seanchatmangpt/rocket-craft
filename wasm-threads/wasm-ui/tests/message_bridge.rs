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
    let mut log = log();
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
    let mut log = log();
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
    let mut log = log();
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
    let mut log = log();
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
    assert!(json.contains("Move"));
}

#[test]
fn bridge_serializes_pause_message() {
    let mut log = log();
    log.info("Given a MessageBridge and a Pause message");
    let bridge = MessageBridge::new();

    log.info("When serialize_to_game() is called");
    let json = bridge.serialize_to_game(&UiToGameMessage::Pause).unwrap();

    log.info("Then the JSON contains 'Pause'");
    assert!(json.contains("Pause"));
}

#[test]
fn bridge_different_messages_produce_different_json() {
    let mut log = log();
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
    let mut log = log();
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
    let mut log = log();
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
    let mut log = log();
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
        let mut log = log();
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
