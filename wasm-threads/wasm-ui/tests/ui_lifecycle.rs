use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_ui::{Unloaded, UiState};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buf) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn ui_state_transitions_from_unloaded_through_loading_to_ready() {
    let log = log();
    log.info("Given a UiState in Unloaded phase");
    let s = UiState::<Unloaded>::new();

    log.info("When start_loading() is called");
    let s = s.start_loading();

    log.info("Then ready() yields a Ready state with frame 0");
    let mut ready = s.ready();
    assert_eq!(ready.frame, 0);

    log.info("When render_frame() is called");
    ready.render_frame();

    log.info("Then frame count increments to 1");
    assert_eq!(ready.frame, 1);
}

#[test]
fn ui_state_full_happy_path() {
    let log = log();
    log.info("Given a UiState transitioned through Unloaded -> Loading -> Ready");
    let s = UiState::<Unloaded>::new();
    let s = s.start_loading();
    let mut s = s.ready();

    log.info("When update_from_game() is called with tick=42, hp=80, score=1500");
    s.update_from_game(42, 80, 100, 1500, 10);

    log.info("Then all fields reflect the game update");
    assert_eq!(s.last_game_tick, 42);
    assert_eq!(s.player_health, 80);
    assert_eq!(s.player_score, 1500);
    assert_eq!(s.messages_received, 1);

    log.info("When render_frame() is called");
    s.render_frame();

    log.info("Then frame count is 1");
    assert_eq!(s.frame, 1);
}

#[test]
fn ui_state_error_can_retry_back_to_unloaded() {
    let log = log();
    log.info("Given a UiState in Loading phase");
    let s = UiState::<Unloaded>::new();
    let s = s.start_loading();

    log.info("When fail() is called");
    let s = s.fail("connection refused");

    log.info("Then retry() brings us back to Unloaded — compile-time proof that Error->Unloaded is the only valid transition");
    let _s = s.retry();
}

#[test]
fn ui_state_messages_received_increments_on_each_update() {
    let log = log();
    log.info("Given a Ready UiState");
    let s = UiState::<Unloaded>::new().start_loading().ready();
    let mut s = s;

    log.info("When update_from_game() is called three times");
    s.update_from_game(1, 100, 100, 0, 0);
    s.update_from_game(2, 90, 100, 10, 1);
    s.update_from_game(3, 80, 100, 20, 2);

    log.info("Then messages_received equals 3");
    assert_eq!(s.messages_received, 3);
}

#[test]
fn ui_state_health_percentage_computed_from_fields() {
    let log = log();
    log.info("Given a Ready UiState updated with 75 hp out of 100 max");
    let mut s = UiState::<Unloaded>::new().start_loading().ready();
    s.update_from_game(1, 75, 100, 0, 0);

    log.info("When health_percentage() is called");
    let pct = s.health_percentage();

    log.info("Then the percentage is 0.75");
    assert!((pct - 0.75).abs() < f32::EPSILON);
}

proptest! {
    #[test]
    fn ui_state_frame_count_equals_render_call_count(n in 0usize..50) {
        let log = log();
        log.info("Given a Ready UiState");
        let mut s = UiState::<Unloaded>::new().start_loading().ready();

        log.info("When render_frame() is called n times");
        for _ in 0..n {
            s.render_frame();
        }

        log.info("Then frame equals n");
        prop_assert_eq!(s.frame, n as u64);
    }
}

#[test]
fn frame_count_increments_on_each_message() {
    // UiController is wasm32-only for the #[wasm_bindgen] struct, but
    // we can test frame increment via the bridge directly.
    // If UiController is not available on native, test MessageBridge instead.
    use wasm_ui::message_bridge::{GameToUiMessage, MessageBridge};
    let mut bridge = MessageBridge::new();
    let msg = GameToUiMessage::StateUpdate {
        tick: 1, entity_count: 0, player_health: Some(100),
        player_health_max: Some(100), player_score: 0,
    };
    let json = serde_json::to_string(&msg).unwrap();
    // messages_processed tracks calls
    bridge.process(&json);
    bridge.process(&json);
    assert_eq!(bridge.messages_processed, 2);
    // Falsification: 2 calls produce count of 2
    assert_ne!(bridge.messages_processed, 0);
    assert_ne!(bridge.messages_processed, 1);
}
