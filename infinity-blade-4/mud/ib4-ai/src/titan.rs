use rand::Rng;
use ib4_core::{enemy::EnemyInstance, types::AttackDir};

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

pub struct TitanAI {
    /// Counts down from 5 after a weapon throw; throw is available when 0.
    pub weapon_throw_cooldown: u32,
}

impl TitanAI {
    pub fn new() -> Self {
        Self { weapon_throw_cooldown: 0 }
    }

    pub fn decide(&mut self, enemy: &EnemyInstance, rng: &mut impl Rng) -> AiDecision {
        let phase = enemy.phase;
        let base_dir = random_dir(rng);

        let (announced, actual, is_bluff) = match phase {
            3 => {
                if rng.gen_bool(0.30) {
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
            2 => format!("The ENRAGED {} lunges {} with terrifying speed!", enemy.name, announced),
            3 => format!("The {} attacks {} — but something feels wrong...", enemy.name, announced),
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

pub fn random_dir(rng: &mut impl Rng) -> AttackDir {
    match rng.gen_range(0..3u32) {
        0 => AttackDir::Overhead,
        1 => AttackDir::Left,
        _ => AttackDir::Right,
    }
}

pub fn different_dir(dir: &AttackDir, rng: &mut impl Rng) -> AttackDir {
    loop {
        let d = random_dir(rng);
        if &d != dir {
            return d;
        }
    }
}
