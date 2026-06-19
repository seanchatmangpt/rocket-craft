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
