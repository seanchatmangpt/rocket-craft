use crate::types::{AttackDir, TitanType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyInstance {
    pub id: String,
    pub name: String,
    pub titan_type: TitanType,
    pub base_hp: f32,
    pub current_hp: f32,
    pub base_attack_damage: f32,
    pub attack_damage: f32, // modified by phase scaling
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TitanType;

    fn titan(hp: f32) -> EnemyInstance {
        EnemyInstance {
            id: "t1".into(),
            name: "Titan".into(),
            titan_type: TitanType::Warrior,
            base_hp: hp,
            current_hp: hp,
            base_attack_damage: 10.0,
            attack_damage: 10.0,
            phase: 1,
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

    // ── hp_percent ────────────────────────────────────────────────────────────

    #[test]
    fn full_health_is_100_percent() {
        assert_eq!(titan(200.0).hp_percent(), 100.0);
    }

    #[test]
    fn half_health_is_50_percent() {
        let mut e = titan(200.0);
        e.current_hp = 100.0;
        assert_eq!(e.hp_percent(), 50.0);
    }

    #[test]
    fn zero_base_hp_returns_zero_percent() {
        let mut e = titan(0.0);
        e.base_hp = 0.0;
        e.current_hp = 0.0;
        assert_eq!(e.hp_percent(), 0.0);
    }

    // ── is_alive / take_damage ────────────────────────────────────────────────

    #[test]
    fn fresh_enemy_is_alive() {
        assert!(titan(100.0).is_alive());
    }

    #[test]
    fn take_damage_reduces_current_hp() {
        let mut e = titan(100.0);
        e.take_damage(30.0);
        assert!((e.current_hp - 70.0).abs() < 0.01);
    }

    #[test]
    fn take_damage_floors_at_zero() {
        let mut e = titan(100.0);
        e.take_damage(9999.0);
        assert_eq!(e.current_hp, 0.0);
        assert!(!e.is_alive());
    }

    #[test]
    fn exact_lethal_damage_kills_enemy() {
        let mut e = titan(100.0);
        e.take_damage(100.0);
        assert!(!e.is_alive());
    }

    // ── stun / tick_stun ─────────────────────────────────────────────────────

    #[test]
    fn apply_stun_sets_stunned_state() {
        let mut e = titan(100.0);
        e.apply_stun(3);
        assert!(e.is_stunned);
        assert_eq!(e.stun_turns_remaining, 3);
    }

    #[test]
    fn tick_stun_decrements_counter() {
        let mut e = titan(100.0);
        e.apply_stun(2);
        e.tick_stun();
        assert_eq!(e.stun_turns_remaining, 1);
        assert!(e.is_stunned);
    }

    #[test]
    fn tick_stun_clears_on_expiry() {
        let mut e = titan(100.0);
        e.apply_stun(1);
        e.tick_stun();
        assert!(!e.is_stunned);
        assert_eq!(e.stun_turns_remaining, 0);
    }

    #[test]
    fn tick_stun_on_non_stunned_is_noop() {
        let mut e = titan(100.0);
        e.tick_stun(); // no-op
        assert!(!e.is_stunned);
    }

    // ── phase_label ───────────────────────────────────────────────────────────

    #[test]
    fn phase_labels_match_spec() {
        let mut e = titan(100.0);
        e.phase = 1; assert_eq!(e.phase_label(), "Phase I");
        e.phase = 2; assert_eq!(e.phase_label(), "Phase II");
        e.phase = 3; assert_eq!(e.phase_label(), "Phase III");
        e.phase = 99; assert_eq!(e.phase_label(), "?");
    }
}
