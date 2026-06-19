use std::marker::PhantomData;

use crate::parry::{AttackDir, ParryOutcome};

// ---------------------------------------------------------------------------
// State markers (zero-sized types — no runtime cost)
// ---------------------------------------------------------------------------

/// The unit is waiting; all actions are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Idle;

/// An attack has been initiated but not yet rereaddressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attacking;

/// A parry is in progress (any-direction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parrying;

/// A perfect-parry window is open (direction must match announced).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfectParrying;

/// A dodge is in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dodging;

// ---------------------------------------------------------------------------
// Combat machine generic over state S
// ---------------------------------------------------------------------------

/// Typestate combat state machine.
///
/// Only valid state transitions are representable at compile time.
/// Illegal sequences (e.g. attacking while dodging, resolving a parry
/// that was never started) are simply not constructible.
///
/// # Examples
///
/// ```
/// use nexus_combat::machine::{CombatMachine, Idle};
/// use nexus_combat::parry::AttackDir;
///
/// // Create unit
/// let unit = CombatMachine::new(100.0);
/// assert_eq!(unit.hp, 100.0);
///
/// // Idle -> Attacking transition
/// let (attacking, dir) = unit.begin_attack(AttackDir::Overhead);
/// assert_eq!(dir, AttackDir::Overhead);
///
/// // Attacking -> Idle transition on hit
/// let mut enemy_hp = 50.0;
/// let idle = attacking.resolve_hit(15.0, &mut enemy_hp);
/// assert_eq!(enemy_hp, 35.0);
/// assert_eq!(idle.combo_depth, 1);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CombatMachine<S> {
    pub hp: f32,
    pub max_hp: f32,
    /// Current combo depth (capped at 5 for the standard machine).
    pub combo_depth: u32,
    state: PhantomData<S>,
}

/// Errors arising from invalid CombatMachine construction.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CombatBuildError {
    /// Health must be positive.
    #[error("initial health must be specified and non-negative (got {0})")]
    InvalidHealth(f32),

    /// Max health must be >= current health.
    #[error("maximum health ({max}) cannot be less than initial health ({hp})")]
    InvalidMaxHealth {
        /// The initial health configured.
        hp: f32,
        /// The maximum health configured.
        max: f32,
    },
}

/// A builder for [`CombatMachine`] to simplify configuration and validate initial values.
///
/// # Examples
///
/// ```
/// use nexus_combat::machine::{CombatMachineBuilder, Idle};
///
/// let machine = CombatMachineBuilder::new()
///     .hp(100.0)
///     .max_hp(120.0)
///     .combo_depth(0)
///     .build()
///     .unwrap();
///
/// assert_eq!(machine.hp, 100.0);
/// assert_eq!(machine.max_hp, 120.0);
/// ```
#[derive(Debug, Clone)]
pub struct CombatMachineBuilder {
    hp: Option<f32>,
    max_hp: Option<f32>,
    combo_depth: u32,
}

impl CombatMachineBuilder {
    /// Create a new builder with default parameters.
    pub fn new() -> Self {
        Self {
            hp: None,
            max_hp: None,
            combo_depth: 0,
        }
    }

    /// Set the initial HP of the unit.
    pub fn hp(mut self, hp: f32) -> Self {
        self.hp = Some(hp);
        self
    }

    /// Set the maximum HP of the unit. If not set, defaults to initial HP.
    pub fn max_hp(mut self, max_hp: f32) -> Self {
        self.max_hp = Some(max_hp);
        self
    }

    /// Set the initial combo depth.
    pub fn combo_depth(mut self, depth: u32) -> Self {
        self.combo_depth = depth;
        self
    }

    /// Validate the parameters and build a [`CombatMachine`] in [`Idle`] state.
    pub fn build(self) -> Result<CombatMachine<Idle>, CombatBuildError> {
        let hp = self.hp.ok_or(CombatBuildError::InvalidHealth(0.0))?;
        if hp < 0.0 {
            return Err(CombatBuildError::InvalidHealth(hp));
        }
        let max_hp = self.max_hp.unwrap_or(hp);
        if max_hp < hp {
            return Err(CombatBuildError::InvalidMaxHealth { hp, max: max_hp });
        }
        Ok(CombatMachine {
            hp,
            max_hp,
            combo_depth: self.combo_depth,
            state: PhantomData,
        })
    }
}

impl Default for CombatMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the dynamic runtime state of a combat unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CombatState {
    Idle,
    Attacking,
    Parrying,
    PerfectParrying,
    Dodging,
}

/// Errors returned when a state transition is invalid.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error(
    "Illegal combat transition: cannot transition from {current:?} to {target:?}. Reason: {reason}"
)]
pub struct CombatTransitionError {
    pub current: CombatState,
    pub target: CombatState,
    pub reason: String,
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
    pub fn begin_perfect_parry(
        self,
        dir: AttackDir,
    ) -> (CombatMachine<PerfectParrying>, AttackDir) {
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
    pub fn resolve(
        self,
        outcome: ParryOutcome,
        incoming_damage: f32,
    ) -> (CombatMachine<Idle>, ParryOutcome) {
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
