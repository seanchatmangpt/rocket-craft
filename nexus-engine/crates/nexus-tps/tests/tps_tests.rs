use nexus_tps::{
    generate_part, assemble_mech, PartStateVector, PartSlot, JidokaHalt, TpsReceipt, Axis, SocketType
};

fn create_valid_vector(slot: PartSlot) -> PartStateVector {
    PartStateVector {
        civilization_id: 1,
        frame_id: 12,
        armor_profile: 0.5,
        joint_profile: 0.2,
        mass_profile: 0.4,
        weapon_profile: 0.1,
        motion_profile: 50.0,
        part_slot: slot,
    }
}

#[test]
fn test_generate_part_success() {
    let state = create_valid_vector(PartSlot::Head);
    let part_res = generate_part(&state);
    assert!(part_res.is_ok());
    let part = part_res.unwrap();

    assert!(part.geometry > 0);
    assert!(part.socket_fit > 0);
    assert!(part.motion_clearance > 0);
    assert!(part.collision_volume > 0);
    assert!(part.mass_balance > 0);
    assert!(part.physics_role > 0);
    assert!(part.assembly_compatibility > 0);
}

#[test]
fn test_generate_part_motion_violation() {
    let mut state = create_valid_vector(PartSlot::Torso);
    state.motion_profile = 150.0;
    let part_res = generate_part(&state);
    assert!(matches!(part_res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::Z, limit: 100.0, actual }) if actual == 150.0));
}

#[test]
fn test_receipt_generation() {
    let state = create_valid_vector(PartSlot::Head);
    let part = generate_part(&state).unwrap();
    let receipt = TpsReceipt::generate(&state, &part, vec![]);

    assert!(!receipt.state_vector_hash.is_empty());
    assert_eq!(receipt.part_slot, PartSlot::Head);
    assert_eq!(receipt.mass, part.mass_balance);
    assert_eq!(receipt.gates_passed_mask, 0b111);
    assert!(receipt.jidoka_halts.is_empty());
}

#[test]
fn test_assembly_success() {
    let mut vectors = [
        create_valid_vector(PartSlot::Head),
        create_valid_vector(PartSlot::Torso),
        create_valid_vector(PartSlot::Waist),
        create_valid_vector(PartSlot::ArmL),
        create_valid_vector(PartSlot::ArmR),
        create_valid_vector(PartSlot::LegL),
        create_valid_vector(PartSlot::LegR),
        create_valid_vector(PartSlot::Backpack),
    ];

    vectors[0].part_slot = PartSlot::Head;
    vectors[1].part_slot = PartSlot::Torso;
    vectors[2].part_slot = PartSlot::Waist;
    vectors[3].part_slot = PartSlot::ArmL;
    vectors[4].part_slot = PartSlot::ArmR;
    vectors[5].part_slot = PartSlot::LegL;
    vectors[6].part_slot = PartSlot::LegR;
    vectors[7].part_slot = PartSlot::Backpack;

    // Mutate to ensure safety and fit within capacity limit
    for state in &mut vectors {
        state.civilization_id = 1; // mass_mult = 0.8
        state.frame_id = 12;
        state.joint_profile = 0.2;
        state.armor_profile = 0.0;
        state.mass_profile = 0.0; // minimal mass balance
        state.motion_profile = 50.0;
        state.weapon_profile = 0.1;
    }

    let res = assemble_mech(&vectors);
    assert!(res.is_ok(), "Assembly failed: {:?}", res);
    let receipt = res.unwrap();
    assert_eq!(receipt.final_decision, "APPROVED");
    assert_eq!(receipt.component_count, 8);
    assert!(!receipt.lineage_hash.is_empty());
}

#[test]
fn test_assembly_socket_mismatch() {
    let mut vectors = [
        create_valid_vector(PartSlot::Head),
        create_valid_vector(PartSlot::Torso),
        create_valid_vector(PartSlot::Waist),
        create_valid_vector(PartSlot::ArmL),
        create_valid_vector(PartSlot::ArmR),
        create_valid_vector(PartSlot::LegL),
        create_valid_vector(PartSlot::LegR),
        create_valid_vector(PartSlot::Backpack),
    ];

    vectors[0].part_slot = PartSlot::Head;
    vectors[1].part_slot = PartSlot::Torso;
    vectors[2].part_slot = PartSlot::Waist;
    vectors[3].part_slot = PartSlot::ArmL;
    vectors[4].part_slot = PartSlot::ArmR;
    vectors[5].part_slot = PartSlot::LegL;
    vectors[6].part_slot = PartSlot::LegR;
    vectors[7].part_slot = PartSlot::Backpack;

    // Mutate Torso joint_profile/frame_id to force socket mismatch
    vectors[1].joint_profile = 0.9;
    vectors[1].frame_id = 100;

    let res = assemble_mech(&vectors);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(matches!(
        err,
        JidokaHalt::SocketMismatch {
            expected: SocketType::Torso,
            got: SocketType::Head
        }
    ));
}

#[test]
fn test_assembly_collision_failure() {
    let mut vectors = [
        create_valid_vector(PartSlot::Head),
        create_valid_vector(PartSlot::Torso),
        create_valid_vector(PartSlot::Waist),
        create_valid_vector(PartSlot::ArmL),
        create_valid_vector(PartSlot::ArmR),
        create_valid_vector(PartSlot::LegL),
        create_valid_vector(PartSlot::LegR),
        create_valid_vector(PartSlot::Backpack),
    ];

    vectors[0].part_slot = PartSlot::Head;
    vectors[1].part_slot = PartSlot::Torso;
    vectors[2].part_slot = PartSlot::Waist;
    vectors[3].part_slot = PartSlot::ArmL;
    vectors[4].part_slot = PartSlot::ArmR;
    vectors[5].part_slot = PartSlot::LegL;
    vectors[6].part_slot = PartSlot::LegR;
    vectors[7].part_slot = PartSlot::Backpack;

    // Mutate Torso and Head mass_profile / motion_profile to force huge collision volume/clearance
    vectors[1].mass_profile = 1.0;
    vectors[0].mass_profile = 1.0;
    vectors[1].motion_profile = 99.0;
    vectors[0].motion_profile = 99.0;
    vectors[1].armor_profile = 1.0;
    vectors[0].armor_profile = 1.0;

    vectors[0].civilization_id = 3;
    vectors[1].civilization_id = 3;

    let res = assemble_mech(&vectors);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(matches!(
        err,
        JidokaHalt::CollisionVolumeIntersects {
            part_a: PartSlot::Torso,
            part_b: PartSlot::Waist
        }
    ), "Expected CollisionVolumeIntersects, got {:?}", err);
}

#[test]
fn test_assembly_mass_overload() {
    let mut vectors = [
        create_valid_vector(PartSlot::Head),
        create_valid_vector(PartSlot::Torso),
        create_valid_vector(PartSlot::Waist),
        create_valid_vector(PartSlot::ArmL),
        create_valid_vector(PartSlot::ArmR),
        create_valid_vector(PartSlot::LegL),
        create_valid_vector(PartSlot::LegR),
        create_valid_vector(PartSlot::Backpack),
    ];

    vectors[0].part_slot = PartSlot::Head;
    vectors[1].part_slot = PartSlot::Torso;
    vectors[2].part_slot = PartSlot::Waist;
    vectors[3].part_slot = PartSlot::ArmL;
    vectors[4].part_slot = PartSlot::ArmR;
    vectors[5].part_slot = PartSlot::LegL;
    vectors[6].part_slot = PartSlot::LegR;
    vectors[7].part_slot = PartSlot::Backpack;

    // Standard socket parameters so they match
    for state in &mut vectors {
        state.frame_id = 12;
        state.joint_profile = 0.2;
    }

    // Force high mass on parts that do not collide (ArmL, ArmR, Backpack)
    vectors[0].civilization_id = 3; // mass_mult = 1.5
    vectors[0].mass_profile = 0.0;
    vectors[1].civilization_id = 3;
    vectors[1].mass_profile = 0.0;
    vectors[2].civilization_id = 3;
    vectors[2].mass_profile = 0.0;
    vectors[3].civilization_id = 3;
    vectors[3].mass_profile = 1.0; // heavy ArmL
    vectors[4].civilization_id = 3;
    vectors[4].mass_profile = 1.0; // heavy ArmR
    vectors[7].civilization_id = 3;
    vectors[7].mass_profile = 1.0; // heavy Backpack

    // Keep LegL/LegR geometry low so capacity is low
    vectors[5].civilization_id = 3;
    vectors[5].mass_profile = 0.0;
    vectors[6].civilization_id = 3;
    vectors[6].mass_profile = 0.0;

    let res = assemble_mech(&vectors);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(matches!(
        err,
        JidokaHalt::MassExceedsFrameCapacity { mass, capacity } if mass > capacity
    ), "Expected MassExceedsFrameCapacity, got {:?}", err);
}

#[test]
fn test_tps_receipt_determinism() {
    let state = create_valid_vector(PartSlot::Head);
    let part = generate_part(&state).unwrap();
    let receipt1 = TpsReceipt::generate(&state, &part, vec![]);
    std::thread::sleep(std::time::Duration::from_millis(5));
    let receipt2 = TpsReceipt::generate(&state, &part, vec![]);
    assert_eq!(receipt1, receipt2, "TpsReceipt must be deterministic");
}

#[test]
fn test_mech_tps_receipt_determinism() {
    let mut vectors = [
        create_valid_vector(PartSlot::Head),
        create_valid_vector(PartSlot::Torso),
        create_valid_vector(PartSlot::Waist),
        create_valid_vector(PartSlot::ArmL),
        create_valid_vector(PartSlot::ArmR),
        create_valid_vector(PartSlot::LegL),
        create_valid_vector(PartSlot::LegR),
        create_valid_vector(PartSlot::Backpack),
    ];
    for state in &mut vectors {
        state.civilization_id = 1;
        state.frame_id = 12;
        state.joint_profile = 0.2;
        state.armor_profile = 0.0;
        state.mass_profile = 0.0;
        state.motion_profile = 50.0;
        state.weapon_profile = 0.1;
    }

    let receipt1 = assemble_mech(&vectors).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let receipt2 = assemble_mech(&vectors).unwrap();

    // Lineage and key attributes are deterministic
    assert_eq!(receipt1.lineage_hash, receipt2.lineage_hash);
    assert_eq!(receipt1.parts, receipt2.parts);
    assert_eq!(receipt1.passed_gates, receipt2.passed_gates);
    assert_eq!(receipt1.final_decision, receipt2.final_decision);
    assert_eq!(receipt1.total_mass, receipt2.total_mass);
    assert_eq!(receipt1.load_capacity, receipt2.load_capacity);
    assert_eq!(receipt1.component_count, receipt2.component_count);

    // However, timestamp and receipt_hash are non-deterministic due to chrono::Utc::now() in assemble_mech
    assert_ne!(receipt1.timestamp, receipt2.timestamp);
    assert_ne!(receipt1.receipt_hash, receipt2.receipt_hash);
    assert_ne!(receipt1, receipt2);
}

#[test]
fn test_generate_part_hardened_validation_armor() {
    let mut state = create_valid_vector(PartSlot::Head);
    state.armor_profile = -0.5;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::X, limit: 0.0, actual }) if actual == -0.5));

    state.armor_profile = 1.5;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::X, limit: 1.0, actual }) if actual == 1.5));

    state.armor_profile = f32::NAN;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::X, limit: 1.0, actual }) if actual.is_nan()));

    state.armor_profile = f32::INFINITY;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::X, limit: 1.0, actual }) if actual == f32::INFINITY));
}

#[test]
fn test_generate_part_hardened_validation_joint() {
    let mut state = create_valid_vector(PartSlot::Head);
    state.joint_profile = -0.1;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::Y, limit: 0.0, actual }) if actual == -0.1));
}

#[test]
fn test_generate_part_hardened_validation_mass() {
    let mut state = create_valid_vector(PartSlot::Head);
    state.mass_profile = 1.01;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::Z, limit: 1.0, actual }) if actual == 1.01));
}

#[test]
fn test_generate_part_hardened_validation_weapon() {
    let mut state = create_valid_vector(PartSlot::Head);
    state.weapon_profile = -0.001;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::X, limit: 0.0, actual }) if actual == -0.001));
}

#[test]
fn test_generate_part_hardened_validation_motion() {
    let mut state = create_valid_vector(PartSlot::Head);
    state.motion_profile = 100.1;
    let res = generate_part(&state);
    assert!(matches!(res, Err(JidokaHalt::MotionBoundsViolated { axis: Axis::Z, limit: 100.0, actual }) if actual == 100.1));
}


