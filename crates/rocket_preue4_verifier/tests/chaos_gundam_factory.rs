//! GC-GUNDAM-FACTORY-001: Chaos tests — each test MUST fail for the EXPECTED reason.
//! These tests verify the refusal boundary law for Gundam Factory: every invalid
//! input must produce a specific RefusalReason, not a panic or silent wrong answer.

use rocket_preue4_verifier::{
    authority::{AuthorityState, MAX_CLASS},
    error::RefusalReason,
    prediction::PredictionState,
    simd::batch_update_damage_simd_equiv,
    transitions::scalar_failure_risk,
};

#[test]
fn chaos_simd_divergence_detected_for_gundam() {
    use rocket_preue4_verifier::simd::verify_simd_scalar_equivalence;
    let heat = vec![15u8; 32];
    let stress = vec![15u8; 32];
    let socket = vec![0u8; 32];
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());

    let mut damage = vec![0u8; 32];
    batch_update_damage_simd_equiv(&heat, &stress, &socket, &mut damage).unwrap();
    let correct_val = damage[10];
    damage[10] = damage[10].wrapping_add(1); // force divergence

    let mut scalar_out = vec![0u8; 32];
    for i in 0..32 {
        scalar_out[i] = scalar_failure_risk(heat[i], stress[i], socket[i]);
    }

    let divergence = scalar_out
        .iter()
        .zip(damage.iter())
        .enumerate()
        .find(|(_, (s, d))| s != d)
        .map(|(i, _)| RefusalReason::SimdScalarDivergence { index: i });

    assert!(divergence.is_some());
    assert!(matches!(
        divergence.unwrap(),
        RefusalReason::SimdScalarDivergence { index: 10 }
    ));
}

#[test]
fn chaos_invalid_class_caught_in_gundam_authority() {
    let mut state = AuthorityState::new(16);
    state.damage[5] = MAX_CLASS + 1; // over boundary
    state.heat[12] = 255; // REFUSED_CLASS

    let errs = state.validate_classes();
    assert!(errs.len() >= 2);

    let has_damage_err = errs.iter().any(|e| {
        matches!(
            e,
            RefusalReason::InvalidAuthorityClass { field, .. } if field.contains("damage")
        )
    });
    assert!(has_damage_err);

    let has_heat_err = errs.iter().any(|e| {
        matches!(
            e,
            RefusalReason::InvalidAuthorityClass { field, .. } if field.contains("heat")
        )
    });
    assert!(has_heat_err);
}

#[test]
fn chaos_mismatched_buffer_lengths_gundam() {
    let mut state = AuthorityState::new(8);
    state.stress.pop(); // length 7 vs others 8

    let result = state.validate_lengths();
    assert!(
        matches!(result, Err(RefusalReason::InvalidAuthorityClass { .. })),
        "Expected InvalidAuthorityClass, got {:?}",
        result
    );
}
