use proptest::prelude::*;

use nexus_combat::{
    combo::StandardCombo,
    damage::{calculate_damage, QipScarTracker},
    parry::{AttackDir, ParryOutcome, ParryResolver},
};

// ---------------------------------------------------------------------------
// proptest strategies
// ---------------------------------------------------------------------------

fn any_dir() -> impl Strategy<Value = AttackDir> {
    prop_oneof![
        Just(AttackDir::Overhead),
        Just(AttackDir::Left),
        Just(AttackDir::Right),
    ]
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

proptest! {
    /// Combo multiplier must never *decrease* as consecutive hits land.
    #[test]
    fn combo_mult_monotone_on_hit(hits in 0usize..10) {
        let mut chain = StandardCombo::new(2);
        let mut prev_mult = chain.multiplier();
        for _ in 0..hits {
            chain.on_hit();
            prop_assert!(chain.multiplier() >= prev_mult,
                "multiplier decreased after a hit: {} -> {}", prev_mult, chain.multiplier());
            prev_mult = chain.multiplier();
        }
    }

    /// After `idle` non-attack turns (where `idle >= reset_after_turns = 2`),
    /// the combo depth must be 0.
    #[test]
    fn combo_resets_after_idle(depth in 1usize..5, idle in 2u32..10) {
        let mut chain = StandardCombo::new(2);
        for _ in 0..depth {
            chain.on_hit();
        }
        prop_assert!(chain.depth() > 0, "depth should be > 0 after hits");
        for _ in 0..idle {
            chain.on_non_attack_turn();
        }
        prop_assert_eq!(chain.depth(), 0, "combo should have reset after idle turns");
    }

    /// A perfect-parry counter must always deal at least as much damage as a
    /// plain (non-counter) hit with the same base, combo, and armor values.
    #[test]
    fn perfect_parry_counter_always_stronger(
        base in 1.0f32..1000.0,
        armor in 0.0f32..50.0,
    ) {
        let normal  = calculate_damage(base, 1.0, 0.0, 1.0, false, armor);
        let perfect = calculate_damage(base, 1.0, 0.0, 1.0, true,  armor);
        prop_assert!(perfect >= normal,
            "perfect parry counter ({}) should be >= normal ({})", perfect, normal);
    }

    /// Damage must always be at least 1.0, regardless of armor value.
    #[test]
    fn damage_floor_always_one(
        base  in 0.1f32..100.0,
        armor in 0.0f32..10_000.0,
    ) {
        let dmg = calculate_damage(base, 1.0, 0.0, 1.0, false, armor);
        prop_assert!(dmg >= 1.0,
            "damage floor violated: got {}", dmg);
    }

    /// `ParryResolver::resolve` must return `Perfect` iff the player direction
    /// exactly matches the announced direction, and `Normal` otherwise.
    #[test]
    fn perfect_parry_requires_exact_dir(
        announced in any_dir(),
        guessed   in any_dir(),
    ) {
        let outcome = ParryResolver::resolve(announced, Some(guessed));
        if announced == guessed {
            prop_assert_eq!(outcome, ParryOutcome::Perfect,
                "exact direction match should yield Perfect");
        } else {
            prop_assert_eq!(outcome, ParryOutcome::Normal,
                "direction mismatch should yield Normal, not Perfect");
        }
    }

    /// Any-direction parry (player_dir = None) always yields Normal.
    #[test]
    fn any_dir_parry_always_normal(announced in any_dir()) {
        let outcome = ParryResolver::resolve(announced, None);
        prop_assert_eq!(outcome, ParryOutcome::Normal,
            "any-direction parry should always be Normal");
    }
}

// ---------------------------------------------------------------------------
// Deterministic unit tests
// ---------------------------------------------------------------------------

/// The GodKing shield must break on the 3rd cumulative perfect parry.
#[test]
fn godking_shield_breaks_after_3_perfect_parries() {
    // 2 parries so far + this one = 3 → should break
    let (_, broke) =
        ParryResolver::resolve_shield_parry(AttackDir::Overhead, Some(AttackDir::Overhead), 2);
    assert!(broke, "3rd perfect parry should break the GodKing shield");

    // 1 parry so far + this one = 2 → should NOT break yet
    let (_, not_broke) =
        ParryResolver::resolve_shield_parry(AttackDir::Left, Some(AttackDir::Left), 1);
    assert!(!not_broke, "2nd perfect parry should NOT break the shield");
}

/// A wrong-direction parry attempt against the GodKing shield does not break it.
#[test]
fn godking_shield_wrong_dir_does_not_break() {
    // 2 parries so far, but this parry is Normal (wrong direction) → not broken
    let (outcome, broke) =
        ParryResolver::resolve_shield_parry(AttackDir::Overhead, Some(AttackDir::Left), 2);
    assert_eq!(outcome, ParryOutcome::Normal);
    assert!(!broke, "wrong-direction parry should not break the shield");
}

/// QIP Scar must force a rebirth exactly at the 3rd stack.
#[test]
fn qip_scar_forces_rebirth_at_3_stacks() {
    let mut tracker = QipScarTracker::new();
    assert!(!tracker.apply_scar(), "1st scar should not force rebirth");
    assert!(!tracker.apply_scar(), "2nd scar should not force rebirth");
    assert!(tracker.apply_scar(), "3rd scar should force rebirth");
}

/// After a reset, the QIP Scar tracker requires another 3 stacks.
#[test]
fn qip_scar_resets_cleanly() {
    let mut tracker = QipScarTracker::new();
    tracker.apply_scar();
    tracker.apply_scar();
    tracker.apply_scar();
    tracker.reset();
    assert_eq!(tracker.stacks, 0);
    assert!(!tracker.apply_scar(), "1st scar after reset should not force rebirth");
}

/// Typestate machine: verify the basic happy path compiles and produces
/// correct values.
#[test]
fn combat_machine_happy_path() {
    use nexus_combat::machine::CombatMachine;
    use nexus_combat::parry::{AttackDir, ParryOutcome};

    let player = CombatMachine::<nexus_combat::machine::Idle>::new(100.0);
    assert_eq!(player.hp, 100.0);

    // begin_attack
    let (attacking, dir) = player.begin_attack(AttackDir::Overhead);
    assert_eq!(dir, AttackDir::Overhead);

    // resolve_hit: deal 20 damage to a dummy target
    let mut enemy_hp = 80.0f32;
    let idle = attacking.resolve_hit(20.0, &mut enemy_hp);
    assert_eq!(enemy_hp, 60.0);
    assert_eq!(idle.combo_depth, 1);

    // begin_parry → resolve Normal parry
    let parrying = idle.begin_parry();
    let (idle2, outcome) = parrying.resolve(ParryOutcome::Normal, 30.0);
    assert_eq!(outcome, ParryOutcome::Normal);
    // 10 % chip of 30 = 3 damage
    assert!((idle2.hp - 97.0).abs() < 1e-6, "hp should be 97 after Normal parry chip");

    // begin_dodge → resolve
    let dodging = idle2.begin_dodge();
    let idle3 = dodging.resolve();
    assert_eq!(idle3.combo_depth, 0, "dodge resets combo");
}

/// Combo depth never exceeds the const generic cap.
#[test]
fn combo_depth_capped_at_max() {
    let mut chain = StandardCombo::new(100); // high reset threshold
    for _ in 0..20 {
        chain.on_hit();
    }
    assert_eq!(chain.depth(), 5, "StandardCombo depth must not exceed 5");
}

/// Trans-Am zone activates at depth 4, not before.
#[test]
fn trans_am_zone_activates_at_depth_4() {
    let mut chain = StandardCombo::new(100);
    for i in 0..5 {
        if i < 3 {
            assert!(!chain.is_trans_am_zone(), "should not be in Trans-Am zone at depth {i}");
        }
        chain.on_hit();
    }
    assert!(chain.is_trans_am_zone(), "should be in Trans-Am zone at depth >= 4");
}
