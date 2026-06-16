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
