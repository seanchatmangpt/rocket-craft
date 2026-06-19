#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HudData {
    pub player_health: u32,
    pub player_health_max: u32,
    pub score: u64,
    pub entity_count: usize,
    pub game_tick: u64,
    pub fps: f32,
    pub messages_per_second: f32,
}

impl HudData {
    pub fn health_percentage(&self) -> f32 {
        if self.player_health_max == 0 {
            return 0.0;
        }
        self.player_health as f32 / self.player_health_max as f32
    }

    pub fn health_color(&self) -> &'static str {
        match (self.health_percentage() * 100.0) as u32 {
            67..=100 => "#00ff00", // green
            34..=66 => "#ffff00",  // yellow
            _ => "#ff0000",        // red
        }
    }

    pub fn is_critical_health(&self) -> bool {
        self.health_percentage() < 0.25
    }

    pub fn format_score(&self) -> String {
        format!("{:>10}", self.score)
    }
}

pub fn render_frame<R: super::renderer::Renderer>(renderer: &mut R, hud: &HudData) {
    renderer.clear();
    renderer.draw_health_bar(10.0, 10.0, 200.0, 20.0, hud.health_percentage());
    renderer.draw_score(hud.score);
    renderer.draw_entity_count(hud.entity_count);
    renderer.draw_tick(hud.game_tick);
    renderer.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hud(health: u32, health_max: u32, score: u64) -> HudData {
        HudData { player_health: health, player_health_max: health_max, score,
            entity_count: 0, game_tick: 0, fps: 0.0, messages_per_second: 0.0 }
    }

    #[test]
    fn health_percentage_full() {
        assert!((make_hud(100, 100, 0).health_percentage() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn health_percentage_half() {
        assert!((make_hud(50, 100, 0).health_percentage() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn health_percentage_zero_max_returns_zero() {
        assert_eq!(make_hud(10, 0, 0).health_percentage(), 0.0);
    }

    #[test]
    fn health_color_green_above_66() {
        assert_eq!(make_hud(70, 100, 0).health_color(), "#00ff00");
    }

    #[test]
    fn health_color_yellow_34_to_66() {
        assert_eq!(make_hud(50, 100, 0).health_color(), "#ffff00");
    }

    #[test]
    fn health_color_red_below_34() {
        assert_eq!(make_hud(20, 100, 0).health_color(), "#ff0000");
    }

    #[test]
    fn is_critical_health_below_25_percent() {
        assert!(make_hud(24, 100, 0).is_critical_health());
    }

    #[test]
    fn is_critical_health_false_at_25_percent() {
        assert!(!make_hud(25, 100, 0).is_critical_health());
    }

    #[test]
    fn format_score_right_aligns_to_10_chars() {
        let s = make_hud(0, 100, 42).format_score();
        assert_eq!(s.len(), 10);
        assert!(s.ends_with("42"));
    }

    #[test]
    fn format_score_large_number() {
        let s = make_hud(0, 100, 1_234_567_890).format_score();
        assert!(s.contains("1234567890"));
    }
}
