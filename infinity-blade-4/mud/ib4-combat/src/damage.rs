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

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::{
        enemy::EnemyInstance,
        player::PlayerState,
        types::TitanType,
    };

    fn enemy(phase: u8, attack: f32) -> EnemyInstance {
        EnemyInstance {
            id: "titan".into(),
            name: "Titan".into(),
            titan_type: TitanType::Warrior,
            base_hp: 1000.0,
            current_hp: 1000.0,
            base_attack_damage: attack,
            attack_damage: attack,
            phase,
            bloodline_required: 0,
            reward_xp: 100,
            reward_gold: 50,
            drop_chance: 0.1,
            pending_attack: None,
            is_stunned: false,
            stun_turns_remaining: 0,
            shield_active: false,
            perfect_parries_received: 0,
        }
    }

    // ── calc_player_damage ────────────────────────────────────────────────────

    #[test]
    fn damage_floor_is_one() {
        let mut p = PlayerState::new("Siris");
        p.weapon = None;
        p.stat_attack = 0;
        let e = enemy(1, 10.0);
        let (dmg, _) = calc_player_damage(&p, &e, 1.0, 1.0, 0.0, 1.0); // rng=1.0 → no crit
        assert!(dmg >= 1.0, "damage floor must be 1.0, got {dmg}");
    }

    #[test]
    fn no_crit_when_rng_exceeds_chance() {
        let p = PlayerState::new("Siris");
        let e = enemy(1, 10.0);
        // Default weapon crit_chance = 0.05; rng=0.9 → no crit
        let (_, is_crit) = calc_player_damage(&p, &e, 1.0, 1.0, 0.0, 0.9);
        assert!(!is_crit);
    }

    #[test]
    fn crit_when_rng_below_chance() {
        let p = PlayerState::new("Siris");
        let e = enemy(1, 10.0);
        // rng=0.01 < 0.05 default crit → is_crit
        let (dmg_crit, is_crit) = calc_player_damage(&p, &e, 1.0, 1.0, 0.0, 0.01);
        let (dmg_normal, _) = calc_player_damage(&p, &e, 1.0, 1.0, 0.0, 0.9);
        assert!(is_crit);
        assert!((dmg_crit / dmg_normal - 1.5).abs() < 0.01, "crit mult must be 1.5×");
    }

    #[test]
    fn phase_2_applies_5_percent_defense() {
        let p = PlayerState::new("Siris");
        let e1 = enemy(1, 10.0);
        let e2 = enemy(2, 10.0);
        let (d1, _) = calc_player_damage(&p, &e1, 1.0, 1.0, 0.0, 0.9);
        let (d2, _) = calc_player_damage(&p, &e2, 1.0, 1.0, 0.0, 0.9);
        assert!(d2 < d1, "phase 2 should reduce damage");
        assert!((d1 - d2) / d1 - 0.05 < 0.01, "5% reduction expected");
    }

    #[test]
    fn phase_3_applies_10_percent_defense() {
        let p = PlayerState::new("Siris");
        let e1 = enemy(1, 10.0);
        let e3 = enemy(3, 10.0);
        let (d1, _) = calc_player_damage(&p, &e1, 1.0, 1.0, 0.0, 0.9);
        let (d3, _) = calc_player_damage(&p, &e3, 1.0, 1.0, 0.0, 0.9);
        assert!((d1 - d3) / d1 - 0.10 < 0.01, "10% reduction expected for phase 3");
    }

    #[test]
    fn combo_multiplier_scales_damage() {
        let p = PlayerState::new("Siris");
        let e = enemy(1, 10.0);
        let (d1, _) = calc_player_damage(&p, &e, 1.0, 1.0, 0.0, 0.9);
        let (d2, _) = calc_player_damage(&p, &e, 2.0, 1.0, 0.0, 0.9);
        assert!((d2 / d1 - 2.0).abs() < 0.01, "2× combo mult should double damage");
    }

    // ── calc_enemy_damage ─────────────────────────────────────────────────────

    #[test]
    fn enemy_damage_floor_is_one() {
        let e = enemy(1, 1.0);
        let mut p = PlayerState::new("Siris");
        p.stat_defense = 9999;
        let dmg = calc_enemy_damage(&e, &p, 1.0);
        assert!(dmg >= 1.0, "enemy damage floor must be 1.0");
    }

    #[test]
    fn higher_defense_reduces_enemy_damage() {
        let e = enemy(1, 100.0);
        let mut p_low = PlayerState::new("Low");
        p_low.stat_defense = 1;
        p_low.shield = None;
        let mut p_high = PlayerState::new("High");
        p_high.stat_defense = 50;
        p_high.shield = None;
        let d_low = calc_enemy_damage(&e, &p_low, 1.0);
        let d_high = calc_enemy_damage(&e, &p_high, 1.0);
        assert!(d_high < d_low, "higher defense must reduce damage taken");
    }

    #[test]
    fn defense_mult_zero_disables_reduction_giving_full_damage() {
        // defense_mult=0 → reduction factor = 0 → no mitigation → full attack_damage
        let e = enemy(1, 100.0);
        let p = PlayerState::new("Siris");
        let dmg = calc_enemy_damage(&e, &p, 0.0);
        assert_eq!(dmg, 100.0, "defense_mult=0 disables all mitigation");
    }
}
