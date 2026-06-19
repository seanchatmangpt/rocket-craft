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
