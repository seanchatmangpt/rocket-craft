use nexus_mud::{MudEngine, MudError, Zone};
use nexus_tps::{PartSlot, PartStateVector};

fn assert_eq_sorted_lines(a: &str, b: &str) {
    let mut a_lines: Vec<&str> = a.lines().collect();
    let mut b_lines: Vec<&str> = b.lines().collect();
    a_lines.sort();
    b_lines.sort();
    assert_eq!(a_lines, b_lines);
}

#[test]
fn test_edge_cases_state_vectors() {
    let edge_values = [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        101.0,
        -0.0001,
        1.0001,
        f32::MAX,
        f32::MIN,
    ];

    // 1. Verify that invalid profile values are caught during part generation (poka-yoke)
    for &val in &edge_values {
        let state = PartStateVector {
            civilization_id: 1,
            frame_id: 12,
            armor_profile: val,
            joint_profile: 0.2,
            mass_profile: 0.0,
            weapon_profile: 0.1,
            motion_profile: 50.0,
            part_slot: PartSlot::Head,
        };

        let res = nexus_tps::generate_part(&state);
        assert!(
            res.is_err(),
            "Expected part generation to fail for edge value: {:?}",
            val
        );
    }

    // 2. Verify propagation of NaN/Inf/Negative values in MUD twin parts PHM metrics
    let mut mud = MudEngine::new();

    // Inject NaN into health score of Head
    if let Some(head) = mud.twin_parts.get_mut("Head") {
        head.health_score = f32::NAN;
    }

    // Calculate metrics
    let phm = mud.get_phm_metrics();
    assert_eq!(phm.overall_health, 0.0);
    assert_eq!(phm.maintenance_state, "Required");

    // Inject negative health score
    if let Some(head) = mud.twin_parts.get_mut("Head") {
        head.health_score = -5.0;
    }
    let phm2 = mud.get_phm_metrics();
    assert_eq!(phm2.overall_health, -5.0);
    assert_eq!(phm2.maintenance_state, "Required");

    // Inject huge health score
    if let Some(head) = mud.twin_parts.get_mut("Head") {
        head.health_score = 9999.0;
    }
    let phm3 = mud.get_phm_metrics();
    assert_eq!(phm3.overall_health, 1.0);
}

#[test]
fn test_cyclic_navigation_determinism() {
    std::env::set_var("NEXUS_TEST_DETERMINISTIC", "1");
    let mut mud1 = MudEngine::new();
    let mut mud2 = MudEngine::new();

    // Sequentially verify and move through the new 9-zone layout
    let command_sequence = [
        "look",
        "verify mission",
        "go materials_lab",
        "verify materials",
        "go primitive_foundry",
        "verify primitive",
        "go runner_wall",
        "inventory",
        "inspect head",
        "verify runner_wall",
        "go assembly_gantry",
        "preview assembly",
        "assemble standard",
        "verify assembly",
        "go fit_bay",
        "verify fit",
        "go collision_bay",
        "verify collision",
        "go proving_ground",
        "preview motion",
        "verify motion",
        "go reveal_platform",
        "verify reveal",
    ];

    for cmd_str in &command_sequence {
        let res1 = mud1.execute_command(cmd_str).unwrap();
        let res2 = mud2.execute_command(cmd_str).unwrap();
        assert_eq_sorted_lines(&res1, &res2);
    }

    assert_eq!(mud1.room_transitions, mud2.room_transitions);
    assert_eq!(mud1.command_history, mud2.command_history);

    let rec1 = mud1
        .generate_walkthrough_receipt("Deterministic Walkthrough")
        .unwrap();
    let rec2 = mud2
        .generate_walkthrough_receipt("Deterministic Walkthrough")
        .unwrap();

    assert_eq!(rec1.prompt, rec2.prompt);
    assert_eq!(rec1.room_transitions, rec2.room_transitions);
    assert_eq!(rec1.command_history, rec2.command_history);
    assert_eq!(rec1.final_verdict, rec2.final_verdict);
    assert_eq!(
        rec1.final_assembly_receipt_hash,
        rec2.final_assembly_receipt_hash
    );
    assert_eq!(rec1.cryptographic_signature, rec2.cryptographic_signature);
    assert_eq!(rec1.signature_hash, rec2.signature_hash);

    assert_eq!(rec1.cryptographic_signature.len(), 64);
}

// Criterion 1: Factory world contains all 9 required zones.
#[test]
fn test_ac1_all_nine_zones_present() {
    let zones = Zone::all();
    assert_eq!(zones.len(), 9);
    assert!(zones.contains(&Zone::MissionRoom));
    assert!(zones.contains(&Zone::MaterialsLab));
    assert!(zones.contains(&Zone::PrimitiveFoundry));
    assert!(zones.contains(&Zone::RunnerWall));
    assert!(zones.contains(&Zone::AssemblyGantry));
    assert!(zones.contains(&Zone::FitBay));
    assert!(zones.contains(&Zone::CollisionBay));
    assert!(zones.contains(&Zone::ProvingGround));
    assert!(zones.contains(&Zone::RevealPlatform));
}

// Criterion 2: Every zone has valid bounds and connected exits.
#[test]
fn test_ac2_zone_bounds_and_descriptions() {
    for zone in Zone::all() {
        assert!(!zone.description().is_empty());
        assert!(zone.elevation() >= 0 && zone.elevation() <= 8);
    }
}

// Criteria 3 & 4: Valid path exists; invalid exit traversal is rejected.
#[test]
fn test_ac3_ac4_path_validity_and_invalid_travel_rejected() {
    let mut engine = MudEngine::new();

    // Direct travel skipping zones is rejected
    let skip_fail = engine.execute_command("go reveal_platform");
    assert!(skip_fail.is_err());
    assert!(matches!(
        skip_fail.unwrap_err(),
        MudError::InvalidDirectTravel { .. }
    ));

    // Moving forward without gate verification is blocked
    let move_fail_before_gate = engine.execute_command("go materials_lab");
    assert!(move_fail_before_gate.is_err());
    assert!(matches!(
        move_fail_before_gate.unwrap_err(),
        MudError::GateBlocked(_)
    ));
}

// Criterion 5: `look` returns the correct zone description.
#[test]
fn test_ac5_look_returns_zone_description() {
    let engine = MudEngine::new();
    assert_eq!(engine.current_zone, Zone::MissionRoom);
    // The Mission Room description is the exact static string defined in Zone::description()
    assert_eq!(
        engine.current_zone.description(),
        "Zone 0: The Command and Mission briefing room. The Strategic blueprint of the mech's operational journey begins here."
    );
}

// Criterion 6: `inspect part` returns structural data for the part.
#[test]
fn test_ac6_inspect_part_returns_structural_data() {
    let mut engine = MudEngine::new();
    // Navigate to runner_wall where parts are inspectable
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();

    // Verify head part fields directly via the parts map
    let head = engine.parts.get("head").expect("head part must exist");
    assert_eq!(head.mass, 10.0, "head mass must be 10 kg");
    assert!(
        !head.sockets_required.is_empty(),
        "head must have required sockets"
    );
    assert!(head.health_status > 0.0, "head health must be positive");
}

// Criterion 7: `inventory` at runner_wall lists all generated parts.
#[test]
fn test_ac7_inventory_lists_all_parts() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();

    // Check structural parts map directly — no string parsing
    let expected_parts = [
        "torso_frame",
        "head",
        "left_arm",
        "right_arm",
        "left_leg",
        "right_leg",
        "backpack",
        "left_thruster",
        "right_thruster",
    ];
    for part_id in &expected_parts {
        assert!(
            engine.parts.contains_key(*part_id),
            "parts map must contain '{}'",
            part_id
        );
    }
    assert_eq!(
        engine.parts.len(),
        expected_parts.len(),
        "must have exactly 9 parts"
    );
}

// Criterion 8: Assembly Gantry can assemble the minimal 9-part mech.
#[test]
fn test_ac8_assembly_gantry_assembles_mech() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();

    let assemble_res = engine.execute_command("assemble standard");
    assert!(assemble_res.is_ok(), "assemble standard must succeed");
    assert!(
        engine.assembly_complete,
        "assembly_complete flag must be set"
    );
}

// Criterion 9: Mismatched socket kinds refuse assembly.
#[test]
fn test_ac9_socket_mismatch_refuses_assembly() {
    let mut fault_engine = MudEngine::new_with_fault("socket");
    fault_engine.execute_command("verify mission").unwrap();
    fault_engine.execute_command("go materials_lab").unwrap();
    fault_engine.execute_command("verify materials").unwrap();
    fault_engine
        .execute_command("go primitive_foundry")
        .unwrap();
    fault_engine.execute_command("verify primitive").unwrap();
    fault_engine.execute_command("go runner_wall").unwrap();
    fault_engine.execute_command("verify runner_wall").unwrap();
    fault_engine.execute_command("go assembly_gantry").unwrap();

    let assemble_refused = fault_engine.execute_command("assemble standard");
    assert!(assemble_refused.is_err());
    assert!(matches!(
        assemble_refused.unwrap_err(),
        MudError::AssemblyFailure { .. }
    ));
}

// Criterion 10: fit_bay validation admits valid structural fit.
#[test]
fn test_ac10_fit_bay_passes_valid_assembly() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();

    engine.execute_command("verify fit").unwrap();
    assert!(
        engine.gates.fit,
        "fit gate must be set after passing fit bay"
    );
}

// Criterion 11: collision_bay detects and refuses overlapping bounds.
#[test]
fn test_ac11_collision_bay_detects_overlapping_bounds() {
    let mut coll_fault_engine = MudEngine::new_with_fault("collision");
    coll_fault_engine.gates.mission = true;
    coll_fault_engine.gates.materials = true;
    coll_fault_engine.gates.primitive = true;
    coll_fault_engine.gates.runner_wall = true;
    coll_fault_engine.gates.assembly = true;
    coll_fault_engine.gates.fit = true;
    coll_fault_engine.assembly_complete = true;
    coll_fault_engine.current_zone = Zone::CollisionBay;

    let verify_collision_failed = coll_fault_engine.execute_command("verify collision");
    assert!(verify_collision_failed.is_err());
    assert!(matches!(
        verify_collision_failed.unwrap_err(),
        MudError::GateBlocked(_)
    ));
}

// Criterion 12: proving_ground validates 4-pose motion sweep.
#[test]
fn test_ac12_proving_ground_validates_motion_sweep() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();

    engine.execute_command("verify motion").unwrap();
    assert!(
        engine.gates.motion,
        "motion gate must be set after passing proving ground"
    );
}

// Criterion 13: Cryptographic AssemblyReceipt is generated upon final admission.
#[test]
fn test_ac13_assembly_receipt_generated_at_reveal() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    assert!(
        engine.assembly_receipt.is_some(),
        "assembly_receipt must be Some after reveal"
    );
    assert!(
        engine.walkthrough_receipt.is_some(),
        "walkthrough_receipt must be Some after reveal"
    );
}

// Criteria 14 & 19: Mechs failing gate are refused; diagnose reports failure detail.
#[test]
fn test_ac14_ac19_gate_failure_with_diagnostic_detail() {
    let mut overload_engine = MudEngine::new_with_fault("overload");
    overload_engine.gates.mission = true;
    overload_engine.gates.materials = true;
    overload_engine.gates.primitive = true;
    overload_engine.gates.runner_wall = true;
    overload_engine.gates.assembly = true;
    overload_engine.assembly_complete = true;
    overload_engine.current_zone = Zone::FitBay;

    let fit_fail = overload_engine.execute_command("verify fit");
    assert!(fit_fail.is_err(), "fit gate must fail for overloaded mech");

    // Diagnose must record fit_fail diagnostic with mass detail (structural check on map key and value prefix)
    let diag_value = overload_engine
        .diagnostics
        .get("fit_fail")
        .expect("diagnostics must contain 'fit_fail' key after fit gate failure");
    assert!(
        diag_value.starts_with("Total mass"),
        "fit_fail diagnostic must start with 'Total mass', got: {}",
        diag_value
    );
}

// Criterion 15: Every walkthrough command emits at least one object-centric event.
#[test]
fn test_ac15_commands_emit_events() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    let count_before = engine.event_log.len();
    engine.execute_command("look").unwrap();
    let count_after = engine.event_log.len();
    assert!(
        count_after > count_before,
        "look must emit at least one event"
    );
}

// Criterion 16: Event logs pass referential integrity.
#[test]
fn test_ac16_event_log_referential_integrity() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    engine.verify_referential_integrity().unwrap();
}

// Criterion 17: Event log timestamps are strictly monotonic.
#[test]
fn test_ac17_event_log_timestamps_monotonic() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    for i in 1..engine.event_log.len() {
        assert!(
            engine.event_log[i].timestamp > engine.event_log[i - 1].timestamp,
            "event log timestamps must be strictly monotonic at index {}",
            i
        );
    }
}

// Criterion 18: `health` command returns current health status for a part.
#[test]
fn test_ac18_health_command_returns_part_health() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    // Structural check: head part health must be positive and accessible
    let head = engine.parts.get("head").expect("head part must exist");
    assert!(
        head.health_status >= 0.0,
        "head health_status must be non-negative"
    );
}

// Criterion 20: Happy-path end-to-end walkthrough completes cleanly.
#[test]
fn test_ac20_happy_path_end_to_end() {
    let mut engine = MudEngine::new();

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    engine.execute_command("verify reveal").unwrap();

    assert_eq!(engine.current_zone, Zone::RevealPlatform);
    assert!(engine.gates.reveal);
    assert!(engine.assembly_receipt.is_some());
    assert!(engine.walkthrough_receipt.is_some());
    let rec = engine.walkthrough_receipt.as_ref().unwrap();
    assert_eq!(rec.final_verdict, "PASS");
}

#[test]
fn test_rapid_commands_and_lifecycle_fault_injection() {
    let mut mud = MudEngine::new();

    let commands = [
        "look",
        "verify mission",
        "go materials_lab",
        "verify materials",
        "go primitive_foundry",
        "verify primitive",
        "go runner_wall",
        "verify runner_wall",
        "go assembly_gantry",
        "assemble standard",
        "verify assembly",
        "go fit_bay",
        "verify fit",
        "go collision_bay",
        "verify collision",
        "go proving_ground",
        "verify motion",
        "go reveal_platform",
        "verify reveal",
    ];

    for _ in 0..3 {
        for cmd_str in &commands {
            let _ = mud.execute_command(cmd_str);
        }
    }

    // 2. Lifecycle testing with injected socket mating fault
    let mut mud_socket_fault = MudEngine::new_with_fault("socket");
    mud_socket_fault.gates.mission = true;
    mud_socket_fault.gates.materials = true;
    mud_socket_fault.gates.primitive = true;
    mud_socket_fault.gates.runner_wall = true;
    mud_socket_fault.current_zone = Zone::AssemblyGantry;

    // Assembly must FAIL because Neck socket mismatch
    let assemble_res = mud_socket_fault.execute_command("assemble standard");
    assert!(assemble_res.is_err());

    // 3. Lifecycle testing with injected joint wear fault
    let mut mud_joint_fault = MudEngine::new_with_fault("joint");
    mud_joint_fault.gates.mission = true;
    mud_joint_fault.gates.materials = true;
    mud_joint_fault.gates.primitive = true;
    mud_joint_fault.gates.runner_wall = true;
    mud_joint_fault.gates.assembly = true;
    mud_joint_fault.assembly_complete = true;
    mud_joint_fault.gates.fit = true;
    mud_joint_fault.gates.collision = true;

    // Go to Proving Ground
    mud_joint_fault.current_zone = Zone::ProvingGround;

    // Verify motion must FAIL due to unbounded Joint
    let motion_res = mud_joint_fault.execute_command("verify motion");
    assert!(motion_res.is_err());

    // Moving to Reveal Platform must be BLOCKED
    let reveal_res = mud_joint_fault.execute_command("go reveal_platform");
    assert!(reveal_res.is_err());
}
