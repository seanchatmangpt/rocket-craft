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
