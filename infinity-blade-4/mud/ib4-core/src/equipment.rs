use crate::types::Rarity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub attack_bonus: i32,
    pub magic_bonus: i32,
    pub crit_chance: f32,
    pub rarity: Rarity,
    pub special_move: Option<String>,
    pub gem_slots: u32,
    pub mastery_xp: u64,
}

impl Weapon {
    pub fn starter() -> Self {
        Weapon {
            id: "steel_sword".to_string(),
            name: "Steel Sword".to_string(),
            attack_bonus: 12,
            magic_bonus: 0,
            crit_chance: 0.05,
            rarity: Rarity::Common,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shield {
    pub id: String,
    pub name: String,
    pub defense_bonus: i32,
    /// Extra turns of PerfectParry grace (0 for basic shields)
    pub parry_bonus_turns: u32,
    pub rarity: Rarity,
}

impl Shield {
    pub fn starter() -> Self {
        Shield {
            id: "iron_shield".to_string(),
            name: "Iron Shield".to_string(),
            defense_bonus: 8,
            parry_bonus_turns: 0,
            rarity: Rarity::Common,
        }
    }
}

/// All 22 weapons from the weapon catalog
pub fn weapon_catalog() -> Vec<Weapon> {
    vec![
        Weapon {
            id: "steel_sword".to_string(),
            name: "Steel Sword".to_string(),
            attack_bonus: 12,
            magic_bonus: 0,
            crit_chance: 0.05,
            rarity: Rarity::Common,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        },
        Weapon {
            id: "bronze_axe".to_string(),
            name: "Bronze Axe".to_string(),
            attack_bonus: 15,
            magic_bonus: 0,
            crit_chance: 0.05,
            rarity: Rarity::Common,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        },
        Weapon {
            id: "talon_blade".to_string(),
            name: "Talon Blade".to_string(),
            attack_bonus: 18,
            magic_bonus: 2,
            crit_chance: 0.08,
            rarity: Rarity::Uncommon,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        },
        Weapon {
            id: "iron_cleaver".to_string(),
            name: "Iron Cleaver".to_string(),
            attack_bonus: 22,
            magic_bonus: 0,
            crit_chance: 0.08,
            rarity: Rarity::Uncommon,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        },
        Weapon {
            id: "twin_fangs".to_string(),
            name: "Twin Fangs".to_string(),
            attack_bonus: 16,
            magic_bonus: 0,
            crit_chance: 0.10,
            rarity: Rarity::Uncommon,
            special_move: None,
            gem_slots: 1,
            mastery_xp: 0,
        },
        Weapon {
            id: "dawn_blade".to_string(),
            name: "Dawn Blade".to_string(),
            attack_bonus: 28,
            magic_bonus: 8,
            crit_chance: 0.10,
            rarity: Rarity::Rare,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "frost_edge".to_string(),
            name: "Frost Edge".to_string(),
            attack_bonus: 25,
            magic_bonus: 12,
            crit_chance: 0.10,
            rarity: Rarity::Rare,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "chrome_duals".to_string(),
            name: "Chrome Duals".to_string(),
            attack_bonus: 24,
            magic_bonus: 4,
            crit_chance: 0.10,
            rarity: Rarity::Rare,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "sword_of_storms".to_string(),
            name: "Sword of Storms".to_string(),
            attack_bonus: 35,
            magic_bonus: 18,
            crit_chance: 0.15,
            rarity: Rarity::Epic,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "void_blade".to_string(),
            name: "Void Blade".to_string(),
            attack_bonus: 45,
            magic_bonus: 10,
            crit_chance: 0.15,
            rarity: Rarity::Epic,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "thunder_kleve".to_string(),
            name: "Thunder Kleve".to_string(),
            attack_bonus: 40,
            magic_bonus: 22,
            crit_chance: 0.15,
            rarity: Rarity::Epic,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "gemini_sword".to_string(),
            name: "Gemini Sword".to_string(),
            attack_bonus: 32,
            magic_bonus: 14,
            crit_chance: 0.15,
            rarity: Rarity::Epic,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "corrupted_blades".to_string(),
            name: "Corrupted Blades".to_string(),
            attack_bonus: 38,
            magic_bonus: 20,
            crit_chance: 0.15,
            rarity: Rarity::Epic,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "legacy_sword".to_string(),
            name: "Legacy Sword".to_string(),
            attack_bonus: 52,
            magic_bonus: 8,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("VoidStrike".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "dark_edge".to_string(),
            name: "Dark Edge".to_string(),
            attack_bonus: 48,
            magic_bonus: 28,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("ShadowRend".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "quantum_blade".to_string(),
            name: "Quantum Blade".to_string(),
            attack_bonus: 60,
            magic_bonus: 20,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("QuantumBlade".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "phoenix_sword".to_string(),
            name: "Phoenix Sword".to_string(),
            attack_bonus: 55,
            magic_bonus: 30,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("PhoenixFlare".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "titan_kleve".to_string(),
            name: "Titan Kleve".to_string(),
            attack_bonus: 65,
            magic_bonus: 15,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("TitanCrush".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "infinity_blade_i".to_string(),
            name: "Infinity Blade I".to_string(),
            attack_bonus: 40,
            magic_bonus: 15,
            crit_chance: 0.15,
            rarity: Rarity::Rare,
            special_move: None,
            gem_slots: 2,
            mastery_xp: 0,
        },
        Weapon {
            id: "infinity_blade_iii".to_string(),
            name: "Infinity Blade III".to_string(),
            attack_bonus: 60,
            magic_bonus: 25,
            crit_chance: 0.20,
            rarity: Rarity::Legendary,
            special_move: Some("InfinitySlash".to_string()),
            gem_slots: 3,
            mastery_xp: 0,
        },
        Weapon {
            id: "infinity_blade_iv".to_string(),
            name: "Infinity Blade IV".to_string(),
            attack_bonus: 80,
            magic_bonus: 40,
            crit_chance: 0.25,
            rarity: Rarity::Infinity,
            special_move: Some("InfinitySlash".to_string()),
            gem_slots: 4,
            mastery_xp: 0,
        },
        Weapon {
            id: "abyssal_edge".to_string(),
            name: "Abyssal Edge".to_string(),
            attack_bonus: 75,
            magic_bonus: 50,
            crit_chance: 0.25,
            rarity: Rarity::Infinity,
            special_move: Some("AbyssalTide".to_string()),
            gem_slots: 4,
            mastery_xp: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Weapon ────────────────────────────────────────────────────────────────

    #[test]
    fn weapon_starter_has_expected_fields() {
        let w = Weapon::starter();
        assert_eq!(w.id, "steel_sword");
        assert_eq!(w.attack_bonus, 12);
        assert_eq!(w.crit_chance, 0.05);
        assert!(w.special_move.is_none());
        assert_eq!(w.gem_slots, 1);
        assert_eq!(w.mastery_xp, 0);
        assert_eq!(w.rarity, Rarity::Common);
    }

    #[test]
    fn weapon_starter_is_clone() {
        let w = Weapon::starter();
        let w2 = w.clone();
        assert_eq!(w.id, w2.id);
        assert_eq!(w.attack_bonus, w2.attack_bonus);
    }

    // ── Shield ────────────────────────────────────────────────────────────────

    #[test]
    fn shield_starter_has_expected_fields() {
        let s = Shield::starter();
        assert_eq!(s.id, "iron_shield");
        assert_eq!(s.defense_bonus, 8);
        assert_eq!(s.parry_bonus_turns, 0);
        assert_eq!(s.rarity, Rarity::Common);
    }

    // ── weapon_catalog ────────────────────────────────────────────────────────

    #[test]
    fn weapon_catalog_has_at_least_one_entry() {
        let catalog = weapon_catalog();
        assert!(!catalog.is_empty());
    }

    #[test]
    fn weapon_catalog_first_entry_is_steel_sword() {
        let catalog = weapon_catalog();
        assert_eq!(catalog[0].id, "steel_sword");
    }

    #[test]
    fn weapon_catalog_contains_only_positive_attack_bonuses() {
        let catalog = weapon_catalog();
        for w in &catalog {
            assert!(w.attack_bonus >= 0, "weapon {} has negative attack", w.id);
        }
    }

    #[test]
    fn weapon_catalog_crit_chances_are_valid_probabilities() {
        let catalog = weapon_catalog();
        for w in &catalog {
            assert!(
                w.crit_chance >= 0.0 && w.crit_chance <= 1.0,
                "weapon {} has invalid crit_chance {}",
                w.id, w.crit_chance
            );
        }
    }
}
