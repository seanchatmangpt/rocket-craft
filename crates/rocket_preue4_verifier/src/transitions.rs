//! GC-MECHBIRTH-002: Branchless Transition Kernels.
//! Scalar reference implementation + table-driven branchless variant.
//! Both must produce bit-identical results for all inputs in [0, MAX_CLASS].

/// Scalar reference transition kernel.
///
/// `failure_risk = clamp((heat + stress + (MAX_CLASS - socket_health)) / 3, 0, MAX_CLASS)`
///
/// Uses saturating arithmetic to guarantee no overflow on u8 inputs.
#[inline]
pub fn scalar_failure_risk(heat: u8, stress: u8, socket_health: u8) -> u8 {
    let degradation = crate::authority::MAX_CLASS.saturating_sub(socket_health);
    let sum = (heat as u16) + (stress as u16) + (degradation as u16);
    // Integer division rounds toward zero; clamp to MAX_CLASS.
    ((sum / 3) as u8).min(crate::authority::MAX_CLASS)
}

/// Table-driven branchless transition for all 16^3 class combinations.
///
/// The lookup table is built once via `build()` and reused across batches.
/// Index encoding: `heat * 256 + stress * 16 + socket_health`
/// (16 possible values per dimension → 4096 entries total).
pub struct TransitionTable {
    table: Vec<u8>,
}

impl TransitionTable {
    /// Build the full 4096-entry lookup table.
    pub fn build() -> Self {
        let size = 16 * 16 * 16; // 4096
        let mut table = vec![0u8; size];
        for heat in 0u8..16 {
            for stress in 0u8..16 {
                for socket_health in 0u8..16 {
                    let idx =
                        (heat as usize) * 256 + (stress as usize) * 16 + (socket_health as usize);
                    table[idx] = scalar_failure_risk(heat, stress, socket_health);
                }
            }
        }
        Self { table }
    }

    /// Perform a branchless table lookup.
    /// Inputs are clamped to [0, 15] before indexing.
    #[inline]
    pub fn lookup(&self, heat: u8, stress: u8, socket_health: u8) -> u8 {
        let idx = (heat.min(15) as usize) * 256
            + (stress.min(15) as usize) * 16
            + (socket_health.min(15) as usize);
        // SAFETY: idx is bounded to [0, 4095] by the .min(15) clamps above;
        // table has exactly 4096 entries built in `build()`.
        self.table[idx]
    }
}

/// Batch scalar damage update — writes damage[i] for every i in state.
pub fn batch_update_damage_scalar(state: &mut crate::authority::AuthorityState) {
    let n = state.damage.len();
    for i in 0..n {
        state.damage[i] =
            scalar_failure_risk(state.heat[i], state.stress[i], state.socket_health[i]);
    }
}

/// Batch table-driven damage update — identical semantics to scalar variant.
pub fn batch_update_damage_table(
    state: &mut crate::authority::AuthorityState,
    table: &TransitionTable,
) {
    let n = state.damage.len();
    for i in 0..n {
        state.damage[i] = table.lookup(state.heat[i], state.stress[i], state.socket_health[i]);
    }
}
