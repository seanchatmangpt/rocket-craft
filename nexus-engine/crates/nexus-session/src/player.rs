pub use nexus_types::{GundamSeries, MagicType};
use serde::{Deserialize, Serialize};

use crate::session::SessionError;

// ────────────────────────────────────────────────────────────────────────────
// Domain enums
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewtypeRank {
    Normal,
    SemiNewtype,
    Newtype,
    FullNewtype,
    Coordinator,
}

// GundamSeries imported from nexus_types

// MagicType imported from nexus_types

// ────────────────────────────────────────────────────────────────────────────
// PlayerProfile
// ────────────────────────────────────────────────────────────────────────────

/// Full profile for a Gundam Nexus player, covering base stats, progression,
/// duel records, and Newtype/Series metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: u64,
    pub username: String,

    // Progression
    pub level: u32,
    pub xp: u64,
    pub bloodline: u32,

    // Economy
    pub gold: u32,

    // Vitals
    pub hp: f32,
    pub max_hp: f32,
    pub mana: f32,
    pub max_mana: f32,

    // Combat stats
    pub stat_attack: u32,
    pub stat_defense: u32,
    pub stat_magic: u32,

    // Perks / abilities
    pub perk_points: u32,
    pub selected_perks: Vec<String>,
    pub magic_unlocks: Vec<MagicType>,
    pub qip_scar_stacks: u32,

    // Gundam Nexus
    pub newtype_rank: NewtypeRank,
    pub pilot_suit: Option<String>,
    pub active_series: Option<GundamSeries>,
    pub duel_rating: u32,
    pub duel_wins: u32,
    pub duel_losses: u32,
}

impl PlayerProfile {
    /// Create a level-1 profile with baseline stats.
    pub fn new(id: u64, username: String) -> Self {
        PlayerProfile {
            id,
            username,
            level: 1,
            xp: 0,
            bloodline: 0,
            gold: 100,
            hp: 100.0,
            max_hp: 100.0,
            mana: 50.0,
            max_mana: 50.0,
            stat_attack: 10,
            stat_defense: 5,
            stat_magic: 5,
            perk_points: 0,
            selected_perks: Vec::new(),
            magic_unlocks: Vec::new(),
            qip_scar_stacks: 0,
            newtype_rank: NewtypeRank::Normal,
            pilot_suit: None,
            active_series: None,
            duel_rating: 1000,
            duel_wins: 0,
            duel_losses: 0,
        }
    }

    /// Award XP and apply any level-up bonuses.
    ///
    /// Returns `true` when the player crossed a level threshold.
    pub fn apply_xp_gain(&mut self, xp: u64) -> bool {
        self.xp += xp;
        let xp_for_next = self.xp_required_for_level(self.level + 1);
        if self.xp >= xp_for_next {
            self.level += 1;
            self.stat_attack += 2;
            self.stat_defense += 1;
            true
        } else {
            false
        }
    }

    /// Quadratic XP curve: `100 * level²`.
    fn xp_required_for_level(&self, level: u32) -> u64 {
        100 * (level as u64).pow(2)
    }

    /// Deduct `amount` gold, returning an error if the player cannot afford it.
    /// Gold can never go below zero.
    pub fn spend_gold(&mut self, amount: u32) -> Result<(), SessionError> {
        self.gold
            .checked_sub(amount)
            .ok_or(SessionError::InsufficientGold {
                need: amount,
                have: self.gold,
            })
            .map(|remaining| self.gold = remaining)
    }

    /// `true` when the player's HP is above zero.
    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    /// Human-readable label for the player's bloodline tier.
    pub fn bloodline_label(&self) -> &'static str {
        match self.bloodline {
            0 => "First Blood",
            1..=5 => "Bloodline Awakened",
            6..=9 => "Veteran",
            10..=14 => "Deathless",
            15..=19 => "Ausar's Echo",
            20 => "Negative Bloodline",
            _ => "Beyond Reckoning",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> PlayerProfile {
        PlayerProfile::new(1, "Heero".into())
    }

    // ── new / defaults ────────────────────────────────────────────────────────

    #[test]
    fn new_player_starts_at_level_1_with_100_gold() {
        let p = player();
        assert_eq!(p.level, 1);
        assert_eq!(p.gold, 100);
        assert_eq!(p.xp, 0);
        assert_eq!(p.duel_rating, 1000);
    }

    #[test]
    fn new_player_is_alive() {
        assert!(player().is_alive());
    }

    // ── apply_xp_gain ─────────────────────────────────────────────────────────

    #[test]
    fn xp_below_threshold_does_not_level_up() {
        let mut p = player();
        // Level 2 requires 100 * 2² = 400 XP
        let levelled = p.apply_xp_gain(399);
        assert!(!levelled);
        assert_eq!(p.level, 1);
    }

    #[test]
    fn xp_at_threshold_levels_up() {
        let mut p = player();
        let levelled = p.apply_xp_gain(400); // 100 * 2² = 400
        assert!(levelled);
        assert_eq!(p.level, 2);
    }

    #[test]
    fn level_up_grants_stat_bonuses() {
        let mut p = player();
        let before_atk = p.stat_attack;
        let before_def = p.stat_defense;
        p.apply_xp_gain(400);
        assert_eq!(p.stat_attack, before_atk + 2);
        assert_eq!(p.stat_defense, before_def + 1);
    }

    #[test]
    fn xp_is_cumulative_across_calls() {
        let mut p = player();
        p.apply_xp_gain(200);
        p.apply_xp_gain(200);
        assert_eq!(p.level, 2, "split XP grants must still trigger level-up");
    }

    // ── spend_gold ────────────────────────────────────────────────────────────

    #[test]
    fn spend_gold_deducts_from_balance() {
        let mut p = player();
        p.spend_gold(50).unwrap();
        assert_eq!(p.gold, 50);
    }

    #[test]
    fn spend_gold_exact_balance_leaves_zero() {
        let mut p = player();
        p.spend_gold(100).unwrap();
        assert_eq!(p.gold, 0);
    }

    #[test]
    fn spend_gold_over_balance_returns_error() {
        let mut p = player();
        let result = p.spend_gold(101);
        assert!(matches!(result, Err(SessionError::InsufficientGold { need: 101, have: 100 })));
        assert_eq!(p.gold, 100, "gold must not be deducted on error");
    }

    // ── bloodline_label ───────────────────────────────────────────────────────

    #[test]
    fn bloodline_0_is_first_blood() {
        assert_eq!(player().bloodline_label(), "First Blood");
    }

    #[test]
    fn bloodline_1_is_awakened() {
        let mut p = player();
        p.bloodline = 1;
        assert_eq!(p.bloodline_label(), "Bloodline Awakened");
    }

    #[test]
    fn bloodline_10_is_deathless() {
        let mut p = player();
        p.bloodline = 10;
        assert_eq!(p.bloodline_label(), "Deathless");
    }

    #[test]
    fn bloodline_20_is_negative_bloodline() {
        let mut p = player();
        p.bloodline = 20;
        assert_eq!(p.bloodline_label(), "Negative Bloodline");
    }

    #[test]
    fn bloodline_beyond_20_is_beyond_reckoning() {
        let mut p = player();
        p.bloodline = 21;
        assert_eq!(p.bloodline_label(), "Beyond Reckoning");
    }

    // ── is_alive ──────────────────────────────────────────────────────────────

    #[test]
    fn player_with_zero_hp_is_not_alive() {
        let mut p = player();
        p.hp = 0.0;
        assert!(!p.is_alive());
    }

    #[test]
    fn player_with_positive_hp_is_alive() {
        let mut p = player();
        p.hp = 0.001;
        assert!(p.is_alive());
    }
}
