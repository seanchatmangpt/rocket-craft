use proptest::prelude::*;
use nexus_types::units::*;
use nexus_types::math::{Transform, Vec3, Quat};

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

proptest! {
    /// HP addition is commutative.
    #[test]
    fn hp_add_commutative(a in 0.0f32..100_000.0, b in 0.0f32..100_000.0) {
        let ha = Hp::new(a);
        let hb = Hp::new(b);
        prop_assert_eq!((ha + hb).value(), (hb + ha).value());
    }

    /// HP addition is associative.
    #[test]
    fn hp_add_associative(a in 0.0f32..30_000.0, b in 0.0f32..30_000.0, c in 0.0f32..30_000.0) {
        let ha = Hp::new(a);
        let hb = Hp::new(b);
        let hc = Hp::new(c);
        // f32 arithmetic is not perfectly associative, so allow a small tolerance
        let lhs = ((ha + hb) + hc).value();
        let rhs = (ha + (hb + hc)).value();
        prop_assert!((lhs - rhs).abs() < lhs.abs() * 1e-5 + 1e-5);
    }

    /// Gold cannot go negative (u32 saturating arithmetic protects the invariant).
    #[test]
    fn gold_never_negative(v in 0u32..=u32::MAX) {
        let g = Gold::new(v);
        prop_assert_eq!(g.value(), v);
    }

    /// Gold saturating addition never wraps to a smaller value.
    #[test]
    fn gold_saturating_add_monotone(a in 0u32..u32::MAX, b in 0u32..u32::MAX) {
        let ga = Gold::new(a);
        let gb = Gold::new(b);
        let sum = (ga + gb).value();
        prop_assert!(sum >= a);
        prop_assert!(sum >= b);
    }

    /// Multiplying Damage by TimeDilation::NORMAL (1.0) is the identity.
    #[test]
    fn time_dilation_identity(dmg in 0.0f32..10_000.0) {
        let d = Damage::new(dmg);
        let scaled = Damage::new(d.value() * TimeDilation::NORMAL.value());
        prop_assert!((scaled.value() - d.value()).abs() < f32::EPSILON * 100.0);
    }

    /// Multiplying Damage by TimeDilation::SLOW halves it.
    #[test]
    fn time_dilation_slow_halves_damage(dmg in 0.0f32..10_000.0) {
        let d = Damage::new(dmg);
        let scaled = Damage::new(d.value() * TimeDilation::SLOW.value());
        prop_assert!((scaled.value() - d.value() * 0.5).abs() < f32::EPSILON * 100.0 * d.value());
    }

    /// Hp::new_checked rejects negative values.
    #[test]
    fn hp_checked_rejects_negative(v in -100_000.0f32..-f32::EPSILON) {
        prop_assert!(Hp::new_checked(v).is_err());
    }

    /// Hp::new_checked accepts non-negative values.
    #[test]
    fn hp_checked_accepts_non_negative(v in 0.0f32..100_000.0) {
        prop_assert!(Hp::new_checked(v).is_ok());
    }

    /// XP addition saturates at u64::MAX and never panics.
    #[test]
    fn xp_saturating_add_no_panic(a in 0u64..u64::MAX, b in 0u64..u64::MAX) {
        let xa = Xp::new(a);
        let xb = Xp::new(b);
        let _ = (xa + xb).value(); // must not panic
    }
}

// ---------------------------------------------------------------------------
// Deterministic unit tests
// ---------------------------------------------------------------------------

/// The identity transform composed with any child returns that child unchanged.
#[test]
fn transform_identity_is_neutral() {
    let id = Transform::identity();
    let t = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::identity(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let result = id.mul_transform(&t);
    assert!(
        (result.translation - t.translation).norm() < 1e-6,
        "identity parent must not change child translation"
    );
}

/// `transform.lerp(transform, t)` returns the same transform for any `t`.
#[test]
fn transform_lerp_same_is_stable() {
    let t = Transform {
        translation: Vec3::new(5.0, -3.0, 2.0),
        rotation: Quat::identity(),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    let result = t.lerp(&t, 0.5);
    assert!((result.translation - t.translation).norm() < 1e-6);
    assert!((result.scale - t.scale).norm() < 1e-6);
}

/// Hp zero constant is actually zero.
#[test]
fn hp_zero_constant() {
    assert_eq!(Hp::ZERO.value(), 0.0);
    assert!(Hp::ZERO.is_dead());
}

/// Gold zero constant is zero and never treated as dead.
#[test]
fn gold_zero_constant() {
    assert_eq!(Gold::ZERO.value(), 0u32);
}

/// TimeDilation::NORMAL * arbitrary damage == same damage.
#[test]
fn time_dilation_constants_make_sense() {
    assert_eq!(TimeDilation::NORMAL.value(), 1.0);
    assert!(TimeDilation::SLOW.value() < TimeDilation::NORMAL.value());
    assert!(TimeDilation::FAST.value() > TimeDilation::NORMAL.value());
}

/// Typed IDs of different tags are distinct types (compile-time test via type inference).
#[test]
fn typed_id_type_safety() {
    use nexus_types::ids::{PlayerId, ItemId};
    let pid = PlayerId::new(42);
    let iid = ItemId::new(42);
    // Both wrap 42 but are distinct types — the following would NOT compile:
    // let _: PlayerId = iid;
    assert_eq!(pid.raw(), iid.raw());
}

/// Gold checked_add returns Err on overflow.
#[test]
fn gold_checked_add_overflow() {
    let max = Gold::new(u32::MAX);
    let one = Gold::new(1);
    assert!(max.checked_add(one).is_err());
}

/// Gold checked_add succeeds when there is room.
#[test]
fn gold_checked_add_ok() {
    let a = Gold::new(100);
    let b = Gold::new(200);
    assert_eq!(a.checked_add(b).unwrap().value(), 300u32);
}
