use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_game_logic::{GameToUiMessage, InputCommand, UiToGameMessage};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn game_to_ui_state_update_roundtrips_json() {
    let mut log = log();
    log.info("Given a GameToUiMessage::StateUpdate with tick=42 and player_health=Some(75)");
    let msg = GameToUiMessage::StateUpdate {
        tick: 42,
        entity_count: 5,
        player_health: Some(75),
        player_health_max: Some(100),
        player_score: 1000,
    };

    log.info("When serialized to JSON and back");
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: GameToUiMessage = serde_json::from_str(&json).unwrap();

    log.info("Then the tick and health values are preserved");
    match decoded {
        GameToUiMessage::StateUpdate {
            tick,
            player_health,
            ..
        } => {
            assert_eq!(tick, 42);
            assert_eq!(player_health, Some(75));
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn game_to_ui_game_over_roundtrips_json() {
    let mut log = log();
    log.info("Given a GameToUiMessage::GameOver with winner_score=42000 and total_ticks=1800");
    let msg = GameToUiMessage::GameOver {
        winner_score: 42000,
        total_ticks: 1800,
    };

    log.info("When serialized to JSON and back");
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: GameToUiMessage = serde_json::from_str(&json).unwrap();

    log.info("Then winner_score and total_ticks are preserved");
    match decoded {
        GameToUiMessage::GameOver {
            winner_score,
            total_ticks,
        } => {
            assert_eq!(winner_score, 42000);
            assert_eq!(total_ticks, 1800);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn game_to_ui_entity_moved_roundtrips_json() {
    let mut log = log();
    log.info("Given a GameToUiMessage::EntityMoved for entity 7 at (1.5, -2.0)");
    let msg = GameToUiMessage::EntityMoved {
        entity_id: 7,
        x: 1.5,
        y: -2.0,
    };

    log.info("When serialized to JSON via to_json() and back via from_json()");
    let json = msg.to_json();
    let decoded = GameToUiMessage::from_json(&json).unwrap();

    log.info("Then entity_id and coordinates are preserved");
    match decoded {
        GameToUiMessage::EntityMoved { entity_id, x, y } => {
            assert_eq!(entity_id, 7);
            assert!((x - 1.5).abs() < 0.001);
            assert!((y - (-2.0)).abs() < 0.001);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn game_to_ui_entity_died_roundtrips_json() {
    let mut log = log();
    log.info("Given a GameToUiMessage::EntityDied for entity 3");
    let msg = GameToUiMessage::EntityDied { entity_id: 3 };

    log.info("When serialized and deserialized");
    let json = msg.to_json();
    let decoded = GameToUiMessage::from_json(&json).unwrap();

    log.info("Then the entity_id is preserved");
    match decoded {
        GameToUiMessage::EntityDied { entity_id } => assert_eq!(entity_id, 3),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn ui_to_game_ping_roundtrips_json() {
    let mut log = log();
    log.info("Given a UiToGameMessage::Ping with seq=7");
    let msg = UiToGameMessage::Ping { seq: 7 };

    log.info("When serialized to JSON and back");
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: UiToGameMessage = serde_json::from_str(&json).unwrap();

    log.info("Then seq value 7 is preserved");
    match decoded {
        UiToGameMessage::Ping { seq } => assert_eq!(seq, 7),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn ui_to_game_input_move_roundtrips_json() {
    let mut log = log();
    log.info("Given a UiToGameMessage::Input(Move) for entity 3 with dx=1.5, dy=-0.5");
    let msg = UiToGameMessage::Input(InputCommand::Move {
        entity: 3,
        dx: 1.5,
        dy: -0.5,
    });

    log.info("When serialized via to_json() and decoded via from_json()");
    let json = msg.to_json();
    let decoded = UiToGameMessage::from_json(&json).unwrap();

    log.info("Then entity, dx, and dy are preserved");
    match decoded {
        UiToGameMessage::Input(InputCommand::Move { entity, dx, dy }) => {
            assert_eq!(entity, 3);
            assert!((dx - 1.5).abs() < 0.001);
            assert!((dy - (-0.5)).abs() < 0.001);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn ui_to_game_pause_resume_restart_roundtrip() {
    let mut log = log();
    for (label, msg) in [
        ("Pause", UiToGameMessage::Pause),
        ("Resume", UiToGameMessage::Resume),
        ("Restart", UiToGameMessage::Restart),
    ] {
        log.info(&format!("Given UiToGameMessage::{label}"));
        let json = msg.to_json();
        log.info("When round-tripped through JSON");
        let decoded = UiToGameMessage::from_json(&json);
        log.info("Then deserialization succeeds");
        assert!(decoded.is_some(), "failed to round-trip {label}");
    }
}

#[test]
fn game_to_ui_from_json_returns_none_on_invalid_input() {
    let mut log = log();
    log.info("Given invalid JSON");
    let bad = "not valid json at all {{{}}}";

    log.info("When GameToUiMessage::from_json is called");
    let result = GameToUiMessage::from_json(bad);

    log.info("Then it returns None");
    assert!(result.is_none());
}

#[test]
fn ui_to_game_from_json_returns_none_on_invalid_input() {
    let mut log = log();
    log.info("Given invalid JSON");
    let bad = "not valid json at all {{{}}}";

    log.info("When UiToGameMessage::from_json is called");
    let result = UiToGameMessage::from_json(bad);

    log.info("Then it returns None");
    assert!(result.is_none());
}
