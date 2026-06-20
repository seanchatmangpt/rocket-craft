use crate::parry::ParryOutcome;
use ib4_core::types::{AttackDir, MagicType, StatusEffect};

#[derive(Debug, Clone)]
pub enum CombatEvent {
    PlayerAttacked {
        dir: AttackDir,
        damage: f32,
        combo_depth: u32,
        multiplier: f32,
        is_crit: bool,
    },
    EnemyAttacked {
        dir: AttackDir,
        damage: f32,
    },
    ParryResult {
        outcome: ParryOutcome,
        dir: AttackDir,
    },
    PlayerDodged,
    MagicUsed {
        magic: MagicType,
        damage: f32,
        heal: f32,
        effect: Option<StatusEffect>,
    },
    BurnTick {
        target: String,
        damage: f32,
    },
    StatusApplied {
        target: String,
        effect: StatusEffect,
        turns: u32,
    },
    ComboReset,
    PhaseTransition {
        enemy_name: String,
        new_phase: u8,
    },
    EnemyStunned {
        turns: u32,
    },
    GodKingShieldHit {
        parries_so_far: u32,
    },
    GodKingShieldBroken,
    QIPScarApplied {
        stacks: u32,
    },
    ForcedRebirth,
    PlayerDefeated,
    EnemyDefeated {
        xp: u64,
        gold: u32,
    },
    TimeDilation {
        label: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parry::ParryOutcome;
    use ib4_core::types::{AttackDir, MagicType, StatusEffect};

    // ── CombatEvent variants construct and clone ──────────────────────────────

    #[test]
    fn player_attacked_event_stores_fields() {
        let e = CombatEvent::PlayerAttacked {
            dir: AttackDir::Overhead, damage: 55.0, combo_depth: 3,
            multiplier: 2.0, is_crit: false,
        };
        if let CombatEvent::PlayerAttacked { dir, damage, combo_depth, multiplier, is_crit } = e {
            assert_eq!(dir, AttackDir::Overhead);
            assert!((damage - 55.0).abs() < 0.001);
            assert_eq!(combo_depth, 3);
            assert!((multiplier - 2.0).abs() < 0.001);
            assert!(!is_crit);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn enemy_attacked_event_stores_dir_and_damage() {
        let e = CombatEvent::EnemyAttacked { dir: AttackDir::Left, damage: 30.0 };
        if let CombatEvent::EnemyAttacked { dir, damage } = e {
            assert_eq!(dir, AttackDir::Left);
            assert!((damage - 30.0).abs() < 0.001);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn parry_result_event_stores_outcome_and_dir() {
        let e = CombatEvent::ParryResult {
            outcome: ParryOutcome::PerfectParry, dir: AttackDir::Right,
        };
        if let CombatEvent::ParryResult { outcome, dir } = e {
            assert_eq!(outcome, ParryOutcome::PerfectParry);
            assert_eq!(dir, AttackDir::Right);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn magic_used_event_stores_all_fields() {
        let e = CombatEvent::MagicUsed {
            magic: MagicType::Ice, damage: 80.0, heal: 0.0,
            effect: Some(StatusEffect::Freeze),
        };
        if let CombatEvent::MagicUsed { magic, damage, heal, effect } = e {
            assert_eq!(magic, MagicType::Ice);
            assert!((damage - 80.0).abs() < 0.001);
            assert_eq!(heal, 0.0);
            assert_eq!(effect, Some(StatusEffect::Freeze));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn unit_variants_construct() {
        let _ = CombatEvent::PlayerDodged;
        let _ = CombatEvent::ComboReset;
        let _ = CombatEvent::GodKingShieldBroken;
        let _ = CombatEvent::ForcedRebirth;
        let _ = CombatEvent::PlayerDefeated;
    }

    #[test]
    fn enemy_defeated_event_stores_xp_and_gold() {
        let e = CombatEvent::EnemyDefeated { xp: 500, gold: 250 };
        if let CombatEvent::EnemyDefeated { xp, gold } = e {
            assert_eq!(xp, 500);
            assert_eq!(gold, 250);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn combat_events_are_clone() {
        let e = CombatEvent::EnemyAttacked { dir: AttackDir::Overhead, damage: 10.0 };
        let e2 = e.clone();
        if let CombatEvent::EnemyAttacked { dir, .. } = e2 {
            assert_eq!(dir, AttackDir::Overhead);
        }
    }

    #[test]
    fn burn_tick_stores_target_and_damage() {
        let e = CombatEvent::BurnTick { target: "Titan".into(), damage: 15.0 };
        if let CombatEvent::BurnTick { target, damage } = e {
            assert_eq!(target, "Titan");
            assert!((damage - 15.0).abs() < 0.001);
        }
    }

    #[test]
    fn godking_shield_hit_stores_parry_count() {
        let e = CombatEvent::GodKingShieldHit { parries_so_far: 2 };
        if let CombatEvent::GodKingShieldHit { parries_so_far } = e {
            assert_eq!(parries_so_far, 2);
        }
    }
}
