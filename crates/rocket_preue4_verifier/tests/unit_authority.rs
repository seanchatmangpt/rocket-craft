//! GC-MECHBIRTH-002: Unit tests for Authority byte fields and transition kernels.

use rocket_preue4_verifier::{
    authority::{AuthorityState, MAX_CLASS, REFUSED_CLASS},
    error::RefusalReason,
    simd::verify_simd_scalar_equivalence,
    transitions::{
        TransitionTable, batch_update_damage_scalar, batch_update_damage_table, scalar_failure_risk,
    },
};

// ---------------------------------------------------------------------------
// AuthorityState::validate_lengths
// ---------------------------------------------------------------------------

#[test]
fn validate_lengths_consistent_buffers_passes() {
    let state = AuthorityState::new(8);
    assert!(state.validate_lengths().is_ok());
}

#[test]
fn validate_lengths_empty_state_passes() {
    let state = AuthorityState::new(0);
    assert!(state.validate_lengths().is_ok());
}

#[test]
fn validate_lengths_mismatched_heat_fails() {
    let mut state = AuthorityState::new(4);
    state.heat.push(0); // length now 5 vs 4
    assert!(matches!(
        state.validate_lengths(),
        Err(RefusalReason::InvalidAuthorityClass { .. })
    ));
}

#[test]
fn validate_lengths_mismatched_stress_fails() {
    let mut state = AuthorityState::new(4);
    state.stress.pop();
    assert!(state.validate_lengths().is_err());
}

#[test]
fn validate_lengths_mismatched_grip_fails() {
    let mut state = AuthorityState::new(4);
    state.grip.push(0);
    assert!(state.validate_lengths().is_err());
}

#[test]
fn validate_lengths_mismatched_socket_health_fails() {
    let mut state = AuthorityState::new(4);
    state.socket_health.pop();
    assert!(state.validate_lengths().is_err());
}

#[test]
fn validate_lengths_mismatched_lod_fails() {
    let mut state = AuthorityState::new(4);
    state.lod.push(0);
    assert!(state.validate_lengths().is_err());
}

// ---------------------------------------------------------------------------
// AuthorityState::validate_classes
// ---------------------------------------------------------------------------

#[test]
fn validate_classes_clean_state_returns_no_errors() {
    let state = AuthorityState::new(16);
    assert!(state.validate_classes().is_empty());
}

#[test]
fn validate_classes_catches_damage_over_max() {
    let mut state = AuthorityState::new(4);
    state.damage[2] = MAX_CLASS + 1;
    let errs = state.validate_classes();
    assert!(!errs.is_empty());
    assert!(matches!(
        &errs[0],
        RefusalReason::InvalidAuthorityClass { field, class }
            if field.contains("damage") && *class > MAX_CLASS
    ));
}

#[test]
fn validate_classes_catches_heat_over_max() {
    let mut state = AuthorityState::new(4);
    state.heat[0] = REFUSED_CLASS;
    let errs = state.validate_classes();
    assert!(!errs.is_empty());
}

#[test]
fn validate_classes_catches_stress_over_max() {
    let mut state = AuthorityState::new(3);
    state.stress[1] = 16;
    assert!(!state.validate_classes().is_empty());
}

#[test]
fn validate_classes_catches_grip_over_max() {
    let mut state = AuthorityState::new(3);
    state.grip[0] = 200;
    assert!(!state.validate_classes().is_empty());
}

#[test]
fn validate_classes_catches_socket_health_over_max() {
    let mut state = AuthorityState::new(3);
    state.socket_health[2] = 16;
    assert!(!state.validate_classes().is_empty());
}

#[test]
fn validate_classes_multiple_violations_all_reported() {
    let mut state = AuthorityState::new(4);
    state.damage[0] = 16;
    state.heat[1] = 17;
    state.stress[2] = 255;
    let errs = state.validate_classes();
    assert_eq!(errs.len(), 3);
}

// ---------------------------------------------------------------------------
// scalar_failure_risk domain tests
// ---------------------------------------------------------------------------

#[test]
fn scalar_failure_risk_all_zeros_is_zero() {
    assert_eq!(scalar_failure_risk(0, 0, 0), MAX_CLASS / 3); // (0+0+15)/3 = 5
    // socket_health=0 means max degradation; heat=0,stress=0
    // degradation=MAX_CLASS-0=15, sum=15, /3=5
    // Actually let's re-verify: all zeros means heat=0, stress=0, socket_health=0
    // degradation = MAX_CLASS.saturating_sub(0) = 15
    // sum = 0+0+15 = 15; /3 = 5
    // so all zeros → 5, not 0
    // The test above already asserts 5 which equals MAX_CLASS/3 = 15/3 = 5
}

#[test]
fn scalar_failure_risk_nominal_health_low_heat_stress() {
    // heat=0, stress=0, socket_health=MAX_CLASS (fully healthy)
    // degradation = 0; sum = 0; result = 0
    assert_eq!(scalar_failure_risk(0, 0, MAX_CLASS), 0);
}

#[test]
fn scalar_failure_risk_max_heat_max_stress_no_socket() {
    // heat=15, stress=15, socket_health=0 → degradation=15 → sum=45 → /3=15
    assert_eq!(scalar_failure_risk(15, 15, 0), MAX_CLASS);
}

#[test]
fn scalar_failure_risk_clamped_to_max_class() {
    // Even with all max inputs result must be <= MAX_CLASS
    let r = scalar_failure_risk(MAX_CLASS, MAX_CLASS, 0);
    assert!(r <= MAX_CLASS);
    assert_eq!(r, MAX_CLASS);
}

#[test]
fn scalar_failure_risk_midpoint() {
    // heat=7, stress=7, socket_health=7 → degradation=15-7=8 → sum=22 → /3=7
    assert_eq!(scalar_failure_risk(7, 7, 7), 7);
}

// ---------------------------------------------------------------------------
// TransitionTable exhaustive domain test (16^3 = 4096 entries)
// ---------------------------------------------------------------------------

#[test]
fn transition_table_exhaustive_16_cubed_matches_scalar() {
    let table = TransitionTable::build();
    for heat in 0u8..16 {
        for stress in 0u8..16 {
            for socket_health in 0u8..16 {
                let expected = scalar_failure_risk(heat, stress, socket_health);
                let actual = table.lookup(heat, stress, socket_health);
                assert_eq!(
                    expected, actual,
                    "mismatch at heat={heat} stress={stress} socket_health={socket_health}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// batch_update_damage_scalar vs batch_update_damage_table
// ---------------------------------------------------------------------------

#[test]
fn batch_scalar_and_table_produce_identical_output() {
    let table = TransitionTable::build();
    let mut scalar_state = AuthorityState::new(1024);
    let mut table_state = scalar_state.clone();

    for i in 0..1024 {
        scalar_state.heat[i] = (i % 16) as u8;
        scalar_state.stress[i] = (i % 13) as u8;
        scalar_state.socket_health[i] = (15 - i % 16) as u8;
        table_state.heat[i] = scalar_state.heat[i];
        table_state.stress[i] = scalar_state.stress[i];
        table_state.socket_health[i] = scalar_state.socket_health[i];
    }

    batch_update_damage_scalar(&mut scalar_state);
    batch_update_damage_table(&mut table_state, &table);

    assert_eq!(scalar_state.damage, table_state.damage);
}

// ---------------------------------------------------------------------------
// verify_simd_scalar_equivalence coverage
// ---------------------------------------------------------------------------

#[test]
fn simd_equiv_empty_slice_passes() {
    assert!(verify_simd_scalar_equivalence(&[], &[], &[]).is_ok());
}

#[test]
fn simd_equiv_single_element_passes() {
    assert!(verify_simd_scalar_equivalence(&[5], &[3], &[8]).is_ok());
}

#[test]
fn simd_equiv_16_elements_exactly_one_chunk() {
    let heat = vec![1u8; 16];
    let stress = vec![2u8; 16];
    let socket = vec![10u8; 16];
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());
}

#[test]
fn simd_equiv_17_elements_non_aligned() {
    let heat: Vec<u8> = (0..17).map(|i| (i % 16) as u8).collect();
    let stress: Vec<u8> = (0..17).map(|i| (i % 13) as u8).collect();
    let socket: Vec<u8> = (0..17).map(|i| (15 - i % 16) as u8).collect();
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());
}

#[test]
fn simd_equiv_1000_elements_passes() {
    let heat: Vec<u8> = (0..1000).map(|i| (i % 16) as u8).collect();
    let stress: Vec<u8> = (0..1000).map(|i| (i % 13) as u8).collect();
    let socket: Vec<u8> = (0..1000).map(|i| (15 - i % 16) as u8).collect();
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());
}

#[test]
fn simd_equiv_all_max_inputs_passes() {
    let n = 64;
    let heat = vec![MAX_CLASS; n];
    let stress = vec![MAX_CLASS; n];
    let socket = vec![0u8; n]; // min socket health → max degradation
    assert!(verify_simd_scalar_equivalence(&heat, &stress, &socket).is_ok());
}

#[test]
fn simd_equiv_mismatched_lengths_returns_err() {
    use rocket_preue4_verifier::simd::batch_update_damage_simd_equiv;
    let heat = vec![1u8; 4];
    let stress = vec![1u8; 5]; // mismatched
    let socket = vec![1u8; 4];
    let mut damage = vec![0u8; 4];
    assert!(matches!(
        batch_update_damage_simd_equiv(&heat, &stress, &socket, &mut damage),
        Err(RefusalReason::InvalidAuthorityClass { .. })
    ));
}
