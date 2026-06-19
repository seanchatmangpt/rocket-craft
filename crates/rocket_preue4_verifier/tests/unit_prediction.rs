//! GC-MECHBIRTH-002: Unit tests for the Prediction Shadow Engine.
//! Verifies that prediction NEVER mutates admitted AuthorityState.

use rocket_preue4_verifier::{
    authority::AuthorityState, error::RefusalReason, prediction::PredictionState,
};

// ---------------------------------------------------------------------------
// predict_n_ticks does NOT modify admitted AuthorityState
// ---------------------------------------------------------------------------

#[test]
fn predict_n_ticks_does_not_modify_admitted_state() {
    let admitted = AuthorityState::new(8);
    let admitted_snapshot = admitted.clone();

    let mut prediction = PredictionState::new(8);
    prediction.predict_n_ticks(&admitted, 5);

    // Admitted state must be byte-identical to its snapshot
    assert_eq!(admitted.damage, admitted_snapshot.damage);
    assert_eq!(admitted.heat, admitted_snapshot.heat);
    assert_eq!(admitted.stress, admitted_snapshot.stress);
    assert_eq!(admitted.grip, admitted_snapshot.grip);
    assert_eq!(admitted.socket_health, admitted_snapshot.socket_health);
    assert_eq!(admitted.lod, admitted_snapshot.lod);
}

// ---------------------------------------------------------------------------
// Shadow buffers are updated after predict_n_ticks
// ---------------------------------------------------------------------------

#[test]
fn predict_n_ticks_updates_shadow_buffers() {
    let mut admitted = AuthorityState::new(4);
    admitted.heat[0] = 3;
    admitted.stress[0] = 2;
    admitted.socket_health[0] = 10;

    let mut prediction = PredictionState::new(4);
    prediction.predict_n_ticks(&admitted, 4);

    // future_heat = 3 + 4 = 7, future_stress = 2 + 2 = 4
    assert_eq!(prediction.future_heat[0], 7);
    assert_eq!(prediction.future_stress[0], 4);
    // future_damage should be nonzero for these values
    // scalar_failure_risk(7, 4, 10) = degradation=5, sum=16, /3=5
    assert_eq!(prediction.future_damage[0], 5);
}

// ---------------------------------------------------------------------------
// attempt_authority_promotion always returns Err(PredictionAuthorityMutation)
// ---------------------------------------------------------------------------

#[test]
fn attempt_authority_promotion_always_refuses() {
    let mut prediction = PredictionState::new(4);
    let result = prediction.attempt_authority_promotion();
    assert!(matches!(
        result,
        Err(RefusalReason::PredictionAuthorityMutation)
    ));
}

#[test]
fn attempt_authority_promotion_sets_mutation_detected_flag() {
    let mut prediction = PredictionState::new(4);
    assert!(!prediction.authority_mutation_detected());
    let _ = prediction.attempt_authority_promotion();
    assert!(prediction.authority_mutation_detected());
}

#[test]
fn attempt_authority_promotion_multiple_calls_all_refuse() {
    let mut prediction = PredictionState::new(2);
    for _ in 0..10 {
        assert!(matches!(
            prediction.attempt_authority_promotion(),
            Err(RefusalReason::PredictionAuthorityMutation)
        ));
    }
}

// ---------------------------------------------------------------------------
// discard clears shadow buffers
// ---------------------------------------------------------------------------

#[test]
fn discard_clears_all_shadow_buffers() {
    let admitted = AuthorityState::new(4);
    let mut prediction = PredictionState::new(4);
    prediction.predict_n_ticks(&admitted, 10);

    prediction.discard();

    assert!(prediction.future_damage.iter().all(|&v| v == 0));
    assert!(prediction.future_heat.iter().all(|&v| v == 0));
    assert!(prediction.future_stress.iter().all(|&v| v == 0));
    assert!(prediction.future_grip.iter().all(|&v| v == 0));
    assert!(prediction.future_lod.iter().all(|&v| v == 0));
    assert!(prediction.confidence.iter().all(|&v| v == 0));
}

// ---------------------------------------------------------------------------
// Confidence degrades with increasing ticks
// ---------------------------------------------------------------------------

#[test]
fn confidence_degrades_with_tick_distance() {
    let admitted = AuthorityState::new(4);
    let mut prediction_near = PredictionState::new(4);
    let mut prediction_far = PredictionState::new(4);

    prediction_near.predict_n_ticks(&admitted, 2);
    prediction_far.predict_n_ticks(&admitted, 12);

    // confidence[i] = 15.saturating_sub(ticks)
    // near: 15 - 2 = 13; far: 15 - 12 = 3
    assert_eq!(prediction_near.confidence[0], 13);
    assert_eq!(prediction_far.confidence[0], 3);
    // Near confidence is higher than far confidence
    assert!(prediction_near.confidence[0] > prediction_far.confidence[0]);
}

#[test]
fn confidence_saturates_to_zero_at_max_ticks() {
    let admitted = AuthorityState::new(4);
    let mut prediction = PredictionState::new(4);
    prediction.predict_n_ticks(&admitted, 255); // saturating_sub must not overflow
    assert_eq!(prediction.confidence[0], 0); // 15.saturating_sub(255) = 0
}

// ---------------------------------------------------------------------------
// Prediction reads admitted state as source, writes shadow only
// ---------------------------------------------------------------------------

#[test]
fn predict_reads_admitted_heat_not_shadow() {
    let mut admitted = AuthorityState::new(2);
    admitted.heat[0] = 10;
    admitted.heat[1] = 5;

    let mut prediction = PredictionState::new(2);
    prediction.predict_n_ticks(&admitted, 3);

    // future_heat[0] = 10 + 3 = 13
    assert_eq!(prediction.future_heat[0], 13);
    // future_heat[1] = 5 + 3 = 8
    assert_eq!(prediction.future_heat[1], 8);
}

#[test]
fn predict_clamps_future_heat_to_max_class() {
    use rocket_preue4_verifier::authority::MAX_CLASS;
    let mut admitted = AuthorityState::new(1);
    admitted.heat[0] = MAX_CLASS; // already at max

    let mut prediction = PredictionState::new(1);
    prediction.predict_n_ticks(&admitted, 10);

    // saturating_add(10) then .min(MAX_CLASS) → still MAX_CLASS
    assert_eq!(prediction.future_heat[0], MAX_CLASS);
}

#[test]
fn predict_lod_demoted_to_crown_when_damage_critical() {
    let mut admitted = AuthorityState::new(1);
    // Push heat and stress to max so predicted damage > 10
    admitted.heat[0] = 14;
    admitted.stress[0] = 14;
    admitted.socket_health[0] = 0; // max degradation

    let mut prediction = PredictionState::new(1);
    prediction.predict_n_ticks(&admitted, 0); // 0 ticks — immediate window
    // future_damage = scalar_failure_risk(14, 14, 0)
    // degradation = 15, sum = 14+14+15 = 43, /3 = 14 → > 10
    assert!(prediction.future_damage[0] > 10);
    assert_eq!(prediction.future_lod[0], 0); // demoted to CROWN
}
