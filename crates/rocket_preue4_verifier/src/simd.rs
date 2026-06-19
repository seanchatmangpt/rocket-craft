//! GC-MECHBIRTH-002: SIMD/SIMDe Equivalence Layer.
//!
//! This module proves semantic equivalence between the scalar reference path and
//! a chunked-arithmetic "SIMD-equivalent" path using stable Rust (no nightly).
//!
//! Full SIMDe FFI integration with C intrinsic wrappers is a documented
//! GC-MECHBIRTH-003 RESIDUAL. This milestone establishes the correctness boundary.

use crate::error::RefusalReason;

/// Process `n` elements in chunks of 16 lanes using explicit chunked arithmetic.
///
/// Semantically equivalent to executing `scalar_failure_risk` on each element,
/// structured to mirror the data flow a 128-bit SIMD kernel would follow.
///
/// Returns `Err(InvalidAuthorityClass)` if input slice lengths are mismatched.
///
/// **GC-MECHBIRTH-003 RESIDUAL**: Replace inner loop with `std::simd` or SIMDe
/// FFI once nightly portable-simd or a C shim is admitted to the workspace.
pub fn batch_update_damage_simd_equiv(
    heat: &[u8],
    stress: &[u8],
    socket_health: &[u8],
    damage: &mut [u8],
) -> Result<(), RefusalReason> {
    let n = heat.len();
    if stress.len() != n || socket_health.len() != n || damage.len() != n {
        return Err(RefusalReason::InvalidAuthorityClass {
            field: "simd input lengths".into(),
            class: 0,
        });
    }

    // --- 16-lane chunked pass -------------------------------------------
    let mut i = 0usize;
    while i + 16 <= n {
        // Unrolled across 16 lanes to mirror the SIMD register width.
        for j in 0..16 {
            damage[i + j] = crate::transitions::scalar_failure_risk(
                heat[i + j],
                stress[i + j],
                socket_health[i + j],
            );
        }
        i += 16;
    }

    // --- Remainder lanes (tail < 16) -------------------------------------
    while i < n {
        damage[i] = crate::transitions::scalar_failure_risk(heat[i], stress[i], socket_health[i]);
        i += 1;
    }

    Ok(())
}

/// Verifies that the SIMD-equivalent path produces bit-identical output to the
/// scalar reference path for the provided inputs.
///
/// Returns `Err(SimdScalarDivergence { index })` at the first divergent lane.
pub fn verify_simd_scalar_equivalence(
    heat: &[u8],
    stress: &[u8],
    socket_health: &[u8],
) -> Result<(), RefusalReason> {
    let n = heat.len();
    // Guard mismatched lengths before any indexing — the scalar loop below would
    // panic rather than return Err without this check.
    if stress.len() != n || socket_health.len() != n {
        return Err(RefusalReason::InvalidAuthorityClass {
            field: "simd_equiv input lengths".into(),
            class: 0,
        });
    }
    let mut scalar_out = vec![0u8; n];
    let mut simd_out = vec![0u8; n];

    // Scalar reference pass
    for i in 0..n {
        scalar_out[i] =
            crate::transitions::scalar_failure_risk(heat[i], stress[i], socket_health[i]);
    }

    // SIMD-equivalent pass
    batch_update_damage_simd_equiv(heat, stress, socket_health, &mut simd_out)?;

    // Lane-by-lane comparison
    for i in 0..n {
        if scalar_out[i] != simd_out[i] {
            return Err(RefusalReason::SimdScalarDivergence { index: i });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── batch_update_damage_simd_equiv ────────────────────────────────────────

    #[test]
    fn simd_matches_scalar_for_small_input() {
        let heat =   [0u8, 5, 10, 15];
        let stress = [0u8, 5, 10, 15];
        let sh =     [15u8, 10, 5, 0];
        let mut damage = [0u8; 4];
        batch_update_damage_simd_equiv(&heat, &stress, &sh, &mut damage).unwrap();
        for i in 0..4 {
            let expected = crate::transitions::scalar_failure_risk(heat[i], stress[i], sh[i]);
            assert_eq!(damage[i], expected, "divergence at i={i}");
        }
    }

    #[test]
    fn simd_processes_exact_16_lane_chunk() {
        let heat   = vec![3u8; 16];
        let stress = vec![3u8; 16];
        let sh     = vec![9u8; 16];
        let mut damage = vec![0u8; 16];
        batch_update_damage_simd_equiv(&heat, &stress, &sh, &mut damage).unwrap();
        let expected = crate::transitions::scalar_failure_risk(3, 3, 9);
        assert!(damage.iter().all(|&v| v == expected));
    }

    #[test]
    fn simd_processes_more_than_16_with_remainder() {
        let n = 20usize;
        let heat   = vec![2u8; n];
        let stress = vec![4u8; n];
        let sh     = vec![12u8; n];
        let mut damage = vec![0u8; n];
        batch_update_damage_simd_equiv(&heat, &stress, &sh, &mut damage).unwrap();
        let expected = crate::transitions::scalar_failure_risk(2, 4, 12);
        assert!(damage.iter().all(|&v| v == expected));
    }

    #[test]
    fn simd_returns_error_on_mismatched_lengths() {
        let heat   = vec![0u8; 4];
        let stress = vec![0u8; 3]; // wrong length
        let sh     = vec![15u8; 4];
        let mut damage = vec![0u8; 4];
        assert!(batch_update_damage_simd_equiv(&heat, &stress, &sh, &mut damage).is_err());
    }

    // ── verify_simd_scalar_equivalence ────────────────────────────────────────

    #[test]
    fn equivalence_holds_for_all_class_values() {
        // Run on a representative sample spanning the full [0,15] range.
        let vals: Vec<u8> = (0..16).collect();
        let result = verify_simd_scalar_equivalence(&vals, &vals, &vals);
        assert!(result.is_ok(), "equivalence failed: {result:?}");
    }

    #[test]
    fn equivalence_holds_for_zero_length_input() {
        let result = verify_simd_scalar_equivalence(&[], &[], &[]);
        assert!(result.is_ok());
    }
}
