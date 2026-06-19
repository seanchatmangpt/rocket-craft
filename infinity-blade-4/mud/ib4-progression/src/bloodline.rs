use ib4_core::{player::PlayerState, types::MagicType};

pub struct RebirthResult {
    pub new_bloodline: i32,
    pub perk_point_gained: bool,
    pub newly_unlocked_magic: Vec<MagicType>,
    pub entered_negative: bool,
    pub mastery_xp_mult: f32,
}

pub struct BloodlineSystem;

impl BloodlineSystem {
    /// Trigger rebirth.
    ///
    /// Preserves: level, xp, stats, stat_points, selected_perks, magic_unlocks.
    /// Resets: gold → 0, weapon → None, shield → None, loot_bag → empty, qip_scar_stacks → 0.
    /// Grants: +1 perk_point (unless entering negative bloodline territory, i.e. bloodline > 20).
    pub fn trigger_rebirth(player: &mut PlayerState) -> RebirthResult {
        player.bloodline += 1;
        let entered_negative = player.bloodline > 20;
        let perk_point_gained = !entered_negative;

        // Reset run economy
        player.gold = 0;
        player.weapon = None;
        player.shield = None;
        player.loot_bag.clear();
        player.qip_scar_stacks = 0;

        if perk_point_gained {
            player.perk_points += 1;
        }

        // Check for new magic unlocks based on current bloodline
        let magic_thresholds: &[(i32, MagicType)] = &[
            (3, MagicType::Lightning),
            (6, MagicType::Ice),
            (10, MagicType::Dark),
            (15, MagicType::Light),
        ];

        let mut newly_unlocked: Vec<MagicType> = Vec::new();
        for (req, magic) in magic_thresholds {
            if player.bloodline >= *req && !player.magic_unlocks.contains(magic) {
                player.magic_unlocks.push(magic.clone());
                newly_unlocked.push(magic.clone());
            }
        }

        // Restore full HP and mana after rebirth
        player.health = player.max_health;
        player.mana = player.max_mana;

        let mastery_xp_mult = 2.0_f32.powi(player.bloodline.max(0));

        RebirthResult {
            new_bloodline: player.bloodline,
            perk_point_gained,
            newly_unlocked_magic: newly_unlocked,
            entered_negative,
            mastery_xp_mult,
        }
    }

    /// Enemy HP scale factor: 1.0 + bloodline × 0.15 (enemies get harder each rebirth).
    pub fn enemy_hp_scale(bloodline: i32) -> f32 {
        1.0 + bloodline.max(0) as f32 * 0.15
    }

    /// GodKing level = 50 × (bloodline + 1).
    pub fn god_king_level(bloodline: i32) -> u32 {
        (50 * (bloodline.max(0) + 1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::{player::PlayerState, types::MagicType};

    fn fresh() -> PlayerState {
        PlayerState::new("Siris")
    }

    // ── BloodlineSystem::trigger_rebirth ─────────────────────────────────────

    #[test]
    fn rebirth_increments_bloodline() {
        let mut p = fresh();
        assert_eq!(p.bloodline, 0);
        BloodlineSystem::trigger_rebirth(&mut p);
        assert_eq!(p.bloodline, 1);
    }

    #[test]
    fn rebirth_resets_gold_to_zero() {
        let mut p = fresh();
        p.gold = 9999;
        BloodlineSystem::trigger_rebirth(&mut p);
        assert_eq!(p.gold, 0);
    }

    #[test]
    fn rebirth_clears_weapon_and_shield() {
        let mut p = fresh();
        assert!(p.weapon.is_some());
        assert!(p.shield.is_some());
        BloodlineSystem::trigger_rebirth(&mut p);
        assert!(p.weapon.is_none());
        assert!(p.shield.is_none());
    }

    #[test]
    fn rebirth_clears_qip_scar_stacks() {
        let mut p = fresh();
        p.qip_scar_stacks = 2;
        BloodlineSystem::trigger_rebirth(&mut p);
        assert_eq!(p.qip_scar_stacks, 0);
    }

    #[test]
    fn rebirth_restores_full_health_and_mana() {
        let mut p = fresh();
        p.health = 10.0;
        p.mana = 5.0;
        BloodlineSystem::trigger_rebirth(&mut p);
        assert_eq!(p.health, p.max_health);
        assert_eq!(p.mana, p.max_mana);
    }

    #[test]
    fn rebirth_grants_perk_point_before_bloodline_21() {
        let mut p = fresh();
        p.bloodline = 19; // rebirth to 20 — still normal
        let before = p.perk_points;
        let result = BloodlineSystem::trigger_rebirth(&mut p);
        assert!(result.perk_point_gained);
        assert_eq!(p.perk_points, before + 1);
    }

    #[test]
    fn rebirth_does_not_grant_perk_point_at_bloodline_21() {
        let mut p = fresh();
        p.bloodline = 20; // rebirth to 21 → negative territory
        let before = p.perk_points;
        let result = BloodlineSystem::trigger_rebirth(&mut p);
        assert!(!result.perk_point_gained);
        assert!(result.entered_negative);
        assert_eq!(p.perk_points, before);
    }

    #[test]
    fn magic_unlocked_at_correct_bloodline_thresholds() {
        let mut p = fresh();
        // Lightning unlocks at bloodline 3
        p.bloodline = 2;
        let r = BloodlineSystem::trigger_rebirth(&mut p); // bloodline → 3
        assert!(r.newly_unlocked_magic.contains(&MagicType::Lightning));
        assert!(p.magic_unlocks.contains(&MagicType::Lightning));
    }

    #[test]
    fn magic_not_re_unlocked_on_second_rebirth_at_same_threshold() {
        let mut p = fresh();
        p.bloodline = 2;
        BloodlineSystem::trigger_rebirth(&mut p); // unlocks Lightning
        let r2 = BloodlineSystem::trigger_rebirth(&mut p); // bloodline → 4
        // Lightning already unlocked — should not appear again
        assert!(!r2.newly_unlocked_magic.contains(&MagicType::Lightning));
    }

    #[test]
    fn mastery_xp_mult_doubles_each_bloodline() {
        // mastery_xp_mult = 2^bloodline
        let mut p = fresh(); // bloodline 0
        let r = BloodlineSystem::trigger_rebirth(&mut p); // bloodline 1
        assert!((r.mastery_xp_mult - 2.0_f32).abs() < 1e-5);
        let r2 = BloodlineSystem::trigger_rebirth(&mut p); // bloodline 2
        assert!((r2.mastery_xp_mult - 4.0_f32).abs() < 1e-5);
    }

    // ── BloodlineSystem::enemy_hp_scale ─────────────────────────────────────

    #[test]
    fn enemy_hp_scale_at_bloodline_0_is_1() {
        assert!((BloodlineSystem::enemy_hp_scale(0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn enemy_hp_scale_increases_per_rebirth() {
        // bloodline 1 → 1.0 + 1*0.15 = 1.15
        assert!((BloodlineSystem::enemy_hp_scale(1) - 1.15).abs() < 1e-5);
    }

    #[test]
    fn enemy_hp_scale_clamps_negative_bloodline_to_baseline() {
        // bloodline < 0 → max(0, bloodline) → still 1.0
        assert!((BloodlineSystem::enemy_hp_scale(-5) - 1.0).abs() < 1e-5);
    }

    // ── BloodlineSystem::god_king_level ──────────────────────────────────────

    #[test]
    fn god_king_level_at_bloodline_0_is_50() {
        assert_eq!(BloodlineSystem::god_king_level(0), 50);
    }

    #[test]
    fn god_king_level_at_bloodline_1_is_100() {
        assert_eq!(BloodlineSystem::god_king_level(1), 100);
    }

    #[test]
    fn god_king_level_clamps_negative_bloodline() {
        assert_eq!(BloodlineSystem::god_king_level(-3), 50);
    }
}
