//! Unit tests for Geometry Surrogate and Motion Surrogate law surfaces.
//!
//! Geometry laws:
//! - Valid Frame envelope → Ok
//! - WeaponMount without socket → Err(MissingSocket)
//! - Valid Aabb → is_valid() == true
//! - Invalid Aabb (min > max on any axis) → validate() Err(GeometryValidationFailed)
//!
//! Motion laws:
//! - Valid FireWeapon trace (PlantFeet, StabilizeShoulder, Fire, AbsorbRecoil) → Ok, heat+2 stress+1
//! - FireWeapon without PlantFeet → Err(MotionClearanceViolation)
//! - FireWeapon without socket → Err(MissingSocket)
//! - FireWeapon with heat_class≥12 without VentHeat → Err(MotionClearanceViolation)
//! - Run with leg_damage≥12 → Err(MotionClearanceViolation)
//! - Walk with leg_damage≥12 → Ok (degraded, admitted)

use rocket_preue4_verifier::error::RefusalReason;
use rocket_preue4_verifier::geometry::{
    Aabb, ClearanceZone, GeometryEnvelope, PartFamily, SemanticFeature, SocketMount,
    validate_assembly,
};
use rocket_preue4_verifier::motion::{MotionFamily, MotionPhase, MotionTrace};
use rocket_preue4_verifier::semantic_lod::LodClass;

// ═══════════════════════════════════════════════════════════════════════════
// GEOMETRY
// ═══════════════════════════════════════════════════════════════════════════

fn valid_aabb() -> Aabb {
    Aabb {
        min: [0.0, 0.0, 0.0],
        max: [1.0, 2.0, 3.0],
    }
}

fn frame_envelope(part_id: &str) -> GeometryEnvelope {
    GeometryEnvelope {
        part_id: part_id.into(),
        family: PartFamily::Frame,
        bounds: valid_aabb(),
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![],
    }
}

// ── Valid Frame envelope → Ok ────────────────────────────────────────────────

#[test]
fn valid_frame_envelope_passes() {
    let env = frame_envelope("frame-01");
    assert!(env.validate().is_ok());
}

// ── WeaponMount without socket → Err(MissingSocket) ─────────────────────────

#[test]
fn weapon_mount_without_socket_is_refused() {
    let env = GeometryEnvelope {
        part_id: "weapon-01".into(),
        family: PartFamily::WeaponMount,
        bounds: valid_aabb(),
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![],
    };
    let result = env.validate();
    assert!(
        matches!(result, Err(RefusalReason::MissingSocket { .. })),
        "Expected MissingSocket, got {:?}",
        result
    );
}

// ── WeaponMount with socket → Ok ─────────────────────────────────────────────

#[test]
fn weapon_mount_with_socket_passes() {
    let env = GeometryEnvelope {
        part_id: "weapon-02".into(),
        family: PartFamily::WeaponMount,
        bounds: valid_aabb(),
        sockets: vec![SocketMount {
            socket_id: "s-01".into(),
            mount_point: [0.5, 1.0, 0.5],
        }],
        clearance_zones: vec![],
        lod_required_features: vec![],
    };
    assert!(env.validate().is_ok());
}

// ── Valid Aabb → is_valid() == true ─────────────────────────────────────────

#[test]
fn valid_aabb_is_valid() {
    let a = valid_aabb();
    assert!(a.is_valid());
}

// ── Degenerate Aabb (min == max) → still valid ───────────────────────────────

#[test]
fn degenerate_aabb_is_valid() {
    let a = Aabb {
        min: [1.0, 1.0, 1.0],
        max: [1.0, 1.0, 1.0],
    };
    assert!(a.is_valid());
}

// ── Invalid Aabb (min[0] > max[0]) → validate() returns Err ─────────────────

#[test]
fn invalid_aabb_fails_validation() {
    let env = GeometryEnvelope {
        part_id: "bad-bounds-01".into(),
        family: PartFamily::Leg,
        bounds: Aabb {
            min: [5.0, 0.0, 0.0],
            max: [1.0, 2.0, 3.0], // x: 5.0 > 1.0
        },
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![],
    };
    let result = env.validate();
    assert!(
        matches!(result, Err(RefusalReason::GeometryValidationFailed { .. })),
        "Expected GeometryValidationFailed, got {:?}",
        result
    );
}

// ── Invalid Aabb on y-axis ───────────────────────────────────────────────────

#[test]
fn invalid_aabb_y_axis_fails() {
    let env = GeometryEnvelope {
        part_id: "bad-bounds-02".into(),
        family: PartFamily::Shoulder,
        bounds: Aabb {
            min: [0.0, 10.0, 0.0],
            max: [1.0, 2.0, 3.0], // y: 10.0 > 2.0
        },
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![],
    };
    let result = env.validate();
    assert!(matches!(
        result,
        Err(RefusalReason::GeometryValidationFailed { .. })
    ));
}

// ── ArmorPanel with Crown features but no clearance zones → Err ──────────────

#[test]
fn armor_panel_crown_feature_no_clearance_zones_fails() {
    let env = GeometryEnvelope {
        part_id: "armor-01".into(),
        family: PartFamily::ArmorPanel,
        bounds: valid_aabb(),
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![SemanticFeature {
            feature_id: "crown-feature-01".into(),
            required_for_lod: LodClass::Crown,
        }],
    };
    let result = env.validate();
    assert!(
        matches!(result, Err(RefusalReason::MotionClearanceViolation { .. })),
        "Expected MotionClearanceViolation, got {:?}",
        result
    );
}

// ── ArmorPanel with Crown features AND clearance zones → Ok ──────────────────

#[test]
fn armor_panel_with_clearance_zones_passes() {
    let env = GeometryEnvelope {
        part_id: "armor-02".into(),
        family: PartFamily::ArmorPanel,
        bounds: valid_aabb(),
        sockets: vec![],
        clearance_zones: vec![ClearanceZone {
            zone_id: "cz-01".into(),
            bounds: valid_aabb(),
        }],
        lod_required_features: vec![SemanticFeature {
            feature_id: "crown-feature-01".into(),
            required_for_lod: LodClass::Crown,
        }],
    };
    assert!(env.validate().is_ok());
}

// ── validate_assembly: all valid → no failures ───────────────────────────────

#[test]
fn validate_assembly_all_valid() {
    let parts = vec![frame_envelope("f-01"), frame_envelope("f-02")];
    let failures = validate_assembly(&parts);
    assert!(failures.is_empty());
}

// ── validate_assembly: one invalid part reported ─────────────────────────────

#[test]
fn validate_assembly_reports_failure() {
    let bad = GeometryEnvelope {
        part_id: "weapon-bad".into(),
        family: PartFamily::WeaponMount,
        bounds: valid_aabb(),
        sockets: vec![],
        clearance_zones: vec![],
        lod_required_features: vec![],
    };
    let parts = vec![frame_envelope("f-01"), bad];
    let failures = validate_assembly(&parts);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].0, "weapon-bad");
    assert!(matches!(failures[0].1, RefusalReason::MissingSocket { .. }));
}

// ═══════════════════════════════════════════════════════════════════════════
// MOTION
// ═══════════════════════════════════════════════════════════════════════════

fn nominal_fire_trace() -> MotionTrace {
    MotionTrace {
        family: MotionFamily::FireWeapon,
        phases: vec![
            MotionPhase::PlantFeet,
            MotionPhase::StabilizeShoulder,
            MotionPhase::Fire,
            MotionPhase::AbsorbRecoil,
        ],
        socket_available: true,
        heat_class: 5,
        stress_class: 2,
        leg_damage_class: 0,
    }
}

// ── Valid FireWeapon trace → Ok, heat+2 stress+1 ─────────────────────────────

#[test]
fn valid_fire_weapon_trace_succeeds() {
    let trace = nominal_fire_trace();
    let result = trace.validate_and_compute_effects();
    assert_eq!(result, Ok((2, 1)), "Expected delta_heat=2, delta_stress=1");
}

// ── FireWeapon without PlantFeet → Err(MotionClearanceViolation) ─────────────

#[test]
fn fire_weapon_without_plant_feet_is_refused() {
    let trace = MotionTrace {
        phases: vec![
            MotionPhase::StabilizeShoulder,
            MotionPhase::Fire,
            MotionPhase::AbsorbRecoil,
        ],
        ..nominal_fire_trace()
    };
    let result = trace.validate_and_compute_effects();
    assert!(
        matches!(result, Err(RefusalReason::MotionClearanceViolation { .. })),
        "Expected MotionClearanceViolation, got {:?}",
        result
    );
}

// ── FireWeapon without socket → Err(MissingSocket) ───────────────────────────

#[test]
fn fire_weapon_without_socket_is_refused() {
    let trace = MotionTrace {
        socket_available: false,
        ..nominal_fire_trace()
    };
    let result = trace.validate_and_compute_effects();
    assert!(
        matches!(result, Err(RefusalReason::MissingSocket { .. })),
        "Expected MissingSocket, got {:?}",
        result
    );
}

// ── FireWeapon with heat_class≥12 without VentHeat → Err ────────────────────

#[test]
fn fire_weapon_high_heat_without_vent_heat_is_refused() {
    let trace = MotionTrace {
        heat_class: 12,
        phases: vec![
            MotionPhase::PlantFeet,
            MotionPhase::StabilizeShoulder,
            MotionPhase::Fire,
            MotionPhase::AbsorbRecoil,
        ],
        ..nominal_fire_trace()
    };
    let result = trace.validate_and_compute_effects();
    assert!(
        matches!(result, Err(RefusalReason::MotionClearanceViolation { .. })),
        "Expected MotionClearanceViolation for heat_class=12, got {:?}",
        result
    );
}

// ── FireWeapon with heat_class≥12 with VentHeat → Ok ────────────────────────

#[test]
fn fire_weapon_high_heat_with_vent_heat_passes() {
    let trace = MotionTrace {
        heat_class: 12,
        phases: vec![
            MotionPhase::PlantFeet,
            MotionPhase::VentHeat,
            MotionPhase::StabilizeShoulder,
            MotionPhase::Fire,
            MotionPhase::AbsorbRecoil,
        ],
        ..nominal_fire_trace()
    };
    let result = trace.validate_and_compute_effects();
    assert_eq!(result, Ok((2, 1)));
}

// ── Run with leg_damage≥12 → Err(MotionClearanceViolation) ──────────────────

#[test]
fn run_with_high_leg_damage_is_refused() {
    let trace = MotionTrace {
        family: MotionFamily::Run,
        phases: vec![MotionPhase::Stride],
        socket_available: true,
        heat_class: 0,
        stress_class: 0,
        leg_damage_class: 12,
    };
    let result = trace.validate_and_compute_effects();
    assert!(
        matches!(result, Err(RefusalReason::MotionClearanceViolation { .. })),
        "Expected MotionClearanceViolation for Run with leg_damage=12, got {:?}",
        result
    );
}

// ── Walk with leg_damage≥12 → Ok (degraded, admitted) ───────────────────────

#[test]
fn walk_with_high_leg_damage_is_admitted_degraded() {
    let trace = MotionTrace {
        family: MotionFamily::Walk,
        phases: vec![MotionPhase::Stride],
        socket_available: true,
        heat_class: 0,
        stress_class: 0,
        leg_damage_class: 12,
    };
    let result = trace.validate_and_compute_effects();
    assert_eq!(
        result,
        Ok((0, 0)),
        "Walk with high leg damage must be admitted (degraded), not refused"
    );
}

// ── FireWeapon with no phases → Ok (no fire, no effects) ────────────────────

#[test]
fn fire_weapon_no_fire_phase_gives_zero_effects() {
    let trace = MotionTrace {
        family: MotionFamily::FireWeapon,
        phases: vec![MotionPhase::PlantFeet, MotionPhase::StabilizeShoulder],
        socket_available: true,
        heat_class: 0,
        stress_class: 0,
        leg_damage_class: 0,
    };
    let result = trace.validate_and_compute_effects();
    assert_eq!(result, Ok((0, 0)));
}
