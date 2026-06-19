use crate::error::RefusalReason;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LodClass {
    Crown = 0,
    Primary = 1,
    Secondary = 2,
    Tertiary = 3,
    Background = 4,
    Refused = 255,
}

#[derive(Debug, Clone)]
pub struct LodInputs {
    pub distance_class: u8,    // 0=near, 15=far
    pub mission_relevance: u8, // 0=irrelevant, 15=critical
    pub damage_class: u8,
    pub threat_class: u8,
    pub interaction_probability: u8,
    pub process_step_relevance: u8, // POWL step relevance 0-15
    pub prediction_relevance: u8,   // shadow relevance — does NOT grant authority
}

#[derive(Debug, Clone)]
pub struct LodOutput {
    pub lod_class: LodClass,
    pub projection_priority: u8,
    pub authority_required: bool,
}

/// Classify LOD for a single object.
///
/// Laws:
/// - Near does NOT automatically mean important.
/// - Far does NOT automatically mean irrelevant.
/// - Process relevance can promote.
/// - Prediction relevance can pre-warm but NOT grant authority.
/// - CROWN requires explicit authority reason (damage, threat, mission, process).
pub fn classify_lod(inputs: &LodInputs) -> Result<LodOutput, RefusalReason> {
    // CF-4 guard: all class fields must be in [0, 15]. Values > 15 are rejected
    // rather than silently promoted to Crown through unclamped arithmetic.
    const MAX_CLASS: u8 = 15;
    let fields = [
        ("distance_class",          inputs.distance_class),
        ("mission_relevance",       inputs.mission_relevance),
        ("damage_class",            inputs.damage_class),
        ("threat_class",            inputs.threat_class),
        ("interaction_probability", inputs.interaction_probability),
        ("process_step_relevance",  inputs.process_step_relevance),
        ("prediction_relevance",    inputs.prediction_relevance),
    ];
    for (name, val) in fields {
        if val > MAX_CLASS {
            return Err(RefusalReason::LodRefused {
                detail: format!("input field '{name}' value {val} exceeds MAX_CLASS {MAX_CLASS}"),
            });
        }
    }

    // Compute authority score (ignores prediction_relevance)
    let authority_score = (inputs.mission_relevance as u16)
        .max(inputs.damage_class as u16)
        .max(inputs.threat_class as u16)
        .max(inputs.process_step_relevance as u16)
        .max(inputs.interaction_probability as u16);

    let lod_class = if authority_score >= 13 {
        LodClass::Crown
    } else if authority_score >= 9 {
        LodClass::Primary
    } else if authority_score >= 5 {
        LodClass::Secondary
    } else if authority_score >= 2 {
        LodClass::Tertiary
    } else {
        LodClass::Background
    };

    // CROWN requires explicit authority reason — not just distance or prediction.
    if lod_class == LodClass::Crown {
        let has_authority_reason = inputs.mission_relevance >= 13
            || inputs.damage_class >= 13
            || inputs.threat_class >= 13
            || inputs.process_step_relevance >= 13;
        if !has_authority_reason {
            return Err(RefusalReason::LodDemotedCrownFeature {
                feature: "no_authority_reason_for_crown".into(),
            });
        }
    }

    let authority_required = matches!(lod_class, LodClass::Crown | LodClass::Primary);
    let projection_priority = match lod_class {
        LodClass::Crown => 255,
        LodClass::Primary => 200,
        LodClass::Secondary => 128,
        LodClass::Tertiary => 64,
        LodClass::Background => 16,
        LodClass::Refused => 0,
    };

    Ok(LodOutput {
        lod_class,
        projection_priority,
        authority_required,
    })
}

/// Batch LOD classification.
pub fn batch_classify(inputs: &[LodInputs]) -> Vec<Result<LodOutput, RefusalReason>> {
    inputs.iter().map(classify_lod).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs_with_authority(score: u8) -> LodInputs {
        LodInputs {
            distance_class: 0,
            mission_relevance: score,
            damage_class: 0,
            threat_class: 0,
            interaction_probability: 0,
            process_step_relevance: 0,
            prediction_relevance: 0,
        }
    }

    // ── LodClass ordering ─────────────────────────────────────────────────────

    #[test]
    fn lod_class_crown_is_lowest_value() {
        assert!(LodClass::Crown < LodClass::Primary);
        assert!(LodClass::Primary < LodClass::Secondary);
        assert!(LodClass::Secondary < LodClass::Tertiary);
        assert!(LodClass::Tertiary < LodClass::Background);
    }

    // ── classify_lod thresholds ───────────────────────────────────────────────

    #[test]
    fn score_13_classifies_crown_with_explicit_authority() {
        let inputs = inputs_with_authority(13); // mission_relevance >= 13 → ok
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Crown);
        assert_eq!(out.projection_priority, 255);
        assert!(out.authority_required);
    }

    #[test]
    fn score_13_without_individual_authority_field_returns_error() {
        // authority_score = max(interaction_probability=13, ...) — but not from
        // mission/damage/threat/process → CROWN refused
        let inputs = LodInputs {
            distance_class: 0,
            mission_relevance: 0,
            damage_class: 0,
            threat_class: 0,
            interaction_probability: 13, // drives score but NOT an authority field
            process_step_relevance: 0,
            prediction_relevance: 0,
        };
        let result = classify_lod(&inputs);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RefusalReason::LodDemotedCrownFeature { .. }));
    }

    #[test]
    fn score_9_to_12_classifies_primary() {
        let inputs = inputs_with_authority(9);
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Primary);
        assert_eq!(out.projection_priority, 200);
        assert!(out.authority_required);
    }

    #[test]
    fn score_5_to_8_classifies_secondary() {
        let inputs = inputs_with_authority(5);
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Secondary);
        assert!(!out.authority_required);
    }

    #[test]
    fn score_2_to_4_classifies_tertiary() {
        let inputs = inputs_with_authority(2);
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Tertiary);
        assert_eq!(out.projection_priority, 64);
    }

    #[test]
    fn score_0_to_1_classifies_background() {
        let inputs = inputs_with_authority(0);
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Background);
        assert_eq!(out.projection_priority, 16);
        assert!(!out.authority_required);
    }

    #[test]
    fn prediction_relevance_alone_cannot_grant_crown() {
        let inputs = LodInputs {
            distance_class: 0, mission_relevance: 0, damage_class: 0,
            threat_class: 0, interaction_probability: 0,
            process_step_relevance: 0,
            prediction_relevance: 15, // high but excluded from authority
        };
        let out = classify_lod(&inputs).unwrap();
        assert_eq!(out.lod_class, LodClass::Background);
    }
}
