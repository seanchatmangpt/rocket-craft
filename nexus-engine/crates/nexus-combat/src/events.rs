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
    QipScarApplied {
        stack: u32,
        forced_rebirth: bool,
    },
}
