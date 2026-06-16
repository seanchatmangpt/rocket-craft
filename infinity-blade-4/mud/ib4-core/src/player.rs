use serde::{Serialize, Deserialize};
use crate::types::{CombatState, MagicType};
use crate::equipment::{Weapon, Shield};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub level: u32,       // 1–45
    pub xp: u64,
    pub bloodline: i32,   // 0 normal, >0 rebirth count, <0 negative
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
