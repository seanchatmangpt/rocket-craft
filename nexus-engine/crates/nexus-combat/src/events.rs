use serde::{Deserialize, Serialize};

use crate::parry::{AttackDir, ParryOutcome};

/// All typed events produced by the combat system.
///
/// Consumers (e.g. the session crate, the UI layer) subscribe to this stream
/// and react to individual variants without polling game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatEvent {
    /// An attack connected with the target.
    AttackLanded {
        dir: AttackDir,
        damage: f32,
        new_combo: u32,
    },

    /// An attack was blocked or missed — combo on the attacker resets.
    AttackBlocked,

    /// A parry attempt was rereaddressed.
    ParrySuccess {
        outcome: ParryOutcome,
        hp_remaining: f32,
    },

    /// The enemy telegraphed an incoming attack direction.
    EnemyAttackAnnounced { dir: AttackDir },

    /// An enemy transitioned to a new combat phase.
    PhaseTransition { enemy_id: String, new_phase: u8 },

    /// An enemy was defeated.
    EnemyDefeated {
        enemy_id: String,
        xp_reward: u64,
        gold_reward: u32,
    },

    /// The player unit died.
    PlayerDied {
        /// `true` when the death was caused by QIP Scar overflow rather than HP depletion.
        qip_forced: bool,
    },

    /// A stun was applied to a unit.
    StunApplied { turns: u32 },

    /// A combo chain was reset (blocked, dodged, or idle timeout).
    ComboReset,

    /// The combo depth reached the Trans-Am zone (depth >= 4).
    TransAmActivated,

    /// A QIP Scar was applied.
    QipScarApplied { stack: u32, forced_rebirth: bool },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parry::{AttackDir, ParryOutcome};

    // CombatEvent is Clone + Serialize + Deserialize — verify JSON round-trips
    // and that all variants construct without error.

    fn ser(e: &CombatEvent) -> String {
        serde_json::to_string(e).unwrap()
    }
    fn de(s: &str) -> CombatEvent {
        serde_json::from_str(s).unwrap()
    }

    // ── AttackLanded ──────────────────────────────────────────────────────────

    #[test]
    fn attack_landed_round_trips() {
        let e = CombatEvent::AttackLanded { dir: AttackDir::Overhead, damage: 30.0, new_combo: 2 };
        let json = ser(&e);
        assert!(json.contains("\"Overhead\""), "direction must be serialized");
        match de(&json) {
            CombatEvent::AttackLanded { damage, new_combo, .. } => {
                assert!((damage - 30.0).abs() < 0.001);
                assert_eq!(new_combo, 2);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    // ── AttackBlocked ─────────────────────────────────────────────────────────

    #[test]
    fn attack_blocked_round_trips() {
        let e = CombatEvent::AttackBlocked;
        assert!(matches!(de(&ser(&e)), CombatEvent::AttackBlocked));
    }

    // ── ParrySuccess ─────────────────────────────────────────────────────────

    #[test]
    fn parry_success_perfect_round_trips() {
        let e = CombatEvent::ParrySuccess { outcome: ParryOutcome::Perfect, hp_remaining: 85.0 };
        let json = ser(&e);
        match de(&json) {
            CombatEvent::ParrySuccess { outcome, hp_remaining } => {
                assert_eq!(outcome, ParryOutcome::Perfect);
                assert!((hp_remaining - 85.0).abs() < 0.001);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── EnemyAttackAnnounced ──────────────────────────────────────────────────

    #[test]
    fn enemy_attack_announced_serializes_direction() {
        let e = CombatEvent::EnemyAttackAnnounced { dir: AttackDir::Left };
        let json = ser(&e);
        assert!(json.contains("\"Left\""));
    }

    // ── PhaseTransition ───────────────────────────────────────────────────────

    #[test]
    fn phase_transition_round_trips() {
        let e = CombatEvent::PhaseTransition { enemy_id: "Galath".into(), new_phase: 2 };
        match de(&ser(&e)) {
            CombatEvent::PhaseTransition { enemy_id, new_phase } => {
                assert_eq!(enemy_id, "Galath");
                assert_eq!(new_phase, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── EnemyDefeated ────────────────────────────────────────────────────────

    #[test]
    fn enemy_defeated_preserves_rewards() {
        let e = CombatEvent::EnemyDefeated {
            enemy_id: "Galath".into(), xp_reward: 500, gold_reward: 250,
        };
        match de(&ser(&e)) {
            CombatEvent::EnemyDefeated { xp_reward, gold_reward, .. } => {
                assert_eq!(xp_reward, 500);
                assert_eq!(gold_reward, 250);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── PlayerDied ───────────────────────────────────────────────────────────

    #[test]
    fn player_died_normal_is_not_qip_forced() {
        let e = CombatEvent::PlayerDied { qip_forced: false };
        match de(&ser(&e)) {
            CombatEvent::PlayerDied { qip_forced } => assert!(!qip_forced),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn player_died_qip_forced_round_trips() {
        let e = CombatEvent::PlayerDied { qip_forced: true };
        match de(&ser(&e)) {
            CombatEvent::PlayerDied { qip_forced } => assert!(qip_forced),
            other => panic!("{other:?}"),
        }
    }

    // ── StunApplied ──────────────────────────────────────────────────────────

    #[test]
    fn stun_applied_preserves_turn_count() {
        let e = CombatEvent::StunApplied { turns: 3 };
        match de(&ser(&e)) {
            CombatEvent::StunApplied { turns } => assert_eq!(turns, 3),
            other => panic!("{other:?}"),
        }
    }

    // ── QipScarApplied ────────────────────────────────────────────────────────

    #[test]
    fn qip_scar_applied_third_stack_is_forced_rebirth() {
        let e = CombatEvent::QipScarApplied { stack: 3, forced_rebirth: true };
        match de(&ser(&e)) {
            CombatEvent::QipScarApplied { stack, forced_rebirth } => {
                assert_eq!(stack, 3);
                assert!(forced_rebirth);
            }
            other => panic!("{other:?}"),
        }
    }

    // ── TransAmActivated / ComboReset ─────────────────────────────────────────

    #[test]
    fn unit_variants_round_trip() {
        assert!(matches!(de(&ser(&CombatEvent::ComboReset)), CombatEvent::ComboReset));
        assert!(matches!(de(&ser(&CombatEvent::TransAmActivated)), CombatEvent::TransAmActivated));
    }

    // ── Clone ─────────────────────────────────────────────────────────────────

    #[test]
    fn combat_event_is_clone() {
        let e = CombatEvent::StunApplied { turns: 2 };
        let _ = e.clone();
    }
}
