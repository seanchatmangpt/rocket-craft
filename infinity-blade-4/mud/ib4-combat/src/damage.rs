use ib4_core::{enemy::EnemyInstance, player::PlayerState};

/// Returns (damage_dealt, is_crit)
pub fn calc_player_damage(
    player: &PlayerState,
    enemy: &EnemyInstance,
    combo_multiplier: f32,
    extra_attack_mult: f32,
    extra_crit_chance: f32,
    rng_crit: f32,
) -> (f32, bool) {
    let weapon_bonus = player.weapon.as_ref().map(|w| w.attack_bonus).unwrap_or(0) as f32;
    let base = weapon_bonus + player.stat_attack as f32;
    let crit_chance = player
        .weapon
        .as_ref()
        .map(|w| w.crit_chance)
        .unwrap_or(0.05)
        + extra_crit_chance;
    let is_crit = rng_crit < crit_chance;
    let crit_mult = if is_crit { 1.5 } else { 1.0 };

    // Defense reduction: each point of enemy phase reduces damage
    let phase_defense = match enemy.phase {
        1 => 0.0,
        2 => 0.05,
        3 => 0.10,
        _ => 0.0,
    };

    let dmg = base * combo_multiplier * extra_attack_mult * crit_mult * (1.0 - phase_defense);
    (dmg.max(1.0), is_crit)
}

/// Returns damage the enemy deals to the player.
pub fn calc_enemy_damage(enemy: &EnemyInstance, player: &PlayerState, defense_mult: f32) -> f32 {
    let shield_bonus = player.shield.as_ref().map(|s| s.defense_bonus).unwrap_or(0) as f32;
    let defense = player.stat_defense as f32 + shield_bonus;
    let reduction = (defense / (defense + 50.0)) * defense_mult;
    let dmg = enemy.attack_damage * (1.0 - reduction);
    dmg.max(1.0)
}
