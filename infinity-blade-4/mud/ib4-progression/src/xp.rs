use ib4_core::player::PlayerState;

pub const MAX_LEVEL: u32 = 45;

/// XP needed to reach level n from scratch: round(100 × n^1.5)
pub fn xp_for_level(n: u32) -> u64 {
    (100.0 * (n as f64).powf(1.5)).round() as u64
}

/// XP needed total to level up from level n (i.e., to reach level n+1).
/// Equals xp_for_level(n), so at level 1 the threshold is 100.
pub fn xp_threshold(level: u32) -> u64 {
    xp_for_level(level)
}

pub struct LevelUpEvent {
    pub new_level: u32,
    pub stat_points_gained: u32, // 2 per level
}

pub struct XPSystem;

impl XPSystem {
    /// Add XP (scaled by xp_mult from InfinitySeeker perk), return any level-up events.
    pub fn add_xp(player: &mut PlayerState, raw_xp: u64, xp_mult: f32) -> Vec<LevelUpEvent> {
        let gained = (raw_xp as f32 * xp_mult).round() as u64;
        player.xp += gained;
        let mut events = Vec::new();
        while player.level < MAX_LEVEL && player.xp >= xp_threshold(player.level) {
            player.level += 1;
            player.stat_points += 2;
            events.push(LevelUpEvent {
                new_level: player.level,
                stat_points_gained: 2,
            });
        }
        events
    }

    /// Allocate a stat point. Returns Err if no points or invalid stat name.
    pub fn allocate_stat(player: &mut PlayerState, stat: &str) -> Result<(), String> {
        if player.stat_points == 0 {
            return Err("No stat points available.".to_string());
        }
        match stat.to_lowercase().as_str() {
            "health" | "hp" => player.stat_health += 1,
            "attack" | "atk" => player.stat_attack += 1,
            "defense" | "def" => player.stat_defense += 1,
            "magic" | "mag" => player.stat_magic += 1,
            _ => {
                return Err(format!(
                    "Unknown stat '{}'. Use: health, attack, defense, magic",
                    stat
                ))
            }
        }
        player.stat_points -= 1;
        player.recalculate_stats();
        Ok(())
    }

    /// XP still needed to reach the next level.
    pub fn xp_to_next(player: &PlayerState) -> u64 {
        if player.level >= MAX_LEVEL {
            return 0;
        }
        xp_threshold(player.level).saturating_sub(player.xp)
    }
}
