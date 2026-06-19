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
