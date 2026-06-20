/// Zero-Fitness / Zero-Conformance Falsification Suite
///
/// Doctrine: Assume fitness=0 and conformance=0 at the start.
/// Every check must prove conformance through falsification and counterfactuals.
/// Nothing is trusted until independently proven to survive mutation.

#[cfg(test)]
mod zero_fitness_conformance_tests {
    use crate::authority::AuthorityState;
    use crate::generated_constants::{
        MAX_DAMAGE_CLASS, MAX_GRIP_CLASS, MAX_HEAT_CLASS, MAX_LOD_CLASS,
        MAX_PROJECTION_STATE_CLASS, MAX_RECEIPT_STATE_CLASS, MAX_SOCKET_HEALTH_CLASS,
        MAX_STATION_STATE_CLASS, MAX_STRESS_CLASS, MAX_WALKTHROUGH_STATE_CLASS,
    };

    // ─── FALSIFICATION SUITE ─────────────────────────────────────────────────
    // Each test proves the verifier REJECTS an invalid authority state.
    // Starting baseline: fitness=0, conformance=0.

    #[test]
    fn falsify_damage_class_overflow_is_rejected() {
        // MUTATION: damage_class exceeds MAX_DAMAGE_CLASS
        let mut state = AuthorityState::new();
        state.damage_class = MAX_DAMAGE_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: damage_class overflow was admitted — fitness=0 baseline violated"
        );
    }

    #[test]
    fn falsify_heat_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        state.heat_class = MAX_HEAT_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: heat_class overflow was admitted — fitness=0 baseline violated"
        );
    }

    #[test]
    fn falsify_stress_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        state.stress_class = MAX_STRESS_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: stress_class overflow was admitted"
        );
    }

    #[test]
    fn falsify_grip_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        state.grip_class = MAX_GRIP_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: grip_class overflow was admitted"
        );
    }

    #[test]
    fn falsify_lod_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        // LOD has a tighter bound (4) — verify it is not mistakenly set to 15
        state.lod_class = MAX_LOD_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: lod_class exceeded MAX_LOD_CLASS={} but was admitted",
            MAX_LOD_CLASS
        );
    }

    #[test]
    fn falsify_walkthrough_state_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        state.walkthrough_state_class = MAX_WALKTHROUGH_STATE_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: walkthrough_state_class overflow was admitted"
        );
    }

    #[test]
    fn falsify_station_state_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        state.station_state_class = MAX_STATION_STATE_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: station_state_class overflow was admitted"
        );
    }

    #[test]
    fn falsify_receipt_state_class_overflow_is_rejected() {
        let mut state = AuthorityState::new();
        // receipt_state_class has tightest bound (3)
        state.receipt_state_class = MAX_RECEIPT_STATE_CLASS + 1;
        assert!(
            !state.validate_classes(),
            "DEFECT: receipt_state_class exceeded MAX_RECEIPT_STATE_CLASS={} but was admitted",
            MAX_RECEIPT_STATE_CLASS
        );
    }

    // ─── COUNTERFACTUAL SUITE ─────────────────────────────────────────────────
    // Each test proves the verifier ADMITS a valid corrected authority state.
    // The canonical (correct) input should always be accepted.

    #[test]
    fn counterfactual_damage_class_at_max_is_admitted() {
        let mut state = AuthorityState::new();
        state.damage_class = MAX_DAMAGE_CLASS;
        assert!(
            state.validate_classes(),
            "DEFECT: canonical damage_class={} was refused",
            MAX_DAMAGE_CLASS
        );
    }

    #[test]
    fn counterfactual_heat_class_at_max_is_admitted() {
        let mut state = AuthorityState::new();
        state.heat_class = MAX_HEAT_CLASS;
        assert!(
            state.validate_classes(),
            "DEFECT: canonical heat_class={} was refused",
            MAX_HEAT_CLASS
        );
    }

    #[test]
    fn counterfactual_lod_class_at_max_is_admitted() {
        let mut state = AuthorityState::new();
        state.lod_class = MAX_LOD_CLASS;
        assert!(
            state.validate_classes(),
            "DEFECT: canonical lod_class={} was refused — tighter bound must still admit max",
            MAX_LOD_CLASS
        );
    }

    #[test]
    fn counterfactual_receipt_state_class_at_max_is_admitted() {
        let mut state = AuthorityState::new();
        state.receipt_state_class = MAX_RECEIPT_STATE_CLASS;
        assert!(
            state.validate_classes(),
            "DEFECT: canonical receipt_state_class={} was refused",
            MAX_RECEIPT_STATE_CLASS
        );
    }

    #[test]
    fn counterfactual_all_fields_at_zero_is_admitted() {
        // The 0-fitness/0-conformance baseline itself must be a valid admitted state
        let state = AuthorityState::new(); // all defaults = 0
        assert!(
            state.validate_classes(),
            "DEFECT: zero-state (fitness=0, conformance=0 baseline) was refused — invalid rejection"
        );
    }

    #[test]
    fn counterfactual_all_fields_at_max_is_admitted() {
        // Fully saturated state must also be admitted — no field-level overflow at max
        let state = AuthorityState {
            damage_class: MAX_DAMAGE_CLASS,
            heat_class: MAX_HEAT_CLASS,
            stress_class: MAX_STRESS_CLASS,
            grip_class: MAX_GRIP_CLASS,
            socket_health_class: MAX_SOCKET_HEALTH_CLASS,
            lod_class: MAX_LOD_CLASS,
            walkthrough_state_class: MAX_WALKTHROUGH_STATE_CLASS,
            station_state_class: MAX_STATION_STATE_CLASS,
            projection_state_class: MAX_PROJECTION_STATE_CLASS,
            receipt_state_class: MAX_RECEIPT_STATE_CLASS,
        };
        assert!(
            state.validate_classes(),
            "DEFECT: all-max state was refused — exhaustive canonical check failed"
        );
    }

    #[test]
    fn counterfactual_lod_class_is_tighter_than_damage_class() {
        // Conformance check: MAX_LOD_CLASS must be strictly less than MAX_DAMAGE_CLASS
        // This prevents the old hardcoded-15 bug from silently equalizing all bounds
        assert!(
            MAX_LOD_CLASS < MAX_DAMAGE_CLASS,
            "DEFECT: LOD class bound is not tighter than damage_class bound — ontology axiom violated"
        );
    }

    #[test]
    fn counterfactual_receipt_state_class_is_tightest_bound() {
        // receipt_state_class (3) must be the tightest bound among all authority fields
        let all_maxes = [
            MAX_DAMAGE_CLASS,
            MAX_HEAT_CLASS,
            MAX_STRESS_CLASS,
            MAX_GRIP_CLASS,
            MAX_SOCKET_HEALTH_CLASS,
            MAX_LOD_CLASS,
            MAX_WALKTHROUGH_STATE_CLASS,
            MAX_STATION_STATE_CLASS,
            MAX_PROJECTION_STATE_CLASS,
        ];
        for m in all_maxes {
            assert!(
                MAX_RECEIPT_STATE_CLASS <= m,
                "DEFECT: receipt_state_class bound {} is not <= another bound {}",
                MAX_RECEIPT_STATE_CLASS,
                m
            );
        }
    }
}
