use serde::{Deserialize, Serialize};
pub use nexus_types::{GundamSeries, MagicType};

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
