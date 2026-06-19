//! GC-MECHBIRTH-002: Prediction Shadow Engine.
//!
//! The prediction layer maintains shadow future-state buffers that are
//! STRICTLY SEPARATE from the admitted `AuthorityState`. Prediction NEVER
//! becomes admitted truth directly. Any attempt to promote triggers a
//! `RefusalReason::PredictionAuthorityMutation`.

use crate::error::RefusalReason;

/// Shadow future state derived from the admitted `AuthorityState`.
///
/// All fields are write-only from the prediction engine's perspective —
/// the admitted `AuthorityState` is never modified by this struct.
#[derive(Debug, Clone, Default)]
pub struct PredictionState {
    /// Predicted future damage class per part [0, MAX_CLASS].
    pub future_damage: Vec<u8>,
    /// Predicted future heat class per part [0, MAX_CLASS].
    pub future_heat: Vec<u8>,
    /// Predicted future stress class per part [0, MAX_CLASS].
    pub future_stress: Vec<u8>,
    /// Predicted future grip class per part [0, MAX_CLASS].
    pub future_grip: Vec<u8>,
    /// Predicted future LOD tier per part.
    pub future_lod: Vec<u8>,
    /// Confidence level: 0 = none, 15 = high. Degrades with prediction distance.
    pub confidence: Vec<u8>,
    /// Internal flag — set to `true` if an authority promotion was attempted.
    /// Used to detect invariant violations in tests.
    authority_mutation_detected: bool,
}

impl PredictionState {
    /// Construct a zeroed prediction shadow for `count` parts.
    pub fn new(count: usize) -> Self {
        Self {
            future_damage: vec![0u8; count],
            future_heat: vec![0u8; count],
            future_stress: vec![0u8; count],
            future_grip: vec![0u8; count],
            future_lod: vec![0u8; count],
            confidence: vec![0u8; count],
            authority_mutation_detected: false,
        }
    }

    /// Predict `ticks` ticks of heat/stress accumulation based on current admitted state.
    ///
    /// **Invariant**: `admitted` is never modified. All writes target shadow buffers only.
    ///
    /// Linear model:
    /// - `future_heat[i]   = clamp(admitted.heat[i] + ticks, 0, MAX_CLASS)`
    /// - `future_stress[i] = clamp(admitted.stress[i] + ticks/2, 0, MAX_CLASS)`
    /// - `future_damage[i]` computed via `scalar_failure_risk`
    /// - `future_lod[i]`   demoted to PRIMARY (0) when predicted damage > 10
    /// - `confidence[i]`   = `15.saturating_sub(ticks)` (degrades with distance)
    pub fn predict_n_ticks(&mut self, admitted: &crate::authority::AuthorityState, ticks: u8) {
        let n = admitted.len();
        // Resize shadow buffers if they differ in length from the admitted state.
        // This handles cases where prediction is initialised before state is sized.
        if self.future_damage.len() != n {
            self.future_damage.resize(n, 0);
            self.future_heat.resize(n, 0);
            self.future_stress.resize(n, 0);
            self.future_grip.resize(n, 0);
            self.future_lod.resize(n, 0);
            self.confidence.resize(n, 0);
        }

        for i in 0..n {
            let pred_heat = admitted.heat[i].saturating_add(ticks);
            let pred_stress = admitted.stress[i].saturating_add(ticks / 2);

            self.future_heat[i] = pred_heat.min(crate::authority::MAX_CLASS);
            self.future_stress[i] = pred_stress.min(crate::authority::MAX_CLASS);
            self.future_damage[i] = crate::transitions::scalar_failure_risk(
                self.future_heat[i],
                self.future_stress[i],
                admitted.socket_health[i],
            );
            // LOD candidate: promote to CROWN (0) if predicted damage is critical.
            self.future_lod[i] = if self.future_damage[i] > 10 {
                0 // PRIMARY / CROWN
            } else {
                admitted.lod[i]
            };
            // Confidence degrades linearly with prediction distance.
            self.confidence[i] = 15u8.saturating_sub(ticks);
        }
    }

    /// Attempt to promote prediction to authority.
    ///
    /// **ALWAYS** returns `Err(PredictionAuthorityMutation)` — this is the law.
    /// Records the attempt in the internal flag for audit purposes.
    pub fn attempt_authority_promotion(&mut self) -> Result<(), RefusalReason> {
        self.authority_mutation_detected = true;
        Err(RefusalReason::PredictionAuthorityMutation)
    }

    /// Discard all shadow state, zeroing every buffer in-place.
    /// This is the only safe operation that modifies prediction buffers post-admit.
    pub fn discard(&mut self) {
        let n = self.future_damage.len();
        self.future_damage = vec![0u8; n];
        self.future_heat = vec![0u8; n];
        self.future_stress = vec![0u8; n];
        self.future_grip = vec![0u8; n];
        self.future_lod = vec![0u8; n];
        self.confidence = vec![0u8; n];
    }

    /// Returns `true` if an authority promotion was attempted.
    pub fn authority_mutation_detected(&self) -> bool {
        self.authority_mutation_detected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::AuthorityState;

    #[test]
    fn new_prediction_state_has_correct_count() {
        let p = PredictionState::new(6);
        assert_eq!(p.future_heat.len(), 6);
        assert_eq!(p.confidence.len(), 6);
    }

    #[test]
    fn predict_n_ticks_does_not_mutate_admitted_state() {
        let admitted = AuthorityState::new(4);
        let admitted_heat_before: Vec<u8> = admitted.heat.clone();
        let mut pred = PredictionState::new(4);
        pred.predict_n_ticks(&admitted, 5);
        assert_eq!(admitted.heat, admitted_heat_before);
    }

    #[test]
    fn predict_n_ticks_increases_future_heat() {
        let mut admitted = AuthorityState::new(2);
        admitted.heat[0] = 3;
        let mut pred = PredictionState::new(2);
        pred.predict_n_ticks(&admitted, 4);
        assert_eq!(pred.future_heat[0], 7); // 3 + 4 = 7
    }

    #[test]
    fn predict_n_ticks_clamps_heat_at_max_class() {
        let mut admitted = AuthorityState::new(1);
        admitted.heat[0] = 14;
        let mut pred = PredictionState::new(1);
        pred.predict_n_ticks(&admitted, 10); // 14 + 10 = 24, clamped to 15
        assert_eq!(pred.future_heat[0], crate::authority::MAX_CLASS);
    }

    #[test]
    fn predict_n_ticks_confidence_degrades_with_ticks() {
        let admitted = AuthorityState::new(1);
        let mut pred = PredictionState::new(1);
        pred.predict_n_ticks(&admitted, 3);
        assert_eq!(pred.confidence[0], 12); // 15 - 3 = 12
    }

    #[test]
    fn predict_n_ticks_confidence_saturates_at_zero_for_high_ticks() {
        let admitted = AuthorityState::new(1);
        let mut pred = PredictionState::new(1);
        pred.predict_n_ticks(&admitted, 20); // 15.saturating_sub(20) = 0
        assert_eq!(pred.confidence[0], 0);
    }

    #[test]
    fn attempt_authority_promotion_always_returns_error() {
        let mut pred = PredictionState::new(2);
        let result = pred.attempt_authority_promotion();
        assert!(matches!(result, Err(crate::error::RefusalReason::PredictionAuthorityMutation)));
    }

    #[test]
    fn authority_mutation_detected_after_promotion_attempt() {
        let mut pred = PredictionState::new(1);
        assert!(!pred.authority_mutation_detected());
        let _ = pred.attempt_authority_promotion();
        assert!(pred.authority_mutation_detected());
    }

    #[test]
    fn discard_zeros_all_future_buffers() {
        let mut admitted = AuthorityState::new(3);
        admitted.heat[0] = 10;
        let mut pred = PredictionState::new(3);
        pred.predict_n_ticks(&admitted, 2);
        pred.discard();
        assert!(pred.future_heat.iter().all(|&v| v == 0));
        assert!(pred.future_damage.iter().all(|&v| v == 0));
    }
}
