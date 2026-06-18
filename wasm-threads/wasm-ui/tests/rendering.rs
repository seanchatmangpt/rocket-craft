use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_ui::{render_frame, DrawCall, HudData, Renderer, TestRenderer};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buf) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn test_renderer_records_draw_calls() {
    let log = log();
    log.info("Given a fresh TestRenderer");
    let mut r = TestRenderer::new();

    log.info("When clear(), draw_health_bar(), draw_score(), and present() are called");
    r.clear();
    r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.75);
    r.draw_score(999);
    r.present();

    log.info("Then frame_count is 1 and call_count is 4");
    assert_eq!(r.frame_count(), 1);
    assert_eq!(r.call_count(), 4);

    log.info("And the calls include Clear and Score(999)");
    assert!(r.has_call(&DrawCall::Clear));
    assert!(r.has_call(&DrawCall::Score(999)));
}

#[test]
fn renderer_draw_calls_reflect_actual_input_not_constant() {
    let log = log();
    log.info("Given two different health percentages");
    let mut r = TestRenderer::new();

    log.info("When draw_health_bar is called with 0.5 then 0.9");
    r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.5);
    r.draw_health_bar(0.0, 0.0, 100.0, 20.0, 0.9);

    log.info("Then the recorded DrawCall percentages must differ — not a constant");
    let bars: Vec<_> = r
        .draw_calls()
        .iter()
        .filter_map(|c| {
            if let DrawCall::HealthBar { percentage, .. } = c {
                Some(*percentage)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(bars.len(), 2);
    assert_ne!(
        bars[0], bars[1],
        "different health must produce different draw calls"
    );
}

#[test]
fn render_frame_produces_correct_call_sequence() {
    let log = log();
    log.info("Given a HudData with known values");
    let mut r = TestRenderer::new();
    let hud = HudData {
        player_health: 75,
        player_health_max: 100,
        score: 500,
        entity_count: 3,
        game_tick: 10,
        fps: 60.0,
        messages_per_second: 30.0,
    };

    log.info("When render_frame() is called");
    render_frame(&mut r, &hud);

    log.info("Then Clear, Score(500), EntityCount(3), and Present are all recorded");
    assert!(r.has_call(&DrawCall::Clear));
    assert!(r.has_call(&DrawCall::Score(500)));
    assert!(r.has_call(&DrawCall::EntityCount(3)));
    assert!(r.has_call(&DrawCall::Present));

    log.info("And frame_count is 1");
    assert_eq!(r.frame_count(), 1);
}

#[test]
fn render_frame_tick_call_records_game_tick() {
    let log = log();
    log.info("Given a HudData with game_tick=42");
    let mut r = TestRenderer::new();
    let hud = HudData {
        player_health: 50,
        player_health_max: 100,
        score: 0,
        entity_count: 0,
        game_tick: 42,
        fps: 0.0,
        messages_per_second: 0.0,
    };

    log.info("When render_frame() is called");
    render_frame(&mut r, &hud);

    log.info("Then DrawCall::Tick(42) is present");
    assert!(r.has_call(&DrawCall::Tick(42)));
}

#[test]
fn test_renderer_call_count_starts_at_zero() {
    let log = log();
    log.info("Given a freshly constructed TestRenderer");
    let r = TestRenderer::new();

    log.info("Then call_count is 0 and frame_count is 0");
    assert_eq!(r.call_count(), 0);
    assert_eq!(r.frame_count(), 0);
}

proptest! {
    #[test]
    fn render_frame_always_calls_clear_and_present(
        hp in 0u32..100,
        score in 0u64..u64::MAX,
        entities in 0usize..1000,
    ) {
        let log = log();
        log.info("Given arbitrary HudData values");
        let mut r = TestRenderer::new();
        let hud = HudData {
            player_health: hp,
            player_health_max: 100,
            score,
            entity_count: entities,
            game_tick: 0,
            fps: 0.0,
            messages_per_second: 0.0,
        };

        log.info("When render_frame() is called");
        render_frame(&mut r, &hud);

        log.info("Then Clear and Present are always present regardless of inputs");
        prop_assert!(r.has_call(&DrawCall::Clear), "must always clear before drawing");
        prop_assert!(r.has_call(&DrawCall::Present), "must always present after drawing");
    }
}

#[test]
fn canvas_renderer_trait_is_implemented_by_test_renderer() {
    use wasm_ui::renderer::{Renderer, TestRenderer};
    let mut r = TestRenderer::new();
    r.clear();
    r.draw_health_bar(0.0, 0.0, 100.0, 10.0, 0.75);
    r.draw_score(1234);
    r.draw_entity_count(5);
    r.draw_tick(99);
    r.present();
    assert_eq!(r.frame_count(), 1);
    // Falsification: frame_count is 1 after one present(), not 0
    assert_ne!(r.frame_count(), 0);
}

#[test]
fn test_renderer_records_health_bar_percentage() {
    use wasm_ui::renderer::{DrawCall, Renderer, TestRenderer};
    let mut r = TestRenderer::new();
    r.draw_health_bar(0.0, 0.0, 100.0, 10.0, 0.5);
    let calls = r.draw_calls();
    let found = calls.iter().any(|c| matches!(c, DrawCall::HealthBar { percentage, .. } if (*percentage - 0.5).abs() < 0.001));
    assert!(found, "health bar with percentage 0.5 should be recorded");
}
