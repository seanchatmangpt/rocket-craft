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

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::player::PlayerState;

    fn fresh() -> PlayerState {
        PlayerState::new("Siris")
    }

    // ── xp_for_level ─────────────────────────────────────────────────────────

    #[test]
    fn xp_for_level_1_is_100() {
        // 100 * 1.0^1.5 = 100
        assert_eq!(xp_for_level(1), 100);
    }

    #[test]
    fn xp_for_level_2_is_283() {
        // 100 * 2.0^1.5 = 100 * 2.828... = 282.8 → round → 283
        assert_eq!(xp_for_level(2), 283);
    }

    #[test]
    fn xp_for_level_scale_is_monotonic() {
        for level in 1..MAX_LEVEL {
            assert!(xp_for_level(level + 1) > xp_for_level(level),
                "xp_for_level must be monotonically increasing");
        }
    }

    // ── XPSystem::add_xp ─────────────────────────────────────────────────────

    #[test]
    fn add_xp_below_threshold_does_not_level_up() {
        let mut p = fresh();
        let events = XPSystem::add_xp(&mut p, 50, 1.0);
        assert!(events.is_empty());
        assert_eq!(p.level, 1);
        assert_eq!(p.xp, 50);
    }

    #[test]
    fn add_xp_reaching_threshold_levels_up() {
        let mut p = fresh();
        // Threshold at level 1 = xp_for_level(1) = 100
        let events = XPSystem::add_xp(&mut p, 100, 1.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_level, 2);
        assert_eq!(p.level, 2);
    }

    #[test]
    fn level_up_awards_two_stat_points() {
        let mut p = fresh();
        let before = p.stat_points;
        XPSystem::add_xp(&mut p, 100, 1.0);
        assert_eq!(p.stat_points, before + 2);
    }

    #[test]
    fn xp_mult_scales_gained_xp() {
        let mut p = fresh();
        XPSystem::add_xp(&mut p, 100, 2.0); // 100 * 2.0 = 200 xp — cumulative, not reset on level-up
        assert_eq!(p.xp, 200);
        assert_eq!(p.level, 2, "200 xp crosses level-1 threshold of 100");
    }

    #[test]
    fn cannot_level_beyond_max() {
        let mut p = fresh();
        p.level = MAX_LEVEL;
        let events = XPSystem::add_xp(&mut p, u64::MAX, 1.0);
        assert!(events.is_empty(), "no events at max level");
        assert_eq!(p.level, MAX_LEVEL);
    }

    #[test]
    fn xp_to_next_decreases_after_gaining_xp() {
        let mut p = fresh();
        let before = XPSystem::xp_to_next(&p);
        XPSystem::add_xp(&mut p, 40, 1.0);
        let after = XPSystem::xp_to_next(&p);
        assert!(after < before, "xp_to_next must decrease after gaining xp");
    }

    #[test]
    fn xp_to_next_is_zero_at_max_level() {
        let mut p = fresh();
        p.level = MAX_LEVEL;
        assert_eq!(XPSystem::xp_to_next(&p), 0);
    }

    // ── XPSystem::allocate_stat ───────────────────────────────────────────────

    #[test]
    fn allocate_stat_with_no_points_returns_err() {
        let mut p = fresh();
        p.stat_points = 0;
        assert!(XPSystem::allocate_stat(&mut p, "health").is_err());
    }

    #[test]
    fn allocate_stat_increments_correct_field() {
        let mut p = fresh();
        p.stat_points = 4;
        let before_hp = p.stat_health;
        let before_atk = p.stat_attack;
        XPSystem::allocate_stat(&mut p, "hp").unwrap();
        XPSystem::allocate_stat(&mut p, "atk").unwrap();
        XPSystem::allocate_stat(&mut p, "def").unwrap();
        XPSystem::allocate_stat(&mut p, "mag").unwrap();
        assert_eq!(p.stat_health, before_hp + 1);
        assert_eq!(p.stat_attack, before_atk + 1);
    }

    #[test]
    fn allocate_stat_consumes_one_point() {
        let mut p = fresh();
        p.stat_points = 3;
        XPSystem::allocate_stat(&mut p, "attack").unwrap();
        assert_eq!(p.stat_points, 2);
    }

    #[test]
    fn allocate_unknown_stat_returns_err() {
        let mut p = fresh();
        p.stat_points = 1;
        assert!(XPSystem::allocate_stat(&mut p, "dexterity").is_err());
        assert_eq!(p.stat_points, 1, "point must not be consumed on error");
    }
}
