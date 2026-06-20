use crate::titan::{random_dir, AiDecision, TitanAI};
use ib4_core::{enemy::EnemyInstance, player::PlayerState};
use rand::{Rng, RngExt};

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
            if self.turn_counter.is_multiple_of(3) {
                self.time_dilation = match rng.random_range(0..3u32) {
                    0 => TimeDilationState::Slowed,
                    1 => TimeDilationState::Accelerated,
                    _ => TimeDilationState::Normal,
                };
            }
            if !self.reinforcements_spawned && self.turn_counter >= 6 {
                self.reinforcements_spawned = true;
                events.push(GodKingEvent::ReinforceSpawned(vec![
                    "LightTitan",
                    "ShadowTitan",
                ]));
            }
        }

        let dir = random_dir(rng);

        let dilation_text = match self.time_dilation {
            TimeDilationState::Slowed => " [Reality SLOWS — time bends to your will!]",
            TimeDilationState::Accelerated => " [Reality ACCELERATES — the God King blurs!]",
            TimeDilationState::Normal => "",
        };

        let galath_text = match enemy.phase {
            1 => format!(
                "Galath's hard-light shield blazes as he strikes {}!{}",
                dir, dilation_text
            ),
            2 => format!("Galath lunges with twin blades — {}!{}", dir, dilation_text),
            3 => format!(
                "GALATH FRACTURES REALITY — {} strike!{}",
                dir, dilation_text
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ib4_core::{enemy::EnemyInstance, player::PlayerState, types::TitanType};

    fn galath() -> EnemyInstance {
        EnemyInstance {
            id: "CorruptedGalath".into(),
            name: "Galath".into(),
            titan_type: TitanType::GodKing,
            base_hp: 2000.0,
            current_hp: 2000.0,
            base_attack_damage: 120.0,
            attack_damage: 120.0,
            phase: 1,
            bloodline_required: 20,
            reward_xp: 2000,
            reward_gold: 5000,
            drop_chance: 0.03,
            pending_attack: None,
            is_stunned: false,
            stun_turns_remaining: 0,
            shield_active: true,
            perfect_parries_received: 0,
        }
    }

    // ── register_perfect_parry ────────────────────────────────────────────────

    #[test]
    fn parry_on_inactive_shield_returns_none() {
        let mut e = galath();
        e.shield_active = false;
        assert!(GodKingAI::register_perfect_parry(&mut e).is_none());
    }

    #[test]
    fn two_parries_do_not_break_shield() {
        let mut e = galath();
        GodKingAI::register_perfect_parry(&mut e);
        let result = GodKingAI::register_perfect_parry(&mut e);
        assert!(result.is_none(), "shield needs 3 parries, not 2");
        assert!(e.shield_active, "shield must still be active after 2 parries");
    }

    #[test]
    fn third_parry_breaks_shield_and_returns_event() {
        let mut e = galath();
        GodKingAI::register_perfect_parry(&mut e);
        GodKingAI::register_perfect_parry(&mut e);
        let event = GodKingAI::register_perfect_parry(&mut e);
        assert!(matches!(event, Some(GodKingEvent::ShieldBroken)));
        assert!(!e.shield_active, "shield must be deactivated after 3 parries");
    }

    #[test]
    fn shield_break_advances_to_phase_2() {
        let mut e = galath();
        for _ in 0..3 {
            GodKingAI::register_perfect_parry(&mut e);
        }
        assert_eq!(e.phase, 2);
        assert!((e.attack_damage - 120.0 * 1.25).abs() < 0.01);
    }

    // ── apply_qip_scar ────────────────────────────────────────────────────────

    #[test]
    fn first_two_qip_scars_return_none() {
        let mut p = PlayerState::new("Siris");
        assert!(GodKingAI::apply_qip_scar(&mut p).is_none());
        assert!(GodKingAI::apply_qip_scar(&mut p).is_none());
        assert_eq!(p.qip_scar_stacks, 2);
    }

    #[test]
    fn third_qip_scar_triggers_forced_rebirth() {
        let mut p = PlayerState::new("Siris");
        GodKingAI::apply_qip_scar(&mut p);
        GodKingAI::apply_qip_scar(&mut p);
        let event = GodKingAI::apply_qip_scar(&mut p);
        assert!(matches!(event, Some(GodKingEvent::ForcedRebirth)));
        assert_eq!(p.qip_scar_stacks, 0, "stacks must reset after ForcedRebirth");
    }

    // ── damage_multiplier ─────────────────────────────────────────────────────

    #[test]
    fn normal_dilation_multiplier_is_1_0() {
        let mut ai = GodKingAI::new();
        ai.time_dilation = TimeDilationState::Normal;
        assert_eq!(ai.damage_multiplier(), 1.0);
    }

    #[test]
    fn slowed_dilation_multiplier_is_0_7() {
        let mut ai = GodKingAI::new();
        ai.time_dilation = TimeDilationState::Slowed;
        assert_eq!(ai.damage_multiplier(), 0.7);
    }

    #[test]
    fn accelerated_dilation_multiplier_is_1_3() {
        let mut ai = GodKingAI::new();
        ai.time_dilation = TimeDilationState::Accelerated;
        assert_eq!(ai.damage_multiplier(), 1.3);
    }
}
