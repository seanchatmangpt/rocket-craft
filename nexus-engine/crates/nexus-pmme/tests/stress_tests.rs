use nexus_gundam::generated_gundam::{Earth, Mars, RotationLimits, Venus, AABB};
use nexus_pmme::*;
use std::marker::PhantomData;

fn create_default_spec() -> MechAssemblySpec<Unvalidated, Earth> {
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
    generate_spec(cp, FunctionalRole::Worker)
}

#[test]
fn test_stress_equipment_mass_nan() {
    let mut spec = create_default_spec();
    spec.equipment_mass = f32::NAN;

    let res = spec.validate();
    assert!(res.is_err(), "NaN mass must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_equipment_mass_negative() {
    let mut spec = create_default_spec();
    spec.equipment_mass = -1000.0;

    let res = spec.validate();
    assert!(res.is_err(), "Negative mass must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_equipment_mass_infinity() {
    let mut spec = create_default_spec();
    spec.equipment_mass = f32::INFINITY;

    let res = spec.validate();
    assert!(res.is_err(), "Infinity mass must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_invalid_aabb_min_greater_than_max() {
    let mut spec = create_default_spec();
    spec.collision_volume.physical_occupancy = AABB::new([10.0, 10.0, 10.0], [0.0, 0.0, 0.0]);

    let res = spec.validate();
    assert!(res.is_err(), "Invalid AABB min > max must fail Gate 3");
    assert_eq!(res.unwrap_err().gate, Gate::Gate3);
}

#[test]
fn test_stress_aabb_nan() {
    let mut spec = create_default_spec();
    spec.collision_volume.physical_occupancy = AABB::new([f32::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let res = spec.validate();
    assert!(res.is_err(), "NaN AABB must fail Gate 3");
    assert_eq!(res.unwrap_err().gate, Gate::Gate3);
}

#[test]
fn test_stress_planetary_values_extreme() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 50.0,
            ambition: -10.0,
            beauty: f32::NAN,
            community: f32::INFINITY,
            order: f32::NEG_INFINITY,
            knowledge: 2.0,
        },
        _marker: PhantomData::<Earth>,
    };

    let weights = cp.compute_weights();
    assert_eq!(weights.angelic_wing_binder_probability, 0.9);
    assert!(weights.white_gold_material_bias);
    assert_eq!(weights.heavy_weapon_mount_probability, 0.1);
    assert!(!weights.symmetric_joint_layout_enforced);
    assert!(weights.sensor_array_hardpoint_required);

    let spec = generate_spec(cp, FunctionalRole::Worker);
    let res = spec.validate();
    assert!(
        res.is_ok(),
        "Extreme planetary values that don't violate rules should pass, got {:?}",
        res
    );
}

#[test]
fn test_stress_unbounded_joint_rotation_limits() {
    let mut spec = create_default_spec();

    let invalid_limits = RotationLimits {
        min_yaw: 180.0,
        max_yaw: -180.0,
        min_pitch: 90.0,
        max_pitch: -90.0,
        min_roll: 180.0,
        max_roll: -180.0,
    };

    spec.joints[0].rotation_limits = Some(invalid_limits);

    let res = spec.validate();
    assert!(
        res.is_err(),
        "Invalid joint limits (min > max) must fail Gate 2"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joint_rotation_limits_nan() {
    let mut spec = create_default_spec();

    let nan_limits = RotationLimits {
        min_yaw: f32::NAN,
        max_yaw: f32::NAN,
        min_pitch: f32::NAN,
        max_pitch: f32::NAN,
        min_roll: f32::NAN,
        max_roll: f32::NAN,
    };

    spec.joints[0].rotation_limits = Some(nan_limits);

    let res = spec.validate();
    assert!(res.is_err(), "NaN joint limits must fail Gate 2");
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_touching_collision_volumes() {
    let mut spec = create_default_spec();

    spec.collision_volume.physical_occupancy = AABB::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    spec.collision_volume.interaction_zones[0] = AABB::new([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]);

    let res = spec.validate();
    assert!(res.is_err(), "Touching volumes must fail Gate 3");
    assert_eq!(res.unwrap_err().gate, Gate::Gate3);
}

#[test]
fn test_mars_mech_allowed_roles() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 0.5,
            ambition: 0.5,
            beauty: 0.5,
            community: 0.5,
            order: 0.5,
            knowledge: 0.5,
        },
        _marker: PhantomData::<Mars>,
    };

    let spec = generate_spec(cp.clone(), FunctionalRole::Warrior);
    assert!(spec.validate().is_ok());

    let spec = generate_spec(cp.clone(), FunctionalRole::Worker);
    assert!(spec.validate().is_ok());

    let spec = generate_spec(cp, FunctionalRole::Ark);
    assert!(spec.validate().is_ok());
}

#[test]
fn test_mars_mech_disallowed_roles() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 0.5,
            ambition: 0.5,
            beauty: 0.5,
            community: 0.5,
            order: 0.5,
            knowledge: 0.5,
        },
        _marker: PhantomData::<Mars>,
    };

    let spec = generate_spec(cp, FunctionalRole::Explorer);
    let res = spec.validate();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().gate, Gate::Gate4);
}

#[test]
fn test_venus_mech_allowed_mobility() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 0.5,
            ambition: 0.5,
            beauty: 0.5,
            community: 0.5,
            order: 0.5,
            knowledge: 0.5,
        },
        _marker: PhantomData::<Venus>,
    };

    let spec = generate_spec(cp.clone(), FunctionalRole::Explorer);
    assert!(spec.validate().is_ok());

    let spec = generate_spec(cp, FunctionalRole::Ark);
    assert!(spec.validate().is_ok());
}

#[test]
fn test_venus_mech_disallowed_mobility() {
    let cp = CulturalProfile {
        planetary_values: PlanetaryValues {
            faith: 0.5,
            ambition: 0.5,
            beauty: 0.5,
            community: 0.5,
            order: 0.5,
            knowledge: 0.5,
        },
        _marker: PhantomData::<Venus>,
    };

    let spec = generate_spec(cp, FunctionalRole::Worker);
    let res = spec.validate();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().gate, Gate::Gate4);
}

#[test]
fn test_builder_pattern_construction() {
    let spec = create_default_spec();

    let reconstructed = MechAssemblySpecBuilder::<Earth>::new()
        .frame(spec.frame.clone())
        .joints(spec.joints.clone())
        .power(spec.power.clone())
        .motion_profile(spec.motion_profile)
        .collision_volume(spec.collision_volume.clone())
        .material_spec(spec.material_spec.clone())
        .cultural_profile(spec.cultural_profile.clone())
        .functional_role(spec.functional_role)
        .equipment_mass(spec.equipment_mass)
        .build();

    assert_eq!(spec, reconstructed);
}

#[test]
fn test_stress_frame_load_capacity_nan() {
    let mut spec = create_default_spec();
    spec.frame.load_capacity = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN frame load capacity must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_load_capacity_infinite() {
    let mut spec = create_default_spec();
    spec.frame.load_capacity = f32::INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Infinite frame load capacity must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_load_capacity_negative() {
    let mut spec = create_default_spec();
    spec.frame.load_capacity = -100.0;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Negative frame load capacity must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_scale_nan() {
    let mut spec = create_default_spec();
    spec.frame.scale = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN frame scale must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_scale_infinite() {
    let mut spec = create_default_spec();
    spec.frame.scale = f32::INFINITY;
    let res = spec.validate();
    assert!(res.is_err(), "Infinite frame scale must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_scale_zero_or_negative() {
    let mut spec = create_default_spec();
    spec.frame.scale = 0.0;
    let res = spec.validate();
    assert!(res.is_err(), "Zero frame scale must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);

    let mut spec = create_default_spec();
    spec.frame.scale = -0.5;
    let res = spec.validate();
    assert!(res.is_err(), "Negative frame scale must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_size_nan() {
    let mut spec = create_default_spec();
    spec.frame.size[1] = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN frame size coordinate must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_size_infinite() {
    let mut spec = create_default_spec();
    spec.frame.size[2] = f32::INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Infinite frame size coordinate must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_frame_size_zero_or_negative() {
    let mut spec = create_default_spec();
    spec.frame.size[0] = 0.0;
    let res = spec.validate();
    assert!(res.is_err(), "Zero frame size coordinate must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);

    let mut spec = create_default_spec();
    spec.frame.size[0] = -1.0;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Negative frame size coordinate must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_energy_capacity_nan() {
    let mut spec = create_default_spec();
    spec.power.energy_capacity = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN power energy capacity must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_energy_capacity_infinite() {
    let mut spec = create_default_spec();
    spec.power.energy_capacity = f32::INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Infinite power energy capacity must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_energy_capacity_negative() {
    let mut spec = create_default_spec();
    spec.power.energy_capacity = -50.0;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Negative power energy capacity must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_output_nan() {
    let mut spec = create_default_spec();
    spec.power.output = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN power output must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_output_infinite() {
    let mut spec = create_default_spec();
    spec.power.output = f32::INFINITY;
    let res = spec.validate();
    assert!(res.is_err(), "Infinite power output must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_power_output_negative() {
    let mut spec = create_default_spec();
    spec.power.output = -10.0;
    let res = spec.validate();
    assert!(res.is_err(), "Negative power output must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_joints_extension_limits_nan() {
    let mut spec = create_default_spec();
    spec.joints[0].extension_limits[0] = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN joint extension limits must fail Gate 2");
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joints_extension_limits_infinite() {
    let mut spec = create_default_spec();
    spec.joints[0].extension_limits[1] = f32::INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Infinite joint extension limits must fail Gate 2"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joints_extension_limits_inverted() {
    let mut spec = create_default_spec();
    spec.joints[0].extension_limits = [5.0, 2.0];
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Inverted joint extension limits (min > max) must fail Gate 2"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joints_attachment_limits_nan() {
    let mut spec = create_default_spec();
    spec.joints[0].attachment_limits[1] = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN joint attachment limits must fail Gate 2");
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joints_attachment_limits_infinite() {
    let mut spec = create_default_spec();
    spec.joints[0].attachment_limits[0] = f32::NEG_INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Infinite joint attachment limits must fail Gate 2"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_joints_attachment_limits_inverted() {
    let mut spec = create_default_spec();
    spec.joints[0].attachment_limits = [10.0, 1.0];
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Inverted joint attachment limits (min > max) must fail Gate 2"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate2);
}

#[test]
fn test_stress_frame_center_of_mass_nan() {
    for i in 0..3 {
        let mut spec = create_default_spec();
        spec.frame.center_of_mass[i] = f32::NAN;
        let res = spec.validate();
        assert!(
            res.is_err(),
            "NaN coordinate in frame center of mass at index {} must fail Gate 1",
            i
        );
        assert_eq!(res.unwrap_err().gate, Gate::Gate1);
    }
}

#[test]
fn test_stress_frame_center_of_mass_infinite() {
    for i in 0..3 {
        let mut spec = create_default_spec();
        spec.frame.center_of_mass[i] = f32::INFINITY;
        let res = spec.validate();
        assert!(
            res.is_err(),
            "Infinite coordinate in frame center of mass at index {} must fail Gate 1",
            i
        );
        assert_eq!(res.unwrap_err().gate, Gate::Gate1);

        let mut spec = create_default_spec();
        spec.frame.center_of_mass[i] = f32::NEG_INFINITY;
        let res = spec.validate();
        assert!(
            res.is_err(),
            "Negative infinite coordinate in frame center of mass at index {} must fail Gate 1",
            i
        );
        assert_eq!(res.unwrap_err().gate, Gate::Gate1);
    }
}

#[test]
fn test_stress_material_wear_state_nan() {
    let mut spec = create_default_spec();
    spec.material_spec.wear_state = f32::NAN;
    let res = spec.validate();
    assert!(res.is_err(), "NaN wear state must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_material_wear_state_infinite() {
    let mut spec = create_default_spec();
    spec.material_spec.wear_state = f32::INFINITY;
    let res = spec.validate();
    assert!(res.is_err(), "Infinite wear state must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);

    let mut spec = create_default_spec();
    spec.material_spec.wear_state = f32::NEG_INFINITY;
    let res = spec.validate();
    assert!(
        res.is_err(),
        "Negative infinite wear state must fail Gate 1"
    );
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}

#[test]
fn test_stress_material_wear_state_negative() {
    let mut spec = create_default_spec();
    spec.material_spec.wear_state = -0.1;
    let res = spec.validate();
    assert!(res.is_err(), "Negative wear state must fail Gate 1");
    assert_eq!(res.unwrap_err().gate, Gate::Gate1);
}
