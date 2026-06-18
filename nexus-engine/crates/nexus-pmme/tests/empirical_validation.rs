use nexus_gundam::generated_gundam::{Earth, RotationLimits, AABB};
use nexus_pmme::{
    generate_spec, CulturalProfile, FunctionalRole, Gate, MechAssemblySpec, PlanetaryValues,
    Unvalidated,
};
use std::marker::PhantomData;

fn get_valid_base_spec() -> MechAssemblySpec<Unvalidated, Earth> {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 0.5,
            ambition: 0.5,
            beauty: 0.5,
            community: 0.5,
            order: 0.5,
            knowledge: 0.5,
        },
        _marker: PhantomData,
    };
    generate_spec(cp, FunctionalRole::Warrior)
}

#[test]
fn test_invalid_aabb_min_greater_than_max_bypasses_intersection() {
    let mut spec = get_valid_base_spec();

    // Invert the physical occupancy AABB so min > max
    spec.collision_volume.physical_occupancy = AABB::new([10.0, 10.0, 10.0], [0.0, 0.0, 0.0]);

    spec.collision_volume.interaction_zones = vec![AABB::new([2.0, 2.0, 2.0], [3.0, 3.0, 3.0])];

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail due to inverted AABB, got: {:?}",
        res
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate3);
}

#[test]
fn test_nan_aabb_coordinates_bypass_intersection() {
    let mut spec = get_valid_base_spec();

    spec.collision_volume.physical_occupancy = AABB::new([f32::NAN, 0.0, 0.0], [5.0, 5.0, 5.0]);

    spec.collision_volume.interaction_zones = vec![AABB::new([1.0, 1.0, 1.0], [2.0, 2.0, 2.0])];

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail due to NaN in AABB, got: {:?}",
        res
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate3);
}

#[test]
fn test_extreme_planetary_profiles_handling() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: f32::NAN,
            ambition: f32::INFINITY,
            beauty: f32::NEG_INFINITY,
            community: -999.0,
            order: 999.0,
            knowledge: 0.8,
        },
        _marker: PhantomData::<Earth>,
    };

    let weights = cp.compute_weights();
    assert!(!weights.white_gold_material_bias); // NaN >= 0.8 is false
    assert!(weights.symmetric_joint_layout_enforced); // 999.0 >= 0.8 is true

    let spec = generate_spec(cp, FunctionalRole::Warrior);
    let res = spec.validate();
    assert!(
        res.is_ok(),
        "Expected extreme planetary values to validate without Gate errors, got: {:?}",
        res
    );
}

#[test]
fn test_nan_equipment_mass_bypasses_gate1() {
    let mut spec = get_valid_base_spec();
    spec.equipment_mass = f32::NAN;

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail for NaN equipment mass, got: {:?}",
        res
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_negative_equipment_mass_passes_gate1() {
    let mut spec = get_valid_base_spec();
    spec.equipment_mass = -500.0;

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail for negative equipment mass, got: {:?}",
        res
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_invalid_rotation_limits_pass_gate2() {
    let mut spec = get_valid_base_spec();

    let invalid_limits = RotationLimits {
        min_yaw: 180.0,
        max_yaw: -180.0,
        min_pitch: 90.0,
        max_pitch: -90.0,
        min_roll: 180.0,
        max_roll: -180.0,
    };

    for joint in &mut spec.joints {
        joint.rotation_limits = Some(invalid_limits);
    }

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail for invalid rotation limits, got: {:?}",
        res
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_touching_collision_volumes_fail_gate3() {
    let mut spec = get_valid_base_spec();

    spec.collision_volume.physical_occupancy = AABB::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    spec.collision_volume.interaction_zones = vec![AABB::new([1.0, 0.0, 0.0], [2.0, 1.0, 1.0])];

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Expected validation to fail (Gate3) for touching collision volumes, got: {:?}",
        res
    );
    let err = res.unwrap_err();
    assert_eq!(err.gate, Gate::Gate3);
}

#[test]
fn test_infinitesimal_gap_collision_volumes_pass_gate3() {
    let mut spec = get_valid_base_spec();

    spec.collision_volume.physical_occupancy = AABB::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    spec.collision_volume.clearance_volumes = AABB::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    spec.collision_volume.interaction_zones =
        vec![AABB::new([1.000001, 0.0, 0.0], [2.0, 1.0, 1.0])];

    let res = spec.validate();
    assert!(
        res.is_ok(),
        "Expected validation to pass for non-touching volumes with small gap, got: {:?}",
        res
    );
}
