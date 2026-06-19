//! Unit tests for Semantic LOD law surfaces.
//!
//! Laws verified:
//! - Near irrelevant → Tertiary/Background (not Crown by proximity alone)
//! - Far mission-critical → Crown/Primary
//! - Damaged socket during repair → Crown (mission + process authority)
//! - Track grip during race → Crown (high interaction + threat class)
//! - Prediction relevance alone does NOT produce Crown
//! - Crown without authority reason → Err(LodDemotedCrownFeature)
//! - batch_classify empty input → empty result

use rocket_preue4_verifier::error::RefusalReason;
use rocket_preue4_verifier::semantic_lod::{LodClass, LodInputs, batch_classify, classify_lod};

fn inputs_all_zero() -> LodInputs {
    LodInputs {
        distance_class: 0,
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    }
}

// ── Near + irrelevant → Background ──────────────────────────────────────────

#[test]
fn near_irrelevant_object_is_background() {
    let inputs = LodInputs {
        distance_class: 0, // very near
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    let out = classify_lod(&inputs).expect("should classify without error");
    // Near distance alone must not elevate to Crown
    assert!(
        matches!(out.lod_class, LodClass::Background | LodClass::Tertiary),
        "Expected Background or Tertiary for irrelevant near object, got {:?}",
        out.lod_class
    );
}

// ── Near + low scores → Tertiary ────────────────────────────────────────────

#[test]
fn near_low_scores_gives_tertiary() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 2,
        damage_class: 2,
        threat_class: 2,
        interaction_probability: 2,
        process_step_relevance: 2,
        prediction_relevance: 14, // high prediction — must NOT grant authority
    };
    let out = classify_lod(&inputs).expect("should classify");
    // authority_score = max(2,2,2,2,2) = 2 → Tertiary
    assert_eq!(out.lod_class, LodClass::Tertiary);
}

// ── Far + mission-critical → Crown ──────────────────────────────────────────

#[test]
fn far_mission_critical_is_crown() {
    let inputs = LodInputs {
        distance_class: 15,    // very far
        mission_relevance: 15, // critical
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    let out = classify_lod(&inputs).expect("should classify");
    assert_eq!(out.lod_class, LodClass::Crown);
    assert_eq!(out.projection_priority, 255);
    assert!(out.authority_required);
}

// ── Damaged socket during repair → Crown ────────────────────────────────────

#[test]
fn damaged_socket_during_repair_is_crown() {
    let inputs = LodInputs {
        distance_class: 5,
        mission_relevance: 15, // socket repair is mission-critical
        damage_class: 14,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 15, // active POWL repair step
        prediction_relevance: 0,
    };
    let out = classify_lod(&inputs).expect("should classify");
    assert_eq!(out.lod_class, LodClass::Crown);
    assert!(out.authority_required);
}

// ── Track grip during race → Crown ──────────────────────────────────────────

#[test]
fn track_grip_during_race_is_crown() {
    let inputs = LodInputs {
        distance_class: 2,
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 14,            // high threat (race opponent)
        interaction_probability: 15, // constant interaction
        process_step_relevance: 13,  // active process step
        prediction_relevance: 0,
    };
    let out = classify_lod(&inputs).expect("should classify");
    // threat_class=14 ≥ 13, process_step_relevance=13 ≥ 13 → authority reason present
    assert_eq!(out.lod_class, LodClass::Crown);
}

// ── Prediction relevance alone does NOT produce Crown ───────────────────────

#[test]
fn prediction_relevance_alone_does_not_crown() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 15, // max prediction — still cannot grant Crown
    };
    // authority_score = 0 → Background — must not be Crown
    let result = classify_lod(&inputs);
    match result {
        Ok(out) => {
            assert_ne!(
                out.lod_class,
                LodClass::Crown,
                "prediction_relevance must not grant Crown"
            );
        }
        Err(RefusalReason::LodDemotedCrownFeature { .. }) => {
            // Also acceptable — Crown was requested but refused
        }
        Err(e) => panic!("unexpected error {:?}", e),
    }
}

// ── Crown without authority reason → Err ────────────────────────────────────

#[test]
fn crown_without_authority_reason_is_refused() {
    // interaction_probability=14 pushes authority_score≥13 → LodClass::Crown
    // but no single explicit authority field (mission/damage/threat/process) ≥ 13
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 14, // only interaction pushes score
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    let result = classify_lod(&inputs);
    assert!(
        matches!(result, Err(RefusalReason::LodDemotedCrownFeature { .. })),
        "Expected LodDemotedCrownFeature, got {:?}",
        result
    );
}

// ── batch_classify empty input → empty result ────────────────────────────────

#[test]
fn batch_classify_empty_input_returns_empty() {
    let results = batch_classify(&[]);
    assert!(results.is_empty());
}

// ── batch_classify mixed input ───────────────────────────────────────────────

#[test]
fn batch_classify_mixed_input() {
    let inputs = vec![
        // Background
        inputs_all_zero(),
        // Crown (mission_relevance=15)
        LodInputs {
            distance_class: 15,
            mission_relevance: 15,
            damage_class: 0,
            threat_class: 0,
            interaction_probability: 0,
            process_step_relevance: 0,
            prediction_relevance: 0,
        },
    ];
    let results = batch_classify(&inputs);
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap().lod_class, LodClass::Background);
    assert_eq!(results[1].as_ref().unwrap().lod_class, LodClass::Crown);
}

// ── Secondary class ──────────────────────────────────────────────────────────

#[test]
fn secondary_class_threshold() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 7,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    let out = classify_lod(&inputs).expect("should classify");
    assert_eq!(out.lod_class, LodClass::Secondary);
    assert_eq!(out.projection_priority, 128);
    assert!(!out.authority_required);
}

// ══════════════════════════════════════════════════════════════════════════════
// CF-4 COUNTERFACTUAL REGRESSION TESTS
// Old code: values > 15 were cast to u16 and silently promoted to Crown.
// Fixed code: LodRefused with detail.
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn cf4_out_of_range_mission_relevance_is_refused() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 255, // would silently have scored 255 → Crown
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    let result = classify_lod(&inputs);
    assert!(
        matches!(result, Err(RefusalReason::LodRefused { .. })),
        "CF-4: out-of-range mission_relevance must return LodRefused, got {:?}",
        result
    );
}

#[test]
fn cf4_out_of_range_damage_class_is_refused() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 0,
        damage_class: 200, // out of range
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 0,
    };
    assert!(matches!(classify_lod(&inputs), Err(RefusalReason::LodRefused { .. })));
}

#[test]
fn cf4_out_of_range_prediction_relevance_refused() {
    let inputs = LodInputs {
        distance_class: 0,
        mission_relevance: 0,
        damage_class: 0,
        threat_class: 0,
        interaction_probability: 0,
        process_step_relevance: 0,
        prediction_relevance: 16, // just over limit
    };
    assert!(matches!(classify_lod(&inputs), Err(RefusalReason::LodRefused { .. })));
}

#[test]
fn all_fields_at_ceiling_15_are_admitted() {
    // 15 is the ceiling — must NOT be refused.
    // All authority fields = 15 ≥ 13 → has_authority_reason = true → Crown admitted.
    let inputs = LodInputs {
        distance_class: 15,
        mission_relevance: 15,
        damage_class: 15,
        threat_class: 15,
        interaction_probability: 15,
        process_step_relevance: 15,
        prediction_relevance: 15,
    };
    let result = classify_lod(&inputs);
    assert!(result.is_ok(), "All-15 inputs must be admitted: {:?}", result);
    assert_eq!(result.unwrap().lod_class, LodClass::Crown);
}
