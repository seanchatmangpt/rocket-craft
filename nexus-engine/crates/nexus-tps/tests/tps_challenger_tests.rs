use nexus_tps::{
    assemble_mech, branchless_clamp01, generate_part, Axis, JidokaHalt, PartSlot, PartStateVector,
};

fn create_base_vector(slot: PartSlot) -> PartStateVector {
    PartStateVector {
        civilization_id: 0,
        frame_id: 10,
        armor_profile: 0.5,
        joint_profile: 0.5,
        mass_profile: 0.5,
        weapon_profile: 0.5,
        motion_profile: 50.0,
        part_slot: slot,
    }
}

/// Helper to build a valid set of vectors that normally passes all gates.
fn create_valid_mech_vectors() -> [PartStateVector; 8] {
    let mut vectors = [
        create_base_vector(PartSlot::Head),
        create_base_vector(PartSlot::Torso),
        create_base_vector(PartSlot::Waist),
        create_base_vector(PartSlot::ArmL),
        create_base_vector(PartSlot::ArmR),
        create_base_vector(PartSlot::LegL),
        create_base_vector(PartSlot::LegR),
        create_base_vector(PartSlot::Backpack),
    ];
    // Adjust values to pass all gates
    for state in &mut vectors {
        state.civilization_id = 1; // mass_mult = 0.8
        state.frame_id = 12;
        state.joint_profile = 0.2;
        state.armor_profile = 0.0;
        state.mass_profile = 0.0; // Minimal mass to avoid collision and mass overload
        state.motion_profile = 50.0;
        state.weapon_profile = 0.1;
    }
    vectors
}

// ============================================================================
// 1. Float Clamp Stability and NaN/INFINITY handling
// ============================================================================

#[test]
fn test_branchless_clamp01_nan_inf_propagation() {
    // Under float math, NAN / INFINITY in branchless_clamp01 propagate NAN.
    // Let's assert this behavior to ensure we know exactly what is returned.
    assert!(branchless_clamp01(f32::NAN).is_nan());
    assert!(branchless_clamp01(f32::INFINITY).is_nan());
    assert!(branchless_clamp01(f32::NEG_INFINITY).is_nan());
}

#[test]
fn test_branchless_clamp01_subnormals_and_limits() {
    // Test extremely small positive/negative floats (subnormals)
    let tiny_pos = 1.4e-45_f32;
    let tiny_neg = -1.4e-45_f32;
    assert_eq!(branchless_clamp01(tiny_pos), tiny_pos);
    assert_eq!(branchless_clamp01(tiny_neg), 0.0);

    // Test massive positive/negative floats
    let huge_pos = 3.4e38_f32;
    let huge_neg = -3.4e38_f32;
    assert_eq!(branchless_clamp01(huge_pos), 1.0);
    assert_eq!(branchless_clamp01(huge_neg), 0.0);

    // Test exactly on limits
    assert_eq!(branchless_clamp01(0.0), 0.0);
    assert_eq!(branchless_clamp01(-0.0), 0.0);
    assert_eq!(branchless_clamp01(1.0), 1.0);
}

#[test]
fn test_generate_part_rejects_nan_inf() {
    let mut state = create_base_vector(PartSlot::Head);

    // Test NaN rejection in each profile field
    let profiles = [
        |s: &mut PartStateVector, val| s.armor_profile = val,
        |s: &mut PartStateVector, val| s.joint_profile = val,
        |s: &mut PartStateVector, val| s.mass_profile = val,
        |s: &mut PartStateVector, val| s.weapon_profile = val,
    ];

    for setter in &profiles {
        setter(&mut state, f32::NAN);
        let res = generate_part(&state);
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            JidokaHalt::MotionBoundsViolated { .. }
        ));

        setter(&mut state, f32::INFINITY);
        let res = generate_part(&state);
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            JidokaHalt::MotionBoundsViolated { .. }
        ));

        setter(&mut state, f32::NEG_INFINITY);
        let res = generate_part(&state);
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            JidokaHalt::MotionBoundsViolated { .. }
        ));

        // Restore to valid value
        setter(&mut state, 0.5);
    }

    // Motion profile has a different limit (100.0)
    state.motion_profile = f32::NAN;
    let res = generate_part(&state);
    assert!(res.is_err());
    assert!(matches!(
        res.err().unwrap(),
        JidokaHalt::MotionBoundsViolated { axis: Axis::Z, .. }
    ));

    state.motion_profile = f32::INFINITY;
    let res = generate_part(&state);
    assert!(res.is_err());
    assert!(matches!(
        res.err().unwrap(),
        JidokaHalt::MotionBoundsViolated { axis: Axis::Z, .. }
    ));

    state.motion_profile = f32::NEG_INFINITY;
    let res = generate_part(&state);
    assert!(res.is_err());
    assert!(matches!(
        res.err().unwrap(),
        JidokaHalt::MotionBoundsViolated { axis: Axis::Z, .. }
    ));
}

// ============================================================================
// 2. Boundary conditions on discrete fields
// ============================================================================

#[test]
fn test_civilization_id_wrapping() {
    // civilization_id uses bitwise `& 3`, mapping any u16 value to the index range [0, 3].
    // Let's verify that arbitrary large/edge civilization IDs do not cause index out-of-bounds or panic.
    let test_ids = [0u16, 3u16, 4u16, 255u16, 256u16, 65535u16];
    for &civ_id in &test_ids {
        let mut state = create_base_vector(PartSlot::Head);
        state.civilization_id = civ_id;
        let part_res = generate_part(&state);
        assert!(part_res.is_ok());
    }
}

#[test]
fn test_frame_id_bounds() {
    // frame_id is u8 and can be [0, 255].
    // Verify that both limits work perfectly.
    let test_frame_ids = [0u8, 255u8];
    for &frame_id in &test_frame_ids {
        let mut state = create_base_vector(PartSlot::Head);
        state.frame_id = frame_id;
        let part_res = generate_part(&state);
        assert!(part_res.is_ok());
    }
}

// ============================================================================
// 3. Collision and Load Capacity Logic
// ============================================================================

#[test]
fn test_collision_gate_exact_math() {
    // Test that the collision distance math is accurate.
    // Torso is at (0, 0, 0) and Head is at (0, 0, 2.0). Distance = 2.0.
    // If the combined radius exceeds 2.0, assembly must fail with JidokaHalt::CollisionVolumeIntersects.
    let mut vectors = create_valid_mech_vectors();

    // Verify successful base assembly first
    assert!(assemble_mech(&vectors).is_ok());

    // Make Torso and Head extremely large by setting max armor, mass, motion profiles
    // and civilization 3 (mass_mult = 1.5, armor_mult = 1.5, motion_mult = 0.5)
    for state in &mut vectors {
        state.civilization_id = 3;
    }
    vectors[PartSlot::Torso as usize].mass_profile = 1.0;
    vectors[PartSlot::Torso as usize].armor_profile = 1.0;
    vectors[PartSlot::Torso as usize].motion_profile = 100.0;

    vectors[PartSlot::Head as usize].mass_profile = 1.0;
    vectors[PartSlot::Head as usize].armor_profile = 1.0;
    vectors[PartSlot::Head as usize].motion_profile = 100.0;

    let res = assemble_mech(&vectors);
    assert!(res.is_err());
    let err = res.err().unwrap();

    // With these values, Torso and Waist will intersect first due to a distance of 1.0.
    // Let's assert that we do indeed halt on collision.
    assert!(
        matches!(err, JidokaHalt::CollisionVolumeIntersects { .. }),
        "Expected CollisionVolumeIntersects, got: {:?}",
        err
    );
}

#[test]
fn test_load_capacity_exact_math() {
    let mut vectors = create_valid_mech_vectors();

    // Under civ 1, mass_mult = 0.8.
    // LegL slot: base_dim_z = 2.4, dim_z_range = 0.8.
    // With mass_profile = 0.0, dim_z = 2.4 * 0.8 = 1.92.
    // geom_z_quant = 192.
    // leg_l_val = 192.
    // leg_r_val = 192.
    // load_capacity = (192 + 192) * 7 / 2 = 384 * 7 / 2 = 1344.
    // Let's verify that if total_mass exceeds 1344, assembly fails.

    // Let's set some parts' mass_profile to increase mass.
    // Head: base_mass = 5.0. mass = 5.0 * 0.8 = 4.0. mass_balance = 40.
    // Torso: base_mass = 40.0. mass_balance = 320.
    // Waist: base_mass = 20.0. mass_balance = 160.
    // ArmL: base_mass = 12.0. mass_balance = 96.
    // ArmR: base_mass = 12.0. mass_balance = 96.
    // LegL: base_mass = 25.0. mass_balance = 200.
    // LegR: base_mass = 25.0. mass_balance = 200.
    // Backpack: base_mass = 15.0. mass_balance = 120.
    // Total base mass_balance with 0.0 mass_profile = 40 + 320 + 160 + 96 + 96 + 200 + 200 + 120 = 1232.
    // This is less than 1344. So it succeeds.

    // If we increase mass_profile of Head to 1.0:
    // Head: base_mass + mass_range = 7.0. mass = 7.0 * 0.8 = 5.6. mass_balance = 56.
    // Delta mass = +16.
    // If we increase mass_profile of Backpack to 1.0:
    // Backpack: base_mass + mass_range = 25.0. mass = 25.0 * 0.8 = 20.0. mass_balance = 200.
    // Delta mass = +80.
    // Total mass becomes 1232 + 16 + 80 = 1328. Still <= 1344.

    // Let's increase Backpack mass_profile further or use higher mass_profile on Arms.
    // If ArmL mass_profile = 1.0:
    // ArmL: base_mass + mass_range = 18.0. mass = 18.0 * 0.8 = 14.4. mass_balance = 144.
    // Delta mass = +48.
    // Total mass becomes 1328 + 48 = 1376. This is > 1344.
    // So if Head mass_profile = 1.0, Backpack mass_profile = 1.0, and ArmL mass_profile = 1.0,
    // total mass exceeds load capacity.

    vectors[PartSlot::Head as usize].mass_profile = 1.0;
    vectors[PartSlot::Backpack as usize].mass_profile = 1.0;
    vectors[PartSlot::ArmL as usize].mass_profile = 1.0;

    let res = assemble_mech(&vectors);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(
        matches!(err, JidokaHalt::MassExceedsFrameCapacity { mass, capacity } if mass == 1384.0 && capacity == 1344.0),
        "Expected MassExceedsFrameCapacity with mass=1384, capacity=1344, got: {:?}",
        err
    );
}
