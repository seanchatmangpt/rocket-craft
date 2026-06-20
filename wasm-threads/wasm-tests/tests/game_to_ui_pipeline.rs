// Scenario 1 — Game Logic → UI message pipeline
//
// Verifies that the JSON wire format produced by wasm-game-logic is correctly
// consumed by wasm-ui's MessageBridge.  Any mismatch in the protocol enum
// names or field names will be caught here before it reaches the browser.

use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_game_logic::GameToUiMessage as GameMsg;
use wasm_ui::MessageBridge;

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

/// Happy path: a StateUpdate message serialised by wasm-game-logic
/// deserialises correctly inside wasm-ui's MessageBridge.
#[test]
fn game_to_ui_state_update_pipeline() {
    let log = log();
    log.info("Given a StateUpdate message produced by wasm-game-logic");
    let msg = GameMsg::StateUpdate {
        tick: 42,
        entity_count: 5,
        player_health: Some(75),
        player_health_max: Some(100),
        player_score: 1500,
    };
    let json = serde_json::to_string(&msg).unwrap();

    log.info("When the JSON is processed by wasm-ui's MessageBridge");
    let mut bridge = MessageBridge::new();
    let hud = bridge.process(&json).expect("UI must parse game-logic StateUpdate");

    log.info("Then all fields must survive the serialisation round-trip");
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
    let log = log();
    log.info("Given two StateUpdate messages with different tick values");
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

    log.info("When both messages are processed by the bridge");
    let hud1 = bridge.process(&serde_json::to_string(&msg1).unwrap()).unwrap();
    let hud2 = bridge.process(&serde_json::to_string(&msg2).unwrap()).unwrap();

    log.info("Then the resulting HUD states must differ on game_tick");
    assert_ne!(
        hud1.game_tick, hud2.game_tick,
        "different tick values must produce different HUD game_tick fields"
    );
    assert_eq!(bridge.messages_processed, 2);
}

/// Falsification: score must propagate correctly and not be zero by default.
#[test]
fn game_score_reaches_hud() {
    let log = log();
    log.info("Given a variety of score values in StateUpdate messages");
    let scores = [0u64, 1, 500, 999_999, u64::MAX / 2];
    let mut bridge = MessageBridge::new();

    log.info("When each message is processed by the bridge");
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

        log.info(&format!("Then score {score} must reach HUD unmodified"));
        assert_eq!(
            hud.score, score,
            "score {score} must reach HUD unmodified"
        );
    }
}

/// Falsification: health percentage computed by HUD must differ for different health values.
#[test]
fn health_percentage_depends_on_actual_health() {
    let log = log();
    log.info("Given two StateUpdate messages with different health values");
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

    log.info("When both messages are processed");
    let full_hud = bridge.process(&full_json).unwrap();
    let low_hud = bridge.process(&low_json).unwrap();

    let full_pct = full_hud.health_percentage();
    let low_pct = low_hud.health_percentage();

    log.info("Then health percentages must differ and be in correct ranges");
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
    let log = log();
    log.info("Given a GameOver message with winner_score=88888 and total_ticks=360");
    let msg = GameMsg::GameOver {
        winner_score: 88_888,
        total_ticks: 360,
    };
    let json = serde_json::to_string(&msg).unwrap();

    log.info("When the message is processed by the bridge");
    let mut bridge = MessageBridge::new();
    let hud = bridge.process(&json).unwrap();

    log.info("Then score and game_tick must be populated from GameOver fields");
    assert_eq!(hud.score, 88_888, "GameOver winner_score must reach HUD");
    assert_eq!(hud.game_tick, 360, "GameOver total_ticks must reach HUD as game_tick");
}

/// Falsification: bridge must reject garbage JSON and return None.
#[test]
fn bridge_rejects_garbage_json() {
    let log = log();
    log.info("Given a bridge and various forms of invalid JSON input");

    log.info("When garbage strings are fed to the bridge");
    let mut bridge = MessageBridge::new();
    assert!(
        bridge.process("not json").is_none(),
        "invalid JSON must be rejected"
    );
    assert!(
        bridge.process(r#"{"unknown_field": 42}"#).is_none(),
        "unknown message type must be rejected"
    );

    log.info("Then failed parses must not increment the message counter");
    assert_eq!(
        bridge.messages_processed, 0,
        "failed parses must not increment message counter"
    );
}

#[test]
fn ping_serializes_to_game_message() {
    use wasm_ui::message_bridge::MessageBridge;
    let bridge = MessageBridge::new();
    let json = bridge.send_ping(42);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["Ping"]["seq"], 42);
    // Falsification: seq is 42 not 0
    assert_ne!(parsed["Ping"]["seq"], 0);
}

#[test]
fn different_ping_seqs_produce_different_json() {
    use wasm_ui::message_bridge::MessageBridge;
    let bridge = MessageBridge::new();
    let j1 = bridge.send_ping(1);
    let j2 = bridge.send_ping(2);
    assert_ne!(j1, j2);
}
