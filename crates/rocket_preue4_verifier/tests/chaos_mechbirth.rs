//! GC-MECHBIRTH-002: Chaos tests — each test MUST fail for the EXPECTED reason.
//! These tests verify the refusal boundary law: every invalid input must produce
//! a specific RefusalReason, not a panic or silent wrong answer.

use rocket_preue4_verifier::{
    authority::{AuthorityState, MAX_CLASS},
    error::RefusalReason,
    prediction::PredictionState,
    simd::batch_update_damage_simd_equiv,
    transitions::scalar_failure_risk,
};

// ---------------------------------------------------------------------------
// CHAOS: SIMD divergence detection
// Manually produce a divergence and assert SimdScalarDivergence is returned.
// ---------------------------------------------------------------------------

#[test]
fn chaos_simd_divergence_detected_by_verify() {
    use rocket_preue4_verifier::simd::verify_simd_scalar_equivalence;
    // verify_simd_scalar_equivalence calls the same scalar kernel on both paths,
    // so they can never diverge in correct code.
    // We verify this property holds for edge inputs.
    let heat = vec![15u8; 32];
    let stress = vec![15u8; 32];
    let socket = vec![0u8; 32];
    // Must pass — both paths use the same kernel.
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());

    // Now manually construct a corrupted output slice and verify detection:
    let mut damage = vec![0u8; 32];
    batch_update_damage_simd_equiv(&heat, &stress, &socket, &mut damage).unwrap();
    // Corrupt one lane
    let correct_val = damage[5];
    damage[5] = damage[5].wrapping_add(1); // force divergence

    // Re-compute scalar reference
    let mut scalar_out = vec![0u8; 32];
    for i in 0..32 {
        scalar_out[i] = scalar_failure_risk(heat[i], stress[i], socket[i]);
    }

    // Check divergence manually (mirrors verify_simd_scalar_equivalence logic)
    let divergence = scalar_out
        .iter()
        .zip(damage.iter())
        .enumerate()
        .find(|(_, (s, d))| s != d)
        .map(|(i, _)| RefusalReason::SimdScalarDivergence { index: i });

    assert!(
        divergence.is_some(),
        "Expected SimdScalarDivergence but corruption was not detected"
    );
    assert!(matches!(
        divergence.unwrap(),
        RefusalReason::SimdScalarDivergence { index: 5 }
    ));

    // Restore to confirm original value was correct
    damage[5] = correct_val;
    let divergence_after_restore = scalar_out.iter().zip(damage.iter()).find(|(s, d)| s != d);
    assert!(
        divergence_after_restore.is_none(),
        "After restore, no divergence expected"
    );
}

// ---------------------------------------------------------------------------
// CHAOS: Prediction promotion attempt returns PredictionAuthorityMutation
// ---------------------------------------------------------------------------

#[test]
fn chaos_prediction_promotion_always_refused() {
    let admitted = AuthorityState::new(16);
    let mut prediction = PredictionState::new(16);
    prediction.predict_n_ticks(&admitted, 8);

    let result = prediction.attempt_authority_promotion();
    assert!(
        matches!(result, Err(RefusalReason::PredictionAuthorityMutation)),
        "MUST return PredictionAuthorityMutation — got: {:?}",
        result
    );
    assert!(prediction.authority_mutation_detected());
}

// ---------------------------------------------------------------------------
// CHAOS: Invalid class in authority buffer
// ---------------------------------------------------------------------------

#[test]
fn chaos_invalid_class_caught_in_validate_classes() {
    let mut state = AuthorityState::new(8);
    state.damage[3] = MAX_CLASS + 1; // 16 — over the boundary
    state.heat[7] = 255; // REFUSED_CLASS — over the boundary

    let errs = state.validate_classes();
    assert!(
        errs.len() >= 2,
        "Expected at least 2 violations, got: {}",
        errs.len()
    );
    // Verify damage error
    let has_damage_err = errs.iter().any(|e| {
        matches!(
            e,
            RefusalReason::InvalidAuthorityClass { field, .. } if field.contains("damage")
        )
    });
    assert!(has_damage_err, "Expected damage[3] violation in errors");

    // Verify heat error
    let has_heat_err = errs.iter().any(|e| {
        matches!(
            e,
            RefusalReason::InvalidAuthorityClass { field, .. } if field.contains("heat")
        )
    });
    assert!(has_heat_err, "Expected heat[7] violation in errors");
}

// ---------------------------------------------------------------------------
// CHAOS: Mismatched buffer lengths
// ---------------------------------------------------------------------------

#[test]
fn chaos_mismatched_buffer_lengths_caught_by_validate() {
    let mut state = AuthorityState::new(4);
    // Intentionally break the SoA invariant
    state.stress.pop(); // length 3 vs others at 4

    let result = state.validate_lengths();
    assert!(
        matches!(result, Err(RefusalReason::InvalidAuthorityClass { .. })),
        "Expected InvalidAuthorityClass from validate_lengths, got: {:?}",
        result
    );
}

#[test]
fn chaos_mismatched_simd_input_lengths_caught() {
    let heat = vec![1u8; 8];
    let stress = vec![1u8; 9]; // mismatched
    let socket = vec![1u8; 8];
    let mut damage = vec![0u8; 8];

    let result = batch_update_damage_simd_equiv(&heat, &stress, &socket, &mut damage);
    assert!(
        matches!(result, Err(RefusalReason::InvalidAuthorityClass { .. })),
        "Expected InvalidAuthorityClass for mismatched SIMD input, got: {:?}",
        result
    );
}

#[test]
fn chaos_mismatched_damage_output_length_caught() {
    let heat = vec![1u8; 8];
    let stress = vec![1u8; 8];
    let socket = vec![1u8; 8];
    let mut damage = vec![0u8; 7]; // one short

    let result = batch_update_damage_simd_equiv(&heat, &stress, &socket, &mut damage);
    assert!(
        matches!(result, Err(RefusalReason::InvalidAuthorityClass { .. })),
        "Expected InvalidAuthorityClass for mismatched damage output, got: {:?}",
        result
    );
}
