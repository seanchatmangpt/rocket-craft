/// Oracle: damage formula
pub fn oracle_damage(
    base: f32,
    combo_depth: u32,
    equipment_bonus_pct: f32,
    armor: f32,
    is_perfect_counter: bool,
) -> f32 {
    let combo_mult = match combo_depth {
        0 | 1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        _ => 3.0,
    };
    let counter = if is_perfect_counter { 1.5 } else { 1.0 };
    let raw = base * combo_mult * (1.0 + equipment_bonus_pct / 100.0) * counter;
    (raw - armor).max(1.0)
}

/// Oracle: XP required for level
pub fn oracle_xp_for_level(level: u32) -> u64 {
    100 * (level as u64).pow(2)
}

/// Oracle: perk unlock bloodline requirement
pub fn oracle_perk_available(perk_tier: u8, bloodline: u32) -> bool {
    match perk_tier {
        1 => {
            let _ = bloodline;
            true
        }
        2 => bloodline >= 5,
        3 => bloodline >= 10,
        _ => false,
    }
}

/// Oracle: gacha pull probability (cumulative pity model)
pub fn oracle_ssr_probability(pull_number: u32) -> f64 {
    if pull_number < 70 {
        0.03 // base 3% SSR rate
    } else if pull_number < 90 {
        // Soft pity: increase by ~5% per pull above 70
        let extra = (pull_number - 70) as f64 * 0.05;
        (0.03 + extra).min(1.0)
    } else {
        1.0 // hard pity at 90
    }
}

/// Oracle: GodKing shield parry counter
pub fn oracle_shield_breaks(parries_so_far: u32, new_parry_is_perfect: bool) -> bool {
    new_parry_is_perfect && parries_so_far + 1 >= 3
}

/// Oracle: Trans-Am activation condition
pub fn oracle_trans_am_activates(combo_depth: u32, gauge_pct: f32) -> bool {
    combo_depth >= 4 && gauge_pct >= 1.0
}

/// Oracle: auction minimum bid
pub fn oracle_min_bid(current_bid: u32) -> u32 {
    if current_bid == 0 {
        1
    } else {
        current_bid + (current_bid / 20).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── oracle_damage ─────────────────────────────────────────────────────────

    #[test]
    fn base_damage_no_combo_no_armor() {
        let d = oracle_damage(100.0, 0, 0.0, 0.0, false);
        // 100 * 1.0 * 1.0 * 1.0 - 0 = 100
        assert!((d - 100.0).abs() < 0.01);
    }

    #[test]
    fn combo_depth_3_doubles_damage() {
        let d = oracle_damage(100.0, 3, 0.0, 0.0, false);
        assert!((d - 200.0).abs() < 0.01);
    }

    #[test]
    fn perfect_counter_adds_50pct() {
        let without = oracle_damage(100.0, 1, 0.0, 0.0, false);
        let with_pc = oracle_damage(100.0, 1, 0.0, 0.0, true);
        assert!((with_pc / without - 1.5).abs() < 0.01);
    }

    #[test]
    fn armor_reduces_damage_with_floor_at_1() {
        let d = oracle_damage(10.0, 0, 0.0, 50.0, false); // raw=10, armor=50 → clamped to 1
        assert_eq!(d, 1.0);
    }

    #[test]
    fn equipment_bonus_increases_damage() {
        let d = oracle_damage(100.0, 0, 50.0, 0.0, false); // 50% bonus
        assert!((d - 150.0).abs() < 0.01);
    }

    #[test]
    fn combo_depth_above_3_uses_3x_multiplier() {
        // depth=3 → 2.0x, depth 4+ → 3.0x (match arm _ => 3.0)
        let d4 = oracle_damage(100.0, 4, 0.0, 0.0, false);
        let d5 = oracle_damage(100.0, 5, 0.0, 0.0, false);
        assert_eq!(d4, d5);
        assert!((d4 - 300.0).abs() < 0.01);
    }

    // ── oracle_xp_for_level ───────────────────────────────────────────────────

    #[test]
    fn xp_for_level_1_is_100() {
        assert_eq!(oracle_xp_for_level(1), 100);
    }

    #[test]
    fn xp_for_level_10_is_10000() {
        assert_eq!(oracle_xp_for_level(10), 10_000);
    }

    #[test]
    fn xp_is_quadratic() {
        assert_eq!(oracle_xp_for_level(5), 2_500);
        assert_eq!(oracle_xp_for_level(20), 40_000);
    }

    // ── oracle_perk_available ─────────────────────────────────────────────────

    #[test]
    fn tier1_perk_always_available() {
        assert!(oracle_perk_available(1, 0));
        assert!(oracle_perk_available(1, 100));
    }

    #[test]
    fn tier2_requires_bloodline_5() {
        assert!(!oracle_perk_available(2, 4));
        assert!(oracle_perk_available(2, 5));
    }

    #[test]
    fn tier3_requires_bloodline_10() {
        assert!(!oracle_perk_available(3, 9));
        assert!(oracle_perk_available(3, 10));
    }

    #[test]
    fn unknown_tier_always_false() {
        assert!(!oracle_perk_available(99, 100));
    }

    // ── oracle_ssr_probability ────────────────────────────────────────────────

    #[test]
    fn base_rate_before_soft_pity() {
        assert!((oracle_ssr_probability(0) - 0.03).abs() < 0.001);
        assert!((oracle_ssr_probability(69) - 0.03).abs() < 0.001);
    }

    #[test]
    fn soft_pity_increases_rate_after_pull_70() {
        let p70 = oracle_ssr_probability(70);
        let p80 = oracle_ssr_probability(80);
        assert!(p80 > p70, "soft pity should increase rate");
    }

    #[test]
    fn hard_pity_at_pull_90_is_100pct() {
        assert_eq!(oracle_ssr_probability(90), 1.0);
        assert_eq!(oracle_ssr_probability(100), 1.0);
    }

    // ── oracle_shield_breaks ─────────────────────────────────────────────────

    #[test]
    fn shield_does_not_break_on_non_perfect_parry() {
        assert!(!oracle_shield_breaks(2, false));
    }

    #[test]
    fn shield_breaks_after_three_perfect_parries() {
        assert!(!oracle_shield_breaks(1, true)); // 1+1=2 < 3
        assert!(!oracle_shield_breaks(2, false)); // not perfect
        assert!(oracle_shield_breaks(2, true));  // 2+1=3 ≥ 3 ✓
    }

    // ── oracle_trans_am_activates ─────────────────────────────────────────────

    #[test]
    fn trans_am_requires_both_conditions() {
        assert!(!oracle_trans_am_activates(4, 0.5)); // gauge too low
        assert!(!oracle_trans_am_activates(3, 1.0)); // depth too low
        assert!(oracle_trans_am_activates(4, 1.0));  // both satisfied
    }
}
