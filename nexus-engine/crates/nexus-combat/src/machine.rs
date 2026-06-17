use std::marker::PhantomData;

use crate::parry::{AttackDir, ParryOutcome};

// ---------------------------------------------------------------------------
// State markers (zero-sized types — no runtime cost)
// ---------------------------------------------------------------------------

/// The unit is waiting; all actions are available.
pub struct Idle;

/// An attack has been initiated but not yet resolved.
pub struct Attacking;

/// A parry is in progress (any-direction).
pub struct Parrying;

/// A perfect-parry window is open (direction must match announced).
pub struct PerfectParrying;

/// A dodge is in progress.
pub struct Dodging;

// ---------------------------------------------------------------------------
// Combat machine generic over state S
// ---------------------------------------------------------------------------

/// Typestate combat state machine.
///
/// Only valid state transitions are representable at compile time.
/// Illegal sequences (e.g. attacking while dodging, resolving a parry
/// that was never started) are simply not constructible.
pub struct CombatMachine<S> {
    pub hp: f32,
    pub max_hp: f32,
    /// Current combo depth (capped at 5 for the standard machine).
    pub combo_depth: u32,
    state: PhantomData<S>,
}

// ---------------------------------------------------------------------------
// Idle transitions
// ---------------------------------------------------------------------------

impl CombatMachine<Idle> {
    /// Create a new combat unit at full health.
    pub fn new(hp: f32) -> Self {
        CombatMachine {
            hp,
            max_hp: hp,
            combo_depth: 0,
            state: PhantomData,
        }
    }

    /// `Idle → Attacking` — returns the machine in the Attacking state plus the
    /// direction chosen for the telegraphed attack.
    pub fn begin_attack(self, dir: AttackDir) -> (CombatMachine<Attacking>, AttackDir) {
        (
            CombatMachine {
                hp: self.hp,
                max_hp: self.max_hp,
                combo_depth: self.combo_depth,
                state: PhantomData,
            },
            dir,
        )
    }

    /// `Idle → Parrying` — enters the any-direction parry window.
    pub fn begin_parry(self) -> CombatMachine<Parrying> {
        CombatMachine {
            hp: self.hp,
            max_hp: self.max_hp,
            combo_depth: self.combo_depth,
            state: PhantomData,
        }
    }

    /// `Idle → PerfectParrying` — enters the directional parry window, announcing
    /// the expected incoming direction.
    pub fn begin_perfect_parry(self, dir: AttackDir) -> (CombatMachine<PerfectParrying>, AttackDir) {
        (
            CombatMachine {
                hp: self.hp,
                max_hp: self.max_hp,
                combo_depth: self.combo_depth,
                state: PhantomData,
            },
            dir,
        )
    }

    /// `Idle → Dodging` — enters the dodge window; the combo resets on resolution.
    pub fn begin_dodge(self) -> CombatMachine<Dodging> {
        CombatMachine {
            hp: self.hp,
            max_hp: self.max_hp,
            combo_depth: self.combo_depth,
            state: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Attacking transitions
// ---------------------------------------------------------------------------

impl CombatMachine<Attacking> {
    /// `Attacking → Idle` — the attack landed.
    ///
    /// Applies `damage` to `target_hp`, increments combo depth (cap 5),
    /// and returns the machine in Idle state.
    pub fn resolve_hit(mut self, damage: f32, target_hp: &mut f32) -> CombatMachine<Idle> {
        *target_hp -= damage;
        self.combo_depth = (self.combo_depth + 1).min(5);
        CombatMachine {
            hp: self.hp,
            max_hp: self.max_hp,
            combo_depth: self.combo_depth,
            state: PhantomData,
        }
    }

    /// `Attacking → Idle` — the attack was blocked or missed.
    ///
    /// Combo depth resets to 0.
    pub fn resolve_blocked(self) -> CombatMachine<Idle> {
        CombatMachine {
            hp: self.hp,
            max_hp: self.max_hp,
            combo_depth: 0,
            state: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Parrying transitions
// ---------------------------------------------------------------------------

impl CombatMachine<Parrying> {
    /// `Parrying → Idle` — resolve the parry outcome.
    ///
    /// Damage taken depends on the outcome:
    /// - `Perfect`: 0 chip damage.
    /// - `Normal`:  10 % chip damage.
    /// - `Miss`:    full damage.
    ///
    /// HP is clamped to 0 (never negative).
    pub fn resolve(self, outcome: ParryOutcome, incoming_damage: f32) -> (CombatMachine<Idle>, ParryOutcome) {
        let new_hp = match outcome {
            ParryOutcome::Perfect => self.hp,
            ParryOutcome::Normal => self.hp - incoming_damage * 0.1,
            ParryOutcome::Miss => self.hp - incoming_damage,
        };
        let new_hp = new_hp.max(0.0);
        (
            CombatMachine {
                hp: new_hp,
                max_hp: self.max_hp,
                combo_depth: self.combo_depth,
                state: PhantomData,
            },
            outcome,
        )
    }
}

// ---------------------------------------------------------------------------
// PerfectParrying transitions
// ---------------------------------------------------------------------------

impl CombatMachine<PerfectParrying> {
    /// `PerfectParrying → Idle` — resolve the directional parry.
    ///
    /// If `player_dir` matches `announced` the outcome is `Perfect` (no damage).
    /// Otherwise it degrades to a `Normal` parry (10 % chip damage).
    pub fn resolve(
        self,
        announced: AttackDir,
        player_dir: AttackDir,
        incoming_damage: f32,
    ) -> (CombatMachine<Idle>, ParryOutcome) {
        let outcome = if player_dir == announced {
            ParryOutcome::Perfect
        } else {
            ParryOutcome::Normal
        };
        let new_hp = match outcome {
            ParryOutcome::Perfect => self.hp,
            ParryOutcome::Normal => self.hp - incoming_damage * 0.1,
            ParryOutcome::Miss => self.hp - incoming_damage,
        };
        let new_hp = new_hp.max(0.0);
        (
            CombatMachine {
                hp: new_hp,
                max_hp: self.max_hp,
                combo_depth: self.combo_depth,
                state: PhantomData,
            },
            outcome,
        )
    }
}

// ---------------------------------------------------------------------------
// Dodging transitions
// ---------------------------------------------------------------------------

impl CombatMachine<Dodging> {
    /// `Dodging → Idle` — the dodge completed; no damage taken from dodged attack.
    ///
    /// Combo depth resets to 0.
    pub fn resolve(self) -> CombatMachine<Idle> {
        CombatMachine {
            hp: self.hp,
            max_hp: self.max_hp,
            combo_depth: 0,
            state: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime states with data (stunned / dead handled via enum, not typestate,
// because they carry a counter / terminal flag)
// ---------------------------------------------------------------------------

/// Runtime-dispatched state for units that are stunned or dead.
pub enum RuntimeCombatState {
    Stunned { turns_remaining: u32 },
    Dead,
}

/// Tick a stunned unit.
///
/// Returns `Some(turns_remaining_after_tick)` when still stunned, or `None`
/// when the stun expires and the caller should convert back to Idle.
pub fn tick_stunned(turns_remaining: u32) -> Option<u32> {
    if turns_remaining <= 1 {
        None
    } else {
        Some(turns_remaining - 1)
    }
}
