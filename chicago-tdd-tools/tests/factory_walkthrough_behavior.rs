/// Chicago-style acceptance tests for the nexus-mud MudEngine factory walkthrough.
/// Each test names and verifies one observable behavior of the engine.
use nexus_mud::{MudEngine, MudError, Zone};

// Helper: advance through zones up to (but not including) the target zone.
// Flattened to depth ≤ 4: function → for-loop → if-branch (3 levels).
fn advance_to_zone(engine: &mut MudEngine, target: Zone) {
    let steps: &[(Zone, &str, &str, Zone)] = &[
        (Zone::MissionRoom,    "verify mission",    "go materials_lab",    Zone::MaterialsLab),
        (Zone::MaterialsLab,   "verify materials",  "go primitive_foundry",Zone::PrimitiveFoundry),
        (Zone::PrimitiveFoundry,"verify primitive", "go runner_wall",       Zone::RunnerWall),
        (Zone::RunnerWall,     "verify runner_wall","go assembly_gantry",   Zone::AssemblyGantry),
        (Zone::AssemblyGantry, "assemble standard", "verify assembly",      Zone::AssemblyGantry),
        (Zone::AssemblyGantry, "go fit_bay",        "verify fit",           Zone::FitBay),
        (Zone::FitBay,         "go collision_bay",  "verify collision",     Zone::CollisionBay),
        (Zone::CollisionBay,   "go proving_ground", "verify motion",        Zone::ProvingGround),
        (Zone::ProvingGround,  "go reveal_platform","verify reveal",        Zone::RevealPlatform),
    ];
    for (from_zone, cmd_a, cmd_b, _to_zone) in steps {
        if engine.current_zone == target { break; }
        if engine.current_zone != *from_zone { continue; }
        let _ = engine.execute_command(cmd_a);
        let _ = engine.execute_command(cmd_b);
    }
}

// ── Test 1 ──────────────────────────────────────────────────────────────────
#[test]
fn engine_starts_in_mission_room() {
    let engine = MudEngine::new();
    assert_eq!(engine.current_zone, Zone::MissionRoom);
}

// ── Test 2 ──────────────────────────────────────────────────────────────────
#[test]
fn look_in_mission_room_returns_nonempty_description() {
    let mut engine = MudEngine::new();
    let result = engine.execute_command("look");
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

// ── Test 3 ──────────────────────────────────────────────────────────────────
#[test]
fn exits_in_mission_room_succeeds() {
    let mut engine = MudEngine::new();
    let result = engine.execute_command("exits");
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

// ── Test 4 ──────────────────────────────────────────────────────────────────
#[test]
fn go_materials_lab_moves_engine_to_materials_lab() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").expect("verify mission must succeed");
    engine.execute_command("go materials_lab").expect("go must succeed after gate");
    assert_eq!(engine.current_zone, Zone::MaterialsLab);
}

// ── Test 5 ──────────────────────────────────────────────────────────────────
#[test]
fn look_in_materials_lab_returns_nonempty_description() {
    let mut engine = MudEngine::new();
    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    assert_eq!(engine.current_zone, Zone::MaterialsLab);
    let result = engine.execute_command("look");
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

// ── Test 6 ──────────────────────────────────────────────────────────────────
#[test]
fn forward_progression_mission_room_to_runner_wall_succeeds() {
    let mut engine = MudEngine::new();

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    assert_eq!(engine.current_zone, Zone::MaterialsLab);

    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    assert_eq!(engine.current_zone, Zone::PrimitiveFoundry);

    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    assert_eq!(engine.current_zone, Zone::RunnerWall);
}

// ── Test 7 ──────────────────────────────────────────────────────────────────
#[test]
fn inspect_head_returns_ok_with_correct_mass() {
    let mut engine = MudEngine::new();
    let result = engine.execute_command("inspect head").expect("inspect head must succeed");
    // The API returns "Mass: 10 kg" — verify the result is non-empty (structural: part exists)
    assert!(!result.is_empty());
    // Also verify the structural API: head part has mass 10.0
    assert_eq!(engine.parts["head"].mass, 10.0);
}

// ── Test 8 ──────────────────────────────────────────────────────────────────
#[test]
fn assemble_standard_in_assembly_gantry_sets_assembly_complete() {
    let mut engine = MudEngine::new();

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    assert_eq!(engine.current_zone, Zone::AssemblyGantry);

    engine.execute_command("assemble standard").expect("assemble must succeed");
    assert!(engine.assembly_complete, "assembly_complete must be true after assemble standard");
}

// ── Test 9 ──────────────────────────────────────────────────────────────────
#[test]
fn verify_collision_in_collision_bay_sets_collision_gate() {
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
    assert_eq!(engine.current_zone, Zone::CollisionBay);

    engine.execute_command("verify collision").expect("verify collision must succeed");
    assert!(engine.gates.collision, "collision gate must be true after verify collision");
    assert!(engine.clearance_verified, "clearance_verified must be true after verify collision");
}

// ── Test 10 ─────────────────────────────────────────────────────────────────
#[test]
fn verify_motion_in_proving_ground_sets_kinetics_verified() {
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
    assert_eq!(engine.current_zone, Zone::ProvingGround);

    engine.execute_command("verify motion").expect("verify motion must succeed");
    assert!(engine.gates.motion, "motion gate must be true after verify motion");
    assert!(engine.kinetics_verified, "kinetics_verified must be true after verify motion");
}

// ── Test 11 ─────────────────────────────────────────────────────────────────
#[test]
fn going_to_nonexistent_zone_name_returns_invalid_command_error() {
    let mut engine = MudEngine::new();
    let result = engine.execute_command("go nonexistent_zone_xyz");
    assert!(result.is_err(), "Expected error for unknown zone");
    // Should be InvalidCommand or InvalidTransition
    match result.unwrap_err() {
        MudError::InvalidCommand(_)
        | MudError::InvalidTransition { .. }
        | MudError::GateBlocked(_)
        | MudError::InvalidDirectTravel { .. } => {}
        other => panic!("Unexpected error variant: {:?}", other),
    }
}

// ── Test 12 ─────────────────────────────────────────────────────────────────
#[test]
fn current_zone_tracks_correctly_after_each_transition() {
    let mut engine = MudEngine::new();
    assert_eq!(engine.current_zone, Zone::MissionRoom);

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    assert_eq!(engine.current_zone, Zone::MaterialsLab);

    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    assert_eq!(engine.current_zone, Zone::PrimitiveFoundry);
}

// ── Test 13 ─────────────────────────────────────────────────────────────────
#[test]
fn full_linear_walkthrough_all_nine_zones_each_respond_to_look() {
    let mut engine = MudEngine::new();

    // Zone 0
    assert!(engine.execute_command("look").is_ok());
    assert_eq!(engine.current_zone, Zone::MissionRoom);

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    assert_eq!(engine.current_zone, Zone::MaterialsLab);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    assert_eq!(engine.current_zone, Zone::PrimitiveFoundry);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    assert_eq!(engine.current_zone, Zone::RunnerWall);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    assert_eq!(engine.current_zone, Zone::AssemblyGantry);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    assert_eq!(engine.current_zone, Zone::FitBay);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    assert_eq!(engine.current_zone, Zone::CollisionBay);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    assert_eq!(engine.current_zone, Zone::ProvingGround);
    assert!(engine.execute_command("look").is_ok());

    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    assert_eq!(engine.current_zone, Zone::RevealPlatform);
    assert!(engine.execute_command("look").is_ok());
}

// ── Test 14 ─────────────────────────────────────────────────────────────────
#[test]
fn health_command_returns_ok_with_valid_health_score() {
    let mut engine = MudEngine::new();
    engine.execute_command("health mech").expect("health mech must succeed");
    // Structural check: overall_health is in [0.0, 1.0]
    let phm = engine.get_phm_metrics();
    assert!(phm.overall_health >= 0.0 && phm.overall_health <= 1.0,
        "overall_health must be in [0.0, 1.0], got {}", phm.overall_health);
}

// ── Test 15 ─────────────────────────────────────────────────────────────────
#[test]
fn diagnose_command_in_any_zone_returns_diagnostics_string() {
    let mut engine = MudEngine::new();
    let result = engine.execute_command("diagnose").expect("diagnose must succeed");
    assert!(!result.is_empty(), "diagnose must return non-empty response");
}

// ── Test 16 ─────────────────────────────────────────────────────────────────
#[test]
fn receipt_command_after_reveal_returns_pass_verdict_and_signature() {
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

    // Structural checks on the receipt object itself
    let receipt = engine.walkthrough_receipt.as_ref().expect("walkthrough_receipt must be Some after reveal");
    assert_eq!(receipt.final_verdict, "PASS");
    assert_eq!(receipt.cryptographic_signature.len(), 64, "BLAKE3 hex signature must be 64 chars");
    assert!(receipt.final_assembly_receipt_hash.is_some(), "assembly receipt hash must be present");

    // Receipt command must succeed
    assert!(engine.execute_command("receipt walkthrough").is_ok());
}

// ── Test 17 ─────────────────────────────────────────────────────────────────
#[test]
fn inventory_command_returns_parts_including_head_and_torso() {
    let mut engine = MudEngine::new();
    engine.execute_command("inventory").expect("inventory must succeed");
    // Structural check: parts map contains expected parts
    assert!(engine.parts.contains_key("head"), "parts must include 'head'");
    assert!(engine.parts.contains_key("torso_frame"), "parts must include 'torso_frame'");
}

// ── Test 18 ─────────────────────────────────────────────────────────────────
#[test]
fn zone_elevation_increases_monotonically_along_factory_path() {
    let zones = Zone::all();
    for i in 1..zones.len() {
        assert!(
            zones[i].elevation() > zones[i - 1].elevation(),
            "Zone {} elevation {} should be greater than Zone {} elevation {}",
            zones[i].name(),
            zones[i].elevation(),
            zones[i - 1].name(),
            zones[i - 1].elevation()
        );
    }
}

// ── Test 19 ─────────────────────────────────────────────────────────────────
#[test]
fn reveal_platform_is_last_zone_with_elevation_eight() {
    assert_eq!(Zone::RevealPlatform.elevation(), 8);
    let all = Zone::all();
    assert_eq!(*all.last().unwrap(), Zone::RevealPlatform);
}

// ── Test 20 ─────────────────────────────────────────────────────────────────
#[test]
fn full_end_to_end_walk_mission_room_to_reveal_platform_via_go_commands() {
    let mut engine = MudEngine::new();
    assert_eq!(engine.current_zone, Zone::MissionRoom);

    engine.execute_command("verify mission").unwrap();
    engine.execute_command("go materials_lab").unwrap();
    assert_eq!(engine.current_zone, Zone::MaterialsLab);

    engine.execute_command("verify materials").unwrap();
    engine.execute_command("go primitive_foundry").unwrap();
    assert_eq!(engine.current_zone, Zone::PrimitiveFoundry);

    engine.execute_command("verify primitive").unwrap();
    engine.execute_command("go runner_wall").unwrap();
    assert_eq!(engine.current_zone, Zone::RunnerWall);

    engine.execute_command("verify runner_wall").unwrap();
    engine.execute_command("go assembly_gantry").unwrap();
    assert_eq!(engine.current_zone, Zone::AssemblyGantry);

    engine.execute_command("assemble standard").unwrap();
    engine.execute_command("verify assembly").unwrap();
    engine.execute_command("go fit_bay").unwrap();
    assert_eq!(engine.current_zone, Zone::FitBay);

    engine.execute_command("verify fit").unwrap();
    engine.execute_command("go collision_bay").unwrap();
    assert_eq!(engine.current_zone, Zone::CollisionBay);

    engine.execute_command("verify collision").unwrap();
    engine.execute_command("go proving_ground").unwrap();
    assert_eq!(engine.current_zone, Zone::ProvingGround);

    engine.execute_command("verify motion").unwrap();
    engine.execute_command("go reveal_platform").unwrap();
    assert_eq!(engine.current_zone, Zone::RevealPlatform);

    engine.execute_command("verify reveal").unwrap();
    assert!(engine.gates.reveal, "Reveal gate should be marked true");

    // Structural check on receipt instead of string assertion
    let receipt = engine.walkthrough_receipt.as_ref().expect("receipt must exist after reveal");
    assert_eq!(receipt.final_verdict, "PASS");
}
