
//! Prediction Shadow Engine.
//! ⚠️ GENERATED FILE — do NOT edit by hand.
//! Source of truth: ontology/mechbirth.ttl
//! Generator:       ggen/templates/prediction.rs.tera
//! SPARQL:          ggen/sparql/extract_prediction_fields.sparql
//!
//! Shadow-Only Invariant (mb:ShadowOnlyInvariant):
//! Prediction NEVER writes to admitted AuthorityState.
//! attempt_authority_promotion() ALWAYS returns Err(PredictionAuthorityMutation).

use crate::error::RefusalReason;

/// Shadow future state derived from the admitted `AuthorityState`.
///
/// Shadow fields generated from extract_prediction_fields.sparql:
/// - `confidence`: 15.saturating_sub(ticks)
/// - `future_damage`: scalar_failure_risk(future_heat, future_stress, admitted.socket_health)
/// - `future_heat`: clamp(admitted.heat + ticks, 0, MAX_CLASS)
/// - `future_lod`: if future_damage > 10 { 0 } else { admitted.lod }
/// - `future_stress`: clamp(admitted.stress + ticks/2, 0, MAX_CLASS)

#[derive(Debug, Clone, Default)]
pub struct PredictionState {

    /// Formula: `15.saturating_sub(ticks)`
    pub confidence: Vec<u8>,

    /// Formula: `scalar_failure_risk(future_heat, future_stress, admitted.socket_health)`
    pub future_damage: Vec<u8>,

    /// Formula: `clamp(admitted.heat + ticks, 0, MAX_CLASS)`
    pub future_heat: Vec<u8>,

    /// Formula: `if future_damage > 10 { 0 } else { admitted.lod }`
    pub future_lod: Vec<u8>,

    /// Formula: `clamp(admitted.stress + ticks/2, 0, MAX_CLASS)`
    pub future_stress: Vec<u8>,

    /// Internal audit flag — set when authority promotion is attempted.
    authority_mutation_detected: bool,
}

impl PredictionState {
    /// Construct a zeroed prediction shadow for `count` parts.
    pub fn new(count: usize) -> Self {
        Self {

            confidence: vec![0u8; count],

            future_damage: vec![0u8; count],

            future_heat: vec![0u8; count],

            future_lod: vec![0u8; count],

            future_stress: vec![0u8; count],

            authority_mutation_detected: false,
        }
    }

    /// Predict `ticks` ticks of accumulation from the admitted state.
    ///
    /// **Invariant**: `admitted` is NEVER modified. All writes target shadow buffers only.
    pub fn predict_n_ticks(
        &mut self,
        admitted: &crate::authority::AuthorityState,
        ticks: u8,
    ) {
        let n = admitted.len();
        if self.future_damage.len() != n {

            self.confidence.resize(n, 0);

            self.future_damage.resize(n, 0);

            self.future_heat.resize(n, 0);

            self.future_lod.resize(n, 0);

            self.future_stress.resize(n, 0);

        }

        for i in 0..n {
            let pred_heat   = admitted.heat[i].saturating_add(ticks);
            let pred_stress = admitted.stress[i].saturating_add(ticks / 2);

            self.future_heat[i]   = pred_heat.min(crate::authority::MAX_CLASS);
            self.future_stress[i] = pred_stress.min(crate::authority::MAX_CLASS);
            self.future_damage[i] = crate::transitions::scalar_failure_risk(
                self.future_heat[i],
                self.future_stress[i],
                admitted.socket_health[i],
            );
            self.future_lod[i] = if self.future_damage[i] > 10 {
                0 // PRIMARY / CROWN
            } else {
                admitted.lod[i]
            };
            self.confidence[i] = 15u8.saturating_sub(ticks);
        }
    }

    /// Attempt to promote prediction to authority.
    ///
    /// **ALWAYS** returns `Err(PredictionAuthorityMutation)` — this is the law.
    pub fn attempt_authority_promotion(&mut self) -> Result<(), RefusalReason> {
        self.authority_mutation_detected = true;
        Err(RefusalReason::PredictionAuthorityMutation)
    }

    /// Discard all shadow state — zero every buffer in-place.
    pub fn discard(&mut self) {
        let n = self.future_damage.len();

        self.confidence = vec![0u8; n];

        self.future_damage = vec![0u8; n];

        self.future_heat = vec![0u8; n];

        self.future_lod = vec![0u8; n];

        self.future_stress = vec![0u8; n];

    }

    /// Returns `true` if an authority promotion was attempted.
    pub fn authority_mutation_detected(&self) -> bool {
        self.authority_mutation_detected
    }
}