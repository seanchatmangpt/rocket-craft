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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::{AuthorityState, MAX_CLASS};

    // ── scalar_failure_risk ───────────────────────────────────────────────────

    #[test]
    fn zero_inputs_give_zero_risk() {
        // heat=0, stress=0, socket=MAX → degradation=0 → sum=0 → risk=0
        assert_eq!(scalar_failure_risk(0, 0, MAX_CLASS), 0);
    }

    #[test]
    fn max_heat_stress_zero_socket_gives_max_class() {
        // heat=15, stress=15, socket=0 → degradation=15 → sum=45 → 45/3=15=MAX_CLASS
        assert_eq!(scalar_failure_risk(15, 15, 0), MAX_CLASS);
    }

    #[test]
    fn formula_matches_expected_midpoint() {
        // heat=6, stress=6, socket=12 → degradation=3 → sum=15 → 15/3=5
        assert_eq!(scalar_failure_risk(6, 6, 12), 5);
    }

    #[test]
    fn clamp_prevents_exceeding_max_class() {
        // All inputs saturating-max: result must never exceed MAX_CLASS
        for h in 0u8..=15 {
            for s in 0u8..=15 {
                for sh in 0u8..=15 {
                    let v = scalar_failure_risk(h, s, sh);
                    assert!(v <= MAX_CLASS, "risk={v} > MAX_CLASS for h={h},s={s},sh={sh}");
                }
            }
        }
    }

    // ── TransitionTable ───────────────────────────────────────────────────────

    #[test]
    fn table_lookup_matches_scalar_for_all_inputs() {
        let table = TransitionTable::build();
        for h in 0u8..16 {
            for s in 0u8..16 {
                for sh in 0u8..16 {
                    let scalar = scalar_failure_risk(h, s, sh);
                    let tbl = table.lookup(h, s, sh);
                    assert_eq!(scalar, tbl, "mismatch at h={h},s={s},sh={sh}");
                }
            }
        }
    }

    #[test]
    fn table_lookup_clamps_oversized_input() {
        let table = TransitionTable::build();
        // input 20 is clamped to 15 internally
        let clamped = table.lookup(20, 20, 0);
        let at_max = table.lookup(15, 15, 0);
        assert_eq!(clamped, at_max);
    }

    // ── batch updates ─────────────────────────────────────────────────────────

    #[test]
    fn batch_scalar_update_writes_correct_damage() {
        let mut state = AuthorityState::new(2);
        state.heat[0] = 6;
        state.stress[0] = 6;
        state.socket_health[0] = 12;
        batch_update_damage_scalar(&mut state);
        assert_eq!(state.damage[0], 5); // formula: (6+6+3)/3 = 5
        assert_eq!(state.damage[1], 0); // all zeros → risk 0
    }

    #[test]
    fn batch_table_update_matches_scalar_batch() {
        let mut scalar_state = AuthorityState::new(4);
        let mut table_state = AuthorityState::new(4);
        for i in 0..4 {
            let v = (i * 3) as u8;
            scalar_state.heat[i] = v;
            scalar_state.stress[i] = v;
            table_state.heat[i] = v;
            table_state.stress[i] = v;
        }
        batch_update_damage_scalar(&mut scalar_state);
        let table = TransitionTable::build();
        batch_update_damage_table(&mut table_state, &table);
        assert_eq!(scalar_state.damage, table_state.damage);
    }
}
