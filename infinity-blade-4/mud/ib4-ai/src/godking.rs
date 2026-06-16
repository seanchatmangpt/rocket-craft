use rand::Rng;
use ib4_core::{enemy::EnemyInstance, player::PlayerState};
use crate::titan::{AiDecision, TitanAI, random_dir};

#[derive(Debug, Clone, PartialEq)]
pub enum TimeDilationState {
    Normal,
    Slowed,
    Accelerated,
}

#[derive(Debug)]
pub enum GodKingEvent {
    ShieldBroken,
    ForcedRebirth,
    /// Enemy IDs of reinforcements to spawn.
    ReinforceSpawned(Vec<&'static str>),
}

pub struct GodKingAI {
    pub titan_ai: TitanAI,
    pub turn_counter: u32,
    pub time_dilation: TimeDilationState,
    pub reinforcements_spawned: bool,
}

impl GodKingAI {
    pub fn new() -> Self {
        Self {
            titan_ai: TitanAI::new(),
            turn_counter: 0,
            time_dilation: TimeDilationState::Normal,
            reinforcements_spawned: false,
        }
    }

    /// Register a perfect parry on the GodKing. Returns `ShieldBroken` once 3 parries land.
    pub fn register_perfect_parry(enemy: &mut EnemyInstance) -> Option<GodKingEvent> {
        if !enemy.shield_active {
            return None;
        }
        enemy.perfect_parries_received += 1;
        if enemy.perfect_parries_received >= 3 {
            enemy.shield_active = false;
            enemy.phase = 2;
            enemy.attack_damage = enemy.base_attack_damage * 1.25;
            Some(GodKingEvent::ShieldBroken)
        } else {
            None
        }
    }

    /// Apply a QIP Scar stack to the player. Returns `ForcedRebirth` when stacks reach 3.
    pub fn apply_qip_scar(player: &mut PlayerState) -> Option<GodKingEvent> {
        player.qip_scar_stacks += 1;
        if player.qip_scar_stacks >= 3 {
            player.qip_scar_stacks = 0;
            Some(GodKingEvent::ForcedRebirth)
        } else {
            None
        }
    }

    pub fn decide(
        &mut self,
        enemy: &EnemyInstance,
        rng: &mut impl Rng,
    ) -> (AiDecision, Vec<GodKingEvent>) {
        self.turn_counter += 1;
        let mut events: Vec<GodKingEvent> = Vec::new();

        // Phase 3 effects: time dilation every 3 turns; reinforcements at turn 6.
        if enemy.phase == 3 {
            if self.turn_counter % 3 == 0 {
                self.time_dilation = match rng.gen_range(0..3u32) {
                    0 => TimeDilationState::Slowed,
                    1 => TimeDilationState::Accelerated,
                    _ => TimeDilationState::Normal,
                };
            }
            if !self.reinforcements_spawned && self.turn_counter >= 6 {
                self.reinforcements_spawned = true;
                events.push(GodKingEvent::ReinforceSpawned(vec!["LightTitan", "ShadowTitan"]));
            }
        }

        let dir = random_dir(rng);

        let dilation_text = match self.time_dilation {
            TimeDilationState::Slowed => " [Reality SLOWS — time bends to your will!]",
            TimeDilationState::Accelerated => " [Reality ACCELERATES — the God King blurs!]",
            TimeDilationState::Normal => "",
        };

        let galath_text = match enemy.phase {
            1 => format!("Galath's hard-light shield blazes as he strikes {}!{}", dir, dilation_text),
            2 => format!("Galath lunges with twin blades — {}!{}", dir, dilation_text),
            3 => format!("GALATH FRACTURES REALITY — {} strike!{}", dir, dilation_text),
            _ => format!("Galath attacks {}!", dir),
        };

        let decision = AiDecision {
            announced_dir: dir.clone(),
            actual_dir: dir,
            announcement_text: galath_text,
            is_bluff: false,
            uses_weapon_throw: false,
            applies_qip_scar: enemy.phase == 2,
        };

        (decision, events)
    }

    /// Returns the damage multiplier based on current time dilation state.
    pub fn damage_multiplier(&self) -> f32 {
        match self.time_dilation {
            TimeDilationState::Slowed => 0.7,
            TimeDilationState::Accelerated => 1.3,
            TimeDilationState::Normal => 1.0,
        }
    }
}

impl Default for GodKingAI {
    fn default() -> Self {
        Self::new()
    }
}
