use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_ui::HudData;

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buf) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

fn make_hud(player_health: u32, player_health_max: u32) -> HudData {
    HudData {
        player_health,
        player_health_max,
        score: 0,
        entity_count: 0,
        game_tick: 0,
        fps: 0.0,
        messages_per_second: 0.0,
    }
}

#[test]
fn hud_color_is_green_at_full_health() {
    let mut log = log();
    log.info("Given a HudData with 100/100 health");
    let hud = make_hud(100, 100);

    log.info("When health_color() is called");
    let color = hud.health_color();

    log.info("Then the color is green (#00ff00)");
    assert_eq!(color, "#00ff00");
}

#[test]
fn hud_color_is_red_at_low_health() {
    let mut log = log();
    log.info("Given a HudData with 10/100 health");
    let hud = make_hud(10, 100);

    log.info("When health_color() is called");
    let color = hud.health_color();

    log.info("Then the color is red (#ff0000)");
    assert_eq!(color, "#ff0000");
}

#[test]
fn hud_color_depends_on_health_level() {
    let mut log = log();
    log.info("Given a full-health HudData and a low-health HudData");
    let full = make_hud(100, 100);
    let low = make_hud(10, 100);

    log.info("When health_color() is called on each");
    log.info("Then the colors must differ by health level");
    assert_ne!(
        full.health_color(),
        low.health_color(),
        "color must differ by health level"
    );
}

#[test]
fn hud_color_is_yellow_in_middle_range() {
    let mut log = log();
    log.info("Given a HudData with 50/100 health (50% — mid tier)");
    let hud = make_hud(50, 100);

    log.info("When health_color() is called");
    let color = hud.health_color();

    log.info("Then the color is yellow (#ffff00)");
    assert_eq!(color, "#ffff00");
}

#[test]
fn hud_critical_health_below_25_percent() {
    let mut log = log();
    log.info("Given a HudData with 20/100 health (20% — below critical threshold)");
    let critical = make_hud(20, 100);

    log.info("When is_critical_health() is called");
    log.info("Then it returns true");
    assert!(critical.is_critical_health());
}

#[test]
fn hud_critical_health_above_25_percent_is_false() {
    let mut log = log();
    log.info("Given a HudData with 80/100 health (above critical threshold)");
    let fine = make_hud(80, 100);

    log.info("When is_critical_health() is called");
    log.info("Then it returns false");
    assert!(!fine.is_critical_health());
}

#[test]
fn hud_score_format_is_right_padded_to_10_chars() {
    let mut log = log();
    log.info("Given a HudData with score=42");
    let hud = HudData {
        player_health: 100,
        player_health_max: 100,
        score: 42,
        entity_count: 0,
        game_tick: 0,
        fps: 0.0,
        messages_per_second: 0.0,
    };

    log.info("When format_score() is called");
    let formatted = hud.format_score();

    log.info("Then the result is right-aligned in a 10-character field");
    assert_eq!(formatted, "        42");
    assert_eq!(formatted.len(), 10);
}

#[test]
fn hud_format_score_large_value() {
    let mut log = log();
    log.info("Given a HudData with a large score");
    let hud = HudData {
        player_health: 100,
        player_health_max: 100,
        score: 9_999_999_999,
        entity_count: 0,
        game_tick: 0,
        fps: 0.0,
        messages_per_second: 0.0,
    };

    log.info("When format_score() is called");
    let formatted = hud.format_score();

    log.info("Then the result contains the score digits");
    assert!(formatted.find("9999999999").is_some());
}

proptest! {
    #[test]
    fn health_percentage_always_in_range(hp in 0u32..10000, max in 1u32..10000) {
        let mut log = log();
        log.info("Given arbitrary hp and max values where hp <= max");
        let hud = make_hud(hp.min(max), max);

        log.info("When health_percentage() is called");
        let pct = hud.health_percentage();

        log.info("Then the result is always in [0.0, 1.0]");
        prop_assert!(pct >= 0.0);
        prop_assert!(pct <= 1.0);
    }

    #[test]
    fn critical_health_is_consistent_with_percentage(hp in 0u32..100, max in 1u32..100) {
        let mut log = log();
        log.info("Given a HudData with arbitrary health values");
        let hp_clamped = hp.min(max);
        let hud = make_hud(hp_clamped, max);

        log.info("When is_critical_health() is checked against the percentage");
        let is_critical = hud.is_critical_health();
        let pct = hud.health_percentage();

        log.info("Then is_critical iff percentage < 0.25");
        prop_assert_eq!(is_critical, pct < 0.25);
    }
}
