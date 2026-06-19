use crate::equipment::{Shield, Weapon};
use crate::types::{CombatState, MagicType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub level: u32, // 1–45
    pub xp: u64,
    pub bloodline: i32, // 0 normal, >0 rebirth count, <0 negative
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub gold: u32,
    pub stat_attack: u32,
    pub stat_defense: u32,
    pub stat_magic: u32,
    pub stat_health: u32,
    pub stat_points: u32,
    pub perk_points: u32,
    pub selected_perks: Vec<String>,
    pub magic_unlocks: Vec<MagicType>,
    pub weapon: Option<Weapon>,
    pub shield: Option<Shield>,
    pub combat_state: CombatState,
    pub qip_scar_stacks: u32,
    pub loot_bag: Vec<Weapon>,
}

impl PlayerState {
    /// Create a new level 1 player with starter equipment.
    /// HP = 100 + stat_health * 60 = 100 + 3 * 60 = 280
    /// Mana = 60 + stat_magic * 10 = 60 + 0 * 10 = 60
    pub fn new(name: &str) -> Self {
        let stat_health = 3u32;
        let stat_magic = 0u32;
        let max_health = 100.0 + stat_health as f32 * 60.0;
        let max_mana = 60.0 + stat_magic as f32 * 10.0;

        PlayerState {
            name: name.to_string(),
            level: 1,
            xp: 0,
            bloodline: 0,
            health: max_health,
            max_health,
            mana: max_mana,
            max_mana,
            gold: 100,
            stat_attack: 2,
            stat_defense: 2,
            stat_magic,
            stat_health,
            stat_points: 0,
            perk_points: 0,
            selected_perks: Vec::new(),
            magic_unlocks: vec![MagicType::Fire],
            weapon: Some(Weapon::starter()),
            shield: Some(Shield::starter()),
            combat_state: CombatState::Idle,
            qip_scar_stacks: 0,
            loot_bag: Vec::new(),
        }
    }

    pub fn recalculate_stats(&mut self) {
        self.max_health = 100.0 + self.stat_health as f32 * 60.0;
        self.max_mana = 60.0 + self.stat_magic as f32 * 10.0;
    }

    pub fn bloodline_label(&self) -> String {
        match self.bloodline {
            0 => "0".to_string(),
            1..=20 => roman(self.bloodline as u32),
            n if n < 0 => format!("N-{}", roman((-n) as u32)),
            _ => format!("BL{}", self.bloodline),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.combat_state = CombatState::Dead;
        }
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn spend_mana(&mut self, cost: f32) -> bool {
        if self.mana >= cost {
            self.mana -= cost;
            true
        } else {
            false
        }
    }

    pub fn has_perk(&self, perk_id: &str) -> bool {
        self.selected_perks.iter().any(|p| p == perk_id)
    }

    pub fn has_magic(&self, magic: &MagicType) -> bool {
        self.magic_unlocks.contains(magic)
    }
}

fn roman(n: u32) -> String {
    let values = [
        (20, "XX"),
        (19, "XIX"),
        (18, "XVIII"),
        (17, "XVII"),
        (16, "XVI"),
        (15, "XV"),
        (14, "XIV"),
        (13, "XIII"),
        (12, "XII"),
        (11, "XI"),
        (10, "X"),
        (9, "IX"),
        (8, "VIII"),
        (7, "VII"),
        (6, "VI"),
        (5, "V"),
        (4, "IV"),
        (3, "III"),
        (2, "II"),
        (1, "I"),
    ];
    for (val, sym) in &values {
        if n >= *val {
            return sym.to_string();
        }
    }
    n.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CombatState, MagicType};

    fn p() -> PlayerState { PlayerState::new("Siris") }

    // ── PlayerState::new defaults ─────────────────────────────────────────────

    #[test]
    fn new_player_starts_at_level_1_bloodline_0() {
        let p = p();
        assert_eq!(p.level, 1);
        assert_eq!(p.bloodline, 0);
    }

    #[test]
    fn new_player_max_health_formula_is_100_plus_stat_health_times_60() {
        let p = p(); // stat_health = 3 → max_health = 100 + 3*60 = 280
        assert_eq!(p.max_health, 280.0);
        assert_eq!(p.health, 280.0);
    }

    #[test]
    fn new_player_has_fire_magic_unlocked() {
        assert!(p().has_magic(&MagicType::Fire));
    }

    // ── recalculate_stats ─────────────────────────────────────────────────────

    #[test]
    fn recalculate_stats_updates_max_health() {
        let mut p = p();
        p.stat_health = 5;
        p.recalculate_stats();
        // 100 + 5 * 60 = 400
        assert_eq!(p.max_health, 400.0);
    }

    #[test]
    fn recalculate_stats_updates_max_mana() {
        let mut p = p();
        p.stat_magic = 4;
        p.recalculate_stats();
        // 60 + 4 * 10 = 100
        assert_eq!(p.max_mana, 100.0);
    }

    // ── is_alive / take_damage ────────────────────────────────────────────────

    #[test]
    fn new_player_is_alive() {
        assert!(p().is_alive());
    }

    #[test]
    fn take_damage_reduces_health() {
        let mut p = p();
        p.take_damage(50.0);
        assert!((p.health - 230.0).abs() < 0.01);
        assert!(p.is_alive());
    }

    #[test]
    fn lethal_damage_kills_player_and_sets_dead_state() {
        let mut p = p();
        p.take_damage(9999.0);
        assert_eq!(p.health, 0.0);
        assert!(!p.is_alive());
        assert_eq!(p.combat_state, CombatState::Dead);
    }

    #[test]
    fn take_damage_floors_health_at_zero() {
        let mut p = p();
        p.take_damage(p.max_health + 100.0);
        assert_eq!(p.health, 0.0);
    }

    // ── heal ──────────────────────────────────────────────────────────────────

    #[test]
    fn heal_increases_health() {
        let mut p = p();
        p.health = 100.0;
        p.heal(50.0);
        assert!((p.health - 150.0).abs() < 0.01);
    }

    #[test]
    fn heal_caps_at_max_health() {
        let mut p = p();
        p.health = p.max_health - 10.0;
        p.heal(9999.0);
        assert_eq!(p.health, p.max_health);
    }

    // ── spend_mana ────────────────────────────────────────────────────────────

    #[test]
    fn spend_mana_returns_true_and_deducts_when_sufficient() {
        let mut p = p();
        let before = p.mana;
        let ok = p.spend_mana(20.0);
        assert!(ok);
        assert!((p.mana - (before - 20.0)).abs() < 0.01);
    }

    #[test]
    fn spend_mana_returns_false_and_does_not_deduct_when_insufficient() {
        let mut p = p();
        p.mana = 5.0;
        let ok = p.spend_mana(20.0);
        assert!(!ok);
        assert_eq!(p.mana, 5.0);
    }

    // ── has_perk / has_magic ──────────────────────────────────────────────────

    #[test]
    fn has_perk_returns_false_for_unowned_perk() {
        assert!(!p().has_perk("infinity-seeker"));
    }

    #[test]
    fn has_perk_returns_true_after_adding() {
        let mut p = p();
        p.selected_perks.push("infinity-seeker".into());
        assert!(p.has_perk("infinity-seeker"));
    }

    #[test]
    fn has_magic_returns_false_for_locked_magic() {
        assert!(!p().has_magic(&MagicType::Dark));
    }

    // ── bloodline_label ───────────────────────────────────────────────────────

    #[test]
    fn bloodline_0_label_is_zero() {
        assert_eq!(p().bloodline_label(), "0");
    }

    #[test]
    fn bloodline_1_label_is_roman_i() {
        let mut p = p();
        p.bloodline = 1;
        assert_eq!(p.bloodline_label(), "I");
    }

    #[test]
    fn bloodline_negative_label_has_n_prefix() {
        let mut p = p();
        p.bloodline = -3;
        assert!(p.bloodline_label().starts_with("N-"), "negative bloodline must start with N-");
    }

    #[test]
    fn bloodline_above_20_uses_numeric_label() {
        let mut p = p();
        p.bloodline = 21;
        assert!(p.bloodline_label().starts_with("BL"), "bloodline > 20 must use BL prefix");
    }
}
