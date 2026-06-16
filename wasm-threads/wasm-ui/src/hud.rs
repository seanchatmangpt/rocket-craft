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
