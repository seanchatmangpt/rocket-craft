/// Phantom-typed numeric units that make illegal cross-unit arithmetic a compile error.
use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul};
use serde::{Serialize, Deserialize};

use crate::errors::TypeError;

// ---------------------------------------------------------------------------
// Unit marker types (zero-sized, never instantiated at runtime)
// ---------------------------------------------------------------------------

/// Marker for hit-point quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HpUnit;
/// Marker for gold-currency quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GoldUnit;
/// Marker for damage quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DamageUnit;
/// Marker for mana quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManaUnit;
/// Marker for time-dilation factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDilationUnit;
/// Marker for experience-point quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XpUnit;
/// Marker for armour-rating quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArmorUnit;
/// Marker for combo-multiplier values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComboMultiplierUnit;

// ---------------------------------------------------------------------------
// Generic typed value
// ---------------------------------------------------------------------------

/// A strongly-typed numeric value parameterised by its unit `U`.
///
/// The `PhantomData<U>` makes `Typed<T, HpUnit>` and `Typed<T, DamageUnit>`
/// distinct types at compile time while adding zero runtime cost.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Typed<T, Unit>(pub(crate) T, pub(crate) PhantomData<Unit>);

impl<T: Copy, U> Typed<T, U> {
    /// Wrap a raw value with its unit tag.
    #[inline]
    pub fn new(v: T) -> Self {
        Typed(v, PhantomData)
    }

    /// Extract the raw inner value, discarding the unit tag.
    #[inline]
    pub fn value(self) -> T {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Concrete unit aliases
// ---------------------------------------------------------------------------

/// Hit points — non-negative, capped at `Hp::MAX`.
pub type Hp = Typed<f32, HpUnit>;
/// Gold currency — unsigned, cannot underflow.
pub type Gold = Typed<u32, GoldUnit>;
/// Raw damage before armour reduction.
pub type Damage = Typed<f32, DamageUnit>;
/// Mana pool value.
pub type Mana = Typed<f32, ManaUnit>;
/// Time-dilation factor applied to animation/tick rate.
pub type TimeDilation = Typed<f32, TimeDilationUnit>;
/// Accumulated experience points (lifetime, not per-level).
pub type Xp = Typed<u64, XpUnit>;
/// Armour rating (damage-reduction coefficient).
pub type Armor = Typed<f32, ArmorUnit>;
/// Combo multiplier applied to damage on successive hits.
pub type ComboMultiplier = Typed<f32, ComboMultiplierUnit>;

// ---------------------------------------------------------------------------
// Arithmetic trait implementations (same-unit operands only)
// ---------------------------------------------------------------------------

macro_rules! impl_add_sub {
    ($ty:ty, $inner:ty) => {
        impl Add for $ty {
            type Output = $ty;
            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.0 + rhs.0)
            }
        }

        impl Sub for $ty {
            type Output = $ty;
            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.0 - rhs.0)
            }
        }
    };
}

macro_rules! impl_mul_f32 {
    ($ty:ty) => {
        impl Mul<f32> for $ty {
            type Output = $ty;
            #[inline]
            fn mul(self, rhs: f32) -> Self::Output {
                Self::new(self.0 * rhs)
            }
        }
    };
}

impl_add_sub!(Hp, f32);
impl_mul_f32!(Hp);

impl_add_sub!(Damage, f32);
impl_mul_f32!(Damage);

impl_add_sub!(Mana, f32);
impl_mul_f32!(Mana);

impl_add_sub!(Armor, f32);
impl_mul_f32!(Armor);

impl_add_sub!(ComboMultiplier, f32);
impl_mul_f32!(ComboMultiplier);

impl_add_sub!(TimeDilation, f32);
impl_mul_f32!(TimeDilation);

// Xp uses u64 — add/sub only, no f32 multiply
impl Add for Xp {
    type Output = Xp;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Xp {
    type Output = Xp;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_sub(rhs.0))
    }
}

// Gold uses u32 — wrapping/saturating so it never silently overflows
impl Add for Gold {
    type Output = Gold;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Gold {
    type Output = Gold;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_sub(rhs.0))
    }
}

// ---------------------------------------------------------------------------
// Validated constructors and domain constants
// ---------------------------------------------------------------------------

impl Hp {
    /// Maximum legal hit-point value in the Gundam Nexus ruleset.
    pub const MAX: Hp = Typed(100_000.0, PhantomData);
    /// Zero HP — entity is dead.
    pub const ZERO: Hp = Typed(0.0, PhantomData);

    /// Construct an `Hp` value, returning an error if `v` is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_types::Hp;
    ///
    /// let hp = Hp::new_checked(100.0);
    /// assert!(hp.is_ok());
    /// assert_eq!(hp.unwrap().value(), 100.0);
    ///
    /// let invalid = Hp::new_checked(-5.0);
    /// assert!(invalid.is_err());
    /// ```
    pub fn new_checked(v: f32) -> Result<Self, TypeError> {
        if v < 0.0 {
            Err(TypeError::NegativeHealth(v))
        } else {
            Ok(Self::new(v))
        }
    }

    /// Returns `true` if this entity is at or below zero HP.
    #[inline]
    pub fn is_dead(self) -> bool {
        self.0 <= 0.0
    }
}

impl Gold {
    /// Zero gold balance.
    pub const ZERO: Gold = Typed(0, PhantomData);

    /// Infallible constructor — `u32` cannot be negative.
    #[inline]
    pub fn new_checked(v: u32) -> Self {
        Self::new(v)
    }

    /// Checked addition that returns an error on overflow instead of saturating.
    pub fn checked_add(self, rhs: Gold) -> Result<Gold, TypeError> {
        self.0
            .checked_add(rhs.0)
            .map(Gold::new)
            .ok_or(TypeError::GoldOverflow { current: self.0, added: rhs.0 })
    }
}

impl TimeDilation {
    /// Normal real-time speed.
    pub const NORMAL: TimeDilation = Typed(1.0, PhantomData);
    /// Half-speed tick rate in the Negative Realm zone.
    pub const SLOW: TimeDilation = Typed(0.5, PhantomData);
    /// Maximum accelerated speed during GodKing Phase 3.
    pub const FAST: TimeDilation = Typed(1.3, PhantomData);

    /// Construct a `TimeDilation` factor, returning an error if outside `0.1..=3.0`.
    pub fn new_checked(v: f32) -> Result<Self, TypeError> {
        if !(0.1..=3.0).contains(&v) {
            Err(TypeError::InvalidTimeDilation(v))
        } else {
            Ok(Self::new(v))
        }
    }
}
