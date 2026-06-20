// GENERATED FILE - DO NOT EDIT MANUALLY
// Source: O* = ontology/ggen-packs/mechbirth/schema/mechbirth_lod_geom_motion.ttl
// Pipeline: ggen -> extract_lod_classes.sparql -> semantic_lod.rs.tera
// Synthesis: van der Aalst (worldline classification) + Carmack (flat u8 SoA, zero allocation)
//
// GC-MECHBIRTH-002 Semantic LOD Engine
// Laws encoded in O*:
//   - Near does NOT automatically mean important (distance_class excluded from authority_score)
//   - Prediction relevance does NOT grant authority (excluded from authority_score)
//   - CROWN requires explicit authority reason (mission/damage/threat/process >= 13)
//   - CF-4: All input fields must be in [0, 15]. Values > 15 -> Err(LodRefused).

use crate::error::RefusalReason;

// --- LOD CLASS ENUM ---
// Discriminants derived from O* LodClass taxonomy via extract_lod_classes.sparql

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

// --- INPUT / OUTPUT STRUCTS ---

#[derive(Debug, Clone)]
pub struct LodInputs {
    /// 0=near, 15=far. NOT included in authority_score (O* law).
    pub distance_class: u8,
    /// 0=irrelevant, 15=critical.
    pub mission_relevance: u8,
    pub damage_class: u8,
    pub threat_class: u8,
    /// Included in authority_score but NOT an explicit Crown authority field.
    pub interaction_probability: u8,
    /// POWL step relevance 0-15.
    pub process_step_relevance: u8,
    /// Shadow relevance - pre-warm only. NOT included in authority_score.
    pub prediction_relevance: u8,
}

#[derive(Debug, Clone)]
pub struct LodOutput {
    pub lod_class: LodClass,
    pub projection_priority: u8,
    pub authority_required: bool,
}

// --- CLASSIFY LOD ---

/// Classify LOD for a single object.
///
/// Laws (from O* - do not derive from this code):
/// - Near does NOT automatically mean important.
/// - Prediction relevance cannot grant authority.
/// - CROWN requires explicit authority reason (damage, threat, mission, process).
/// - CF-4: all input fields must be in [0, MAX_CLASS=15].
pub fn classify_lod(inputs: &LodInputs) -> Result<LodOutput, RefusalReason> {
    // CF-4 guard: all class fields must be in [0, 15].
    const MAX_CLASS: u8 = 15;
    let fields: &[(&str, u8)] = &[
        ("distance_class",          inputs.distance_class),
        ("mission_relevance",       inputs.mission_relevance),
        ("damage_class",            inputs.damage_class),
        ("threat_class",            inputs.threat_class),
        ("interaction_probability", inputs.interaction_probability),
        ("process_step_relevance",  inputs.process_step_relevance),
        ("prediction_relevance",    inputs.prediction_relevance),
    ];
    for (name, val) in fields {
        if *val > MAX_CLASS {
            return Err(RefusalReason::LodRefused {
                detail: format!(
                    "input field '{}' value {} exceeds MAX_CLASS {}",
                    name, val, MAX_CLASS
                ),
            });
        }
    }

    // Authority score: max over fields that contributesToAuthorityScore=true.
    // Excludes: distance_class (proximity != importance) and prediction_relevance (O* law).
    let authority_score: u16 = (inputs.mission_relevance as u16)
        .max(inputs.damage_class as u16)
        .max(inputs.threat_class as u16)
        .max(inputs.interaction_probability as u16)
        .max(inputs.process_step_relevance as u16);

    // Tier classification from O* LodClass discriminant thresholds.


    // Crown: authority_score in [13, 15]



    // Primary: authority_score in [9, 12]



    // Secondary: authority_score in [5, 8]



    // Tertiary: authority_score in [2, 4]



    // Background: authority_score in [0, 1]




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

    // CROWN explicit authority guard (O* law: requiresExplicitReason=true, threshold=13).
    // Explicit authority fields: mission_relevance, damage_class, threat_class, process_step_relevance.
    // interaction_probability alone is NOT sufficient for Crown.
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

    Ok(LodOutput { lod_class, projection_priority, authority_required })
}

/// Batch LOD classification. Zero allocation - returns Vec of Results.
pub fn batch_classify(inputs: &[LodInputs]) -> Vec<Result<LodOutput, RefusalReason>> {
    inputs.iter().map(classify_lod).collect()
}