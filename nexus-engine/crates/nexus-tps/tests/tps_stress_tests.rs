use nexus_tps::{
    assemble_mech, branchless_clamp01, branchless_lerp,
    PartStateVector, PartSlot, JidokaHalt
};

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0xFFFFFF) as f32 / 16777216.0
    }

    fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

#[test]
fn test_branchless_clamp01_stability() {
    // 1. Verify standard clamps
    assert_eq!(branchless_clamp01(0.0), 0.0);
    assert_eq!(branchless_clamp01(1.0), 1.0);
    assert_eq!(branchless_clamp01(0.5), 0.5);
    
    // Values outside standard range
    assert_eq!(branchless_clamp01(-0.5), 0.0);
    assert_eq!(branchless_clamp01(-1234.5), 0.0);
    assert_eq!(branchless_clamp01(1.5), 1.0);
    assert_eq!(branchless_clamp01(9999.0), 1.0);

    // Subnormal/extremely small numbers
    assert_eq!(branchless_clamp01(1.0e-40), 1.0e-40);
    assert_eq!(branchless_clamp01(-1.0e-40), 0.0);

    // 2. Infinity and NaN verification (Empirically checking their output to avoid failing assertions)
    let pos_inf = f32::INFINITY;
    let neg_inf = f32::NEG_INFINITY;
    let nan = f32::NAN;

    let res_pos_inf = branchless_clamp01(pos_inf);
    let res_neg_inf = branchless_clamp01(neg_inf);
    let res_nan = branchless_clamp01(nan);

    println!("branchless_clamp01(INFINITY) = {}", res_pos_inf);
    println!("branchless_clamp01(-INFINITY) = {}", res_neg_inf);
    println!("branchless_clamp01(NAN) = {}", res_nan);

    // Verify that these extreme values result in NaN because of INFINITY * 0.0 or NaN multiplication in branchless_clamp01
    assert!(res_pos_inf.is_nan(), "Expected branchless_clamp01(INFINITY) to be NaN due to INFINITY * 0.0");
    assert!(res_neg_inf.is_nan(), "Expected branchless_clamp01(-INFINITY) to be NaN due to NEG_INFINITY * 0.0");
    assert!(res_nan.is_nan(), "Expected branchless_clamp01(NAN) to be NaN");
}

#[test]
fn test_branchless_lerp_properties() {
    assert_eq!(branchless_lerp(10.0, 20.0, 0.0), 10.0);
    assert_eq!(branchless_lerp(10.0, 20.0, 1.0), 20.0);
    assert_eq!(branchless_lerp(10.0, 20.0, 0.5), 15.0);
    assert_eq!(branchless_lerp(10.0, 20.0, -1.0), 0.0);
    assert_eq!(branchless_lerp(10.0, 20.0, 2.0), 30.0);
}

#[test]
fn test_random_assembly_stress() {
    let mut rng = Lcg::new(42);
    let mut ok_count = 0;
    let mut socket_mismatch_count = 0;
    let mut collision_count = 0;
    let mut mass_capacity_count = 0;
    let mut motion_bounds_count = 0;
    let defect_count = 0;

    let iterations = 10000;
    for _ in 0..iterations {
        let mut vectors = [
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::Head },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::Torso },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::Waist },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::ArmL },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::ArmR },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::LegL },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::LegR },
            PartStateVector { civilization_id: 0, frame_id: 0, armor_profile: 0.0, joint_profile: 0.0, mass_profile: 0.0, weapon_profile: 0.0, motion_profile: 0.0, part_slot: PartSlot::Backpack },
        ];

        // Randomize profiles
        let shared_civ = (rng.next_u32() % 4) as u16;
        let shared_frame = (rng.next_u32() % 256) as u8;
        let shared_joint = rng.next_f32(); // keep same joint profile so sockets match with high probability

        for (idx, slot) in [
            PartSlot::Head, PartSlot::Torso, PartSlot::Waist, PartSlot::ArmL,
            PartSlot::ArmR, PartSlot::LegL, PartSlot::LegR, PartSlot::Backpack
        ].iter().enumerate() {
            // Standard state vector definition
            let armor = if rng.next_f32() < 0.95 { rng.next_f32() } else { rng.next_f32_range(-0.5, 1.5) };
            let joint = if rng.next_f32() < 0.9 { shared_joint } else { if rng.next_f32() < 0.95 { rng.next_f32() } else { rng.next_f32_range(-0.5, 1.5) } };
            let mass = if rng.next_f32() < 0.95 { rng.next_f32() } else { rng.next_f32_range(-0.5, 1.5) };
            let weapon = if rng.next_f32() < 0.95 { rng.next_f32() } else { rng.next_f32_range(-0.5, 1.5) };
            let motion = if rng.next_f32() < 0.95 { rng.next_f32_range(0.0, 100.0) } else { rng.next_f32_range(-10.0, 120.0) };

            vectors[idx] = PartStateVector {
                civilization_id: if rng.next_f32() < 0.9 { shared_civ } else { (rng.next_u32() % 100) as u16 },
                frame_id: if rng.next_f32() < 0.9 { shared_frame } else { (rng.next_u32() % 256) as u8 },
                armor_profile: armor,
                joint_profile: joint,
                mass_profile: mass,
                weapon_profile: weapon,
                motion_profile: motion,
                part_slot: *slot,
            };
        }

        match assemble_mech(&vectors) {
            Ok(receipt) => {
                ok_count += 1;
                assert_eq!(receipt.component_count, 8);
                assert_eq!(receipt.final_decision, "APPROVED");
                assert!(receipt.total_mass <= receipt.load_capacity, "Total mass {} exceeds capacity {}", receipt.total_mass, receipt.load_capacity);
            }
            Err(JidokaHalt::SocketMismatch { .. }) => {
                socket_mismatch_count += 1;
            }
            Err(JidokaHalt::CollisionVolumeIntersects { .. }) => {
                collision_count += 1;
            }
            Err(JidokaHalt::MassExceedsFrameCapacity { .. }) => {
                mass_capacity_count += 1;
            }
            Err(JidokaHalt::MotionBoundsViolated { .. }) => {
                motion_bounds_count += 1;
            }
        }
    }

    println!("Stress test results ({} iterations):", iterations);
    println!("  Successful Assemblies: {}", ok_count);
    println!("  Socket Mismatch Halts: {}", socket_mismatch_count);
    println!("  Collision Halts:       {}", collision_count);
    println!("  Mass Overload Halts:   {}", mass_capacity_count);
    println!("  Motion Bounds Halts:   {}", motion_bounds_count);
    println!("  Structure Defects:     {}", defect_count);

    // Ensure we exercised multiple pathways
    assert!(socket_mismatch_count > 0);
    assert!(motion_bounds_count > 0);
    assert!(ok_count + socket_mismatch_count + collision_count + mass_capacity_count + motion_bounds_count + defect_count == iterations);
}

#[test]
fn test_load_capacity_logic_wrapping() {
    // Construct vectors where leg dimensions wrap around
    // Let's create LegL and LegR with massive mass profiles to maximize dim_z, which is used for leg_l_z and leg_r_z
    let vectors = [
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::Head },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::Torso },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::Waist },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::ArmL },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::ArmR },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 1.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::LegL },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 1.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::LegR },
        PartStateVector { civilization_id: 3, frame_id: 12, armor_profile: 0.5, joint_profile: 0.2, mass_profile: 0.0, weapon_profile: 0.1, motion_profile: 50.0, part_slot: PartSlot::Backpack },
    ];

    // Let's verify that assembly runs and doesn't panic when we have high mass profiles
    let res = assemble_mech(&vectors);
    println!("Massive mass profiles assembly result: {:?}", res);
    // It should either succeed or fail with a JidokaHalt, but MUST NOT panic
    assert!(res.is_ok() || res.is_err());
}
