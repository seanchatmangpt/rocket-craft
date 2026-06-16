use serde::{Serialize, Deserialize};
use crate::types::{AttackDir, TitanType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyInstance {
    pub id: String,
    pub name: String,
    pub titan_type: TitanType,
    pub base_hp: f32,
    pub current_hp: f32,
    pub base_attack_damage: f32,
    pub attack_damage: f32,   // modified by phase scaling
    pub phase: u8,
    pub bloodline_required: i32,
    pub reward_xp: u64,
    pub reward_gold: u32,
    pub drop_chance: f32,
    pub pending_attack: Option<AttackDir>,
    pub is_stunned: bool,
    pub stun_turns_remaining: u32,
    pub shield_active: bool,
    pub perfect_parries_received: u32,
}

impl EnemyInstance {
    pub fn hp_percent(&self) -> f32 {
        if self.base_hp <= 0.0 {
            return 0.0;
        }
        self.current_hp / self.base_hp * 100.0
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current_hp = (self.current_hp - amount).max(0.0);
    }

    pub fn apply_stun(&mut self, turns: u32) {
        self.is_stunned = true;
        self.stun_turns_remaining = turns;
    }

    pub fn tick_stun(&mut self) {
        if self.is_stunned {
            if self.stun_turns_remaining > 0 {
                self.stun_turns_remaining -= 1;
            }
            if self.stun_turns_remaining == 0 {
                self.is_stunned = false;
            }
        }
    }

    pub fn phase_label(&self) -> &str {
        match self.phase {
            1 => "Phase I",
            2 => "Phase II",
            3 => "Phase III",
            _ => "?",
        }
    }
}
