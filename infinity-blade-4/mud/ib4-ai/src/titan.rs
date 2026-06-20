use ib4_core::{enemy::EnemyInstance, types::AttackDir};
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct AiDecision {
    /// Telegraphed direction shown to the player.
    pub announced_dir: AttackDir,
    /// Direction that actually lands (may differ from announced in Phase 3 bluffs).
    pub actual_dir: AttackDir,
    pub announcement_text: String,
    pub is_bluff: bool,
    /// Phase 3: weapon throw every 5 turns.
    pub uses_weapon_throw: bool,
    /// Only true for GodKing attacks in Phase 2.
    pub applies_qip_scar: bool,
}

#[derive(Debug, Clone)]
pub struct TitanAI {
    /// Counts down from 5 after a weapon throw; throw is available when 0.
    pub weapon_throw_cooldown: u32,
}

impl TitanAI {
    pub fn new() -> Self {
        Self {
            weapon_throw_cooldown: 0,
        }
    }

    pub fn decide(&mut self, enemy: &EnemyInstance, rng: &mut impl RngExt) -> AiDecision {
        let phase = enemy.phase;
        let base_dir = random_dir(rng);

        let (announced, actual, is_bluff) = match phase {
            3 => {
                if rng.random_bool(0.30) {
                    let bluff = different_dir(&base_dir, rng);
                    (bluff, base_dir.clone(), true)
                } else {
                    (base_dir.clone(), base_dir.clone(), false)
                }
            }
            _ => (base_dir.clone(), base_dir.clone(), false),
        };

        let throw = phase == 3 && self.weapon_throw_cooldown == 0;
        if throw {
            self.weapon_throw_cooldown = 5;
        }
        if self.weapon_throw_cooldown > 0 {
            self.weapon_throw_cooldown -= 1;
        }

        let text = match phase {
            1 => format!("The {} telegraphs a {} strike!", enemy.name, announced),
            2 => format!(
                "The ENRAGED {} lunges {} with terrifying speed!",
                enemy.name, announced
            ),
            3 => format!(
                "The {} attacks {} — but something feels wrong...",
                enemy.name, announced
            ),
            _ => format!("The {} attacks {}!", enemy.name, announced),
        };

        AiDecision {
            announced_dir: announced,
            actual_dir: actual,
            announcement_text: text,
            is_bluff,
            uses_weapon_throw: throw,
            applies_qip_scar: false,
        }
    }

    /// Call when enemy HP crosses a phase threshold. Returns new phase if transitioned.
    pub fn check_phase_transition(enemy: &mut EnemyInstance) -> Option<u8> {
        let hp_pct = enemy.hp_percent();
        let new_phase = if hp_pct <= 30.0 {
            3
        } else if hp_pct <= 60.0 {
            2
        } else {
            1
        };

        if new_phase > enemy.phase {
            enemy.phase = new_phase;
            enemy.attack_damage = match new_phase {
                2 => enemy.base_attack_damage * 1.25,
                3 => enemy.base_attack_damage * 1.875, // 1.25 × 1.5
                _ => enemy.base_attack_damage,
            };
            Some(new_phase)
        } else {
            None
        }
    }
}

impl Default for TitanAI {
    fn default() -> Self {
        Self::new()
    }
}

pub fn random_dir(rng: &mut impl RngExt) -> AttackDir {
    match rng.random_range(0..3u32) {
        0 => AttackDir::Overhead,
        1 => AttackDir::Left,
        _ => AttackDir::Right,
    }
}

pub fn different_dir(dir: &AttackDir, rng: &mut impl RngExt) -> AttackDir {
    loop {
        let d = random_dir(rng);
        if &d != dir {
            return d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::{enemy::EnemyInstance, types::TitanType};

    fn enemy(hp: f32) -> EnemyInstance {
        EnemyInstance {
            id: "test".into(),
            name: "Test".into(),
            titan_type: TitanType::Warrior,
            base_hp: hp,
            current_hp: hp,
            base_attack_damage: 40.0,
            attack_damage: 40.0,
            phase: 1,
            bloodline_required: 0,
            reward_xp: 100,
            reward_gold: 100,
            drop_chance: 0.1,
            pending_attack: None,
            is_stunned: false,
            stun_turns_remaining: 0,
            shield_active: false,
            perfect_parries_received: 0,
        }
    }

    // ── check_phase_transition ────────────────────────────────────────────────

    #[test]
    fn above_60pct_hp_stays_phase_1() {
        let mut e = enemy(100.0);
        e.current_hp = 80.0;
        assert!(TitanAI::check_phase_transition(&mut e).is_none());
        assert_eq!(e.phase, 1);
    }

    #[test]
    fn below_60pct_hp_transitions_to_phase_2() {
        let mut e = enemy(100.0);
        e.current_hp = 55.0;
        let p = TitanAI::check_phase_transition(&mut e);
        assert_eq!(p, Some(2));
        assert_eq!(e.phase, 2);
    }

    #[test]
    fn phase_2_attack_damage_is_1_25x_base() {
        let mut e = enemy(100.0);
        e.current_hp = 55.0;
        TitanAI::check_phase_transition(&mut e);
        assert!((e.attack_damage - 40.0 * 1.25).abs() < 0.01);
    }

    #[test]
    fn below_30pct_hp_transitions_to_phase_3() {
        let mut e = enemy(100.0);
        e.current_hp = 25.0;
        let p = TitanAI::check_phase_transition(&mut e);
        assert_eq!(p, Some(3));
        assert_eq!(e.phase, 3);
    }

    #[test]
    fn phase_3_attack_damage_is_1_875x_base() {
        let mut e = enemy(100.0);
        e.current_hp = 25.0;
        TitanAI::check_phase_transition(&mut e);
        assert!((e.attack_damage - 40.0 * 1.875).abs() < 0.01);
    }

    #[test]
    fn phase_does_not_go_backwards() {
        let mut e = enemy(100.0);
        e.current_hp = 25.0;
        TitanAI::check_phase_transition(&mut e); // → phase 3
        assert_eq!(e.phase, 3);
        e.current_hp = 70.0; // heal back up
        assert!(TitanAI::check_phase_transition(&mut e).is_none());
        assert_eq!(e.phase, 3, "phase must never regress");
    }

    #[test]
    fn already_in_phase_3_returns_none() {
        let mut e = enemy(100.0);
        e.current_hp = 10.0;
        e.phase = 3;
        e.attack_damage = e.base_attack_damage * 1.875;
        assert!(TitanAI::check_phase_transition(&mut e).is_none());
    }
}
