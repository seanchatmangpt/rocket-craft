//! GMF OCEL Adapter — Chicago-style behavior tests.
//! Every test here validates an event from the taxonomy, not an implementation detail.
//! If the event log cannot prove a lawful process happened, then it did not happen.

use gmf_ocel_adapter::{
    events::{
        GmfEventKind, battery_inserted_event, damage_observed_event, eden_grid_health_changed,
        jidoka_halt_to_event, part_generated_event, pilot_entered_event, thermal_overload_event,
    },
    ocel::{OcelLog, OcelObject},
};
use nexus_tps::{JidokaHalt, PartSlot, PartStateVector, SocketType};

// ── Part generation events ─────────────────────────────────────────────────

#[test]
fn part_generated_event_has_correct_activity_name() {
    let psv = PartStateVector {
        civilization_id: 1,
        frame_id: 7,
        armor_profile: 0.8,
        joint_profile: 0.6,
        mass_profile: 0.72,
        weapon_profile: 0.5,
        motion_profile: 0.9,
        part_slot: PartSlot::ArmL,
    };
    let ev = part_generated_event(&psv, "part_ArmL_001", "zone_runner_wall", "ev_001", 1000);
    assert_eq!(ev.kind.activity_name(), "part.generated");
}

#[test]
fn part_generated_event_references_both_part_and_zone() {
    let psv = PartStateVector {
        civilization_id: 1,
        frame_id: 1,
        armor_profile: 0.5,
        joint_profile: 0.5,
        mass_profile: 0.5,
        weapon_profile: 0.5,
        motion_profile: 0.5,
        part_slot: PartSlot::Head,
    };
    let ev = part_generated_event(&psv, "part_Head_001", "zone_runner_wall", "ev_002", 1000);
    let ids: Vec<&str> = ev.object_refs.iter().map(|r| r.object_id.as_str()).collect();
    assert!(ids.contains(&"part_Head_001"), "Must reference generated part");
    assert!(ids.contains(&"zone_runner_wall"), "Must reference manufacturing zone");
}

#[test]
fn part_generated_event_encodes_part_slot_as_attribute() {
    let psv = PartStateVector {
        civilization_id: 3,
        frame_id: 2,
        armor_profile: 0.9,
        joint_profile: 0.7,
        mass_profile: 0.6,
        weapon_profile: 0.4,
        motion_profile: 0.8,
        part_slot: PartSlot::Backpack,
    };
    let ev = part_generated_event(&psv, "part_Backpack_001", "zone_runner_wall", "ev_003", 2000);
    let slot_attr = ev.attributes.get("part_slot").expect("must have part_slot attribute");
    assert_eq!(slot_attr.as_str(), Some("Backpack"), "part_slot attribute must be the Debug representation of PartSlot::Backpack");
}

// ── Jidoka halt events ─────────────────────────────────────────────────────

#[test]
fn jidoka_socket_mismatch_emits_correct_activity_name() {
    let halt = JidokaHalt::SocketMismatch {
        expected: SocketType::ArmL,
        got: SocketType::ArmR,
    };
    let ev = jidoka_halt_to_event(&halt, "jidoka_001", 5000);
    assert_eq!(ev.kind.activity_name(), "jidoka.halt");
    assert_eq!(ev.attributes["halt_code"].as_str(), Some("socket_mismatch"));
}

#[test]
fn jidoka_mass_exceed_emits_halt_code_attribute() {
    let halt = JidokaHalt::MassExceedsFrameCapacity { mass: 120.0, capacity: 100.0 };
    let ev = jidoka_halt_to_event(&halt, "jidoka_002", 6000);
    assert_eq!(ev.attributes["halt_code"].as_str(), Some("mass_exceeds_frame_capacity"));
    let desc = ev.attributes["halt_description"].as_str().unwrap();
    assert_eq!(desc, "Mass exceeds frame capacity: mass 120, capacity 100", "halt_description must be the exact Display string for MassExceedsFrameCapacity {{ mass: 120.0, capacity: 100.0 }}");
}

#[test]
fn jidoka_collision_emits_correct_halt_code() {
    let halt = JidokaHalt::CollisionVolumeIntersects {
        part_a: PartSlot::ArmL,
        part_b: PartSlot::Torso,
    };
    let ev = jidoka_halt_to_event(&halt, "jidoka_003", 7000);
    assert_eq!(ev.attributes["halt_code"].as_str(), Some("collision_volume_intersects"));
}

// ── Thermal overload (MechWarrior heat loop) ────────────────────────────────

#[test]
fn thermal_overload_event_activity_name_is_correct() {
    let ev = thermal_overload_event("part_ArmR_001", "zone_collision_bay", 0.98, "thermal_001", 8000);
    assert_eq!(ev.kind.activity_name(), "thermal.overload");
}

#[test]
fn thermal_overload_event_encodes_load_and_threshold_exceeded() {
    let ev = thermal_overload_event("part_Torso_001", "zone_gantry", 0.95, "thermal_002", 9000);
    let load = ev.attributes["thermal_load"].as_f64().unwrap();
    assert!((load - 0.95).abs() < 0.001);
    assert_eq!(ev.attributes["threshold_exceeded"].as_bool(), Some(true));
}

#[test]
fn thermal_overload_event_references_part_and_zone() {
    let ev = thermal_overload_event("part_LegL_001", "zone_proving_ground", 1.0, "thermal_003", 10000);
    let ids: Vec<&str> = ev.object_refs.iter().map(|r| r.object_id.as_str()).collect();
    assert!(ids.contains(&"part_LegL_001"));
    assert!(ids.contains(&"zone_proving_ground"));
}

// ── Location-based damage (BattleTech) ─────────────────────────────────────

#[test]
fn damage_observed_event_activity_name_is_correct() {
    let ev = damage_observed_event(PartSlot::Head, "part_Head_001", "attacker_X", 45.0, "dmg_001", 11000);
    assert_eq!(ev.kind.activity_name(), "damage.observed");
}

#[test]
fn damage_observed_event_encodes_damage_amount_and_slot() {
    let ev = damage_observed_event(PartSlot::ArmR, "part_ArmR_001", "attacker_X", 30.5, "dmg_002", 12000);
    let amount = ev.attributes["damage_amount"].as_f64().unwrap();
    assert!((amount - 30.5).abs() < 0.001);
    let slot = ev.attributes["part_slot"].as_str().unwrap();
    assert_eq!(slot, "ArmR", "part_slot attribute must be the Debug representation of PartSlot::ArmR");
}

// ── Pilot / cockpit (Titanfall bond) ───────────────────────────────────────

#[test]
fn pilot_entered_event_activity_name_is_correct() {
    let ev = pilot_entered_event("pilot_001", "cockpit_001", "mech_001", "pilot_enter_001", 13000);
    assert_eq!(ev.kind.activity_name(), "pilot.entered");
}

#[test]
fn pilot_entered_event_references_pilot_cockpit_and_mech() {
    let ev = pilot_entered_event("pilot_001", "cockpit_001", "mech_001", "pilot_enter_002", 14000);
    let ids: Vec<&str> = ev.object_refs.iter().map(|r| r.object_id.as_str()).collect();
    assert!(ids.contains(&"pilot_001"), "Must reference pilot");
    assert!(ids.contains(&"cockpit_001"), "Must reference cockpit");
    assert!(ids.contains(&"mech_001"), "Must reference mech");
}

// ── Battery / field support (Titanfall rodeo) ──────────────────────────────

#[test]
fn battery_inserted_event_activity_name_is_correct() {
    let ev = battery_inserted_event("bat_001", "mech_001", "agent_001", 0.5, "bat_ev_001", 15000);
    assert_eq!(ev.kind.activity_name(), "battery.inserted");
}

#[test]
fn battery_inserted_event_encodes_shield_restore() {
    let ev = battery_inserted_event("bat_001", "mech_001", "agent_001", 0.75, "bat_ev_002", 16000);
    let restore = ev.attributes["shield_restore"].as_f64().unwrap();
    assert!((restore - 0.75).abs() < 0.001);
}

// ── Infrastructure defense (Into the Breach Eden grid) ─────────────────────

#[test]
fn eden_grid_health_changed_activity_name_is_correct() {
    let ev = eden_grid_health_changed("grid_node_A1", 3, -1, "grid_ev_001", 17000);
    assert_eq!(ev.kind.activity_name(), "eden.grid_health_changed");
}

#[test]
fn eden_grid_fallen_node_emits_fallen_standing() {
    let ev = eden_grid_health_changed("grid_node_B2", 0, -3, "grid_ev_002", 18000);
    assert_eq!(ev.attributes["civilization_standing"].as_str(), Some("fallen"));
}

#[test]
fn eden_grid_healthy_node_emits_standing() {
    let ev = eden_grid_health_changed("grid_node_C3", 5, 1, "grid_ev_003", 19000);
    assert_eq!(ev.attributes["civilization_standing"].as_str(), Some("standing"));
}

// ── OCEL log validation ─────────────────────────────────────────────────────

#[test]
fn ocel_log_validates_referential_integrity() {
    let mut log = OcelLog::default();

    let part_obj = OcelObject::new("part_ArmL_001", "MechPart");
    let zone_obj = OcelObject::new("zone_runner_wall", "ManufacturingZone");
    log.add_object(part_obj);
    log.add_object(zone_obj);

    let psv = PartStateVector {
        civilization_id: 1,
        frame_id: 7,
        armor_profile: 0.8,
        joint_profile: 0.6,
        mass_profile: 0.72,
        weapon_profile: 0.5,
        motion_profile: 0.9,
        part_slot: PartSlot::ArmL,
    };
    let ev = part_generated_event(&psv, "part_ArmL_001", "zone_runner_wall", "ev_001", 1000);
    log.add_event(ev.into_ocel_event());

    let violations = log.validate();
    assert!(violations.is_empty(), "Valid OCEL log must have no violations: {violations:?}");
}

#[test]
fn ocel_log_detects_unknown_object_reference() {
    let mut log = OcelLog::default();
    // No objects added — but event references them
    let psv = PartStateVector {
        civilization_id: 1,
        frame_id: 1,
        armor_profile: 0.5,
        joint_profile: 0.5,
        mass_profile: 0.5,
        weapon_profile: 0.5,
        motion_profile: 0.5,
        part_slot: PartSlot::Torso,
    };
    let ev = part_generated_event(&psv, "part_Torso_001", "zone_runner_wall", "ev_bad", 1000);
    log.add_event(ev.into_ocel_event());

    let violations = log.validate();
    assert!(!violations.is_empty(), "Referencing non-existent objects must produce violations");
}

#[test]
fn ocel_log_detects_non_monotonic_attribute_timestamps() {
    let mut log = OcelLog::default();

    let obj = OcelObject::new("mech_001", "MechAssembly")
        .with_attr_change("status", "assembling", 2000)
        .with_attr_change("status", "idle", 1000); // backward — non-monotonic
    log.add_object(obj);

    let violations = log.validate();
    assert!(!violations.is_empty(), "Non-monotonic timestamps must produce violations");
}

// ── Full factory walkthrough OCEL construction ─────────────────────────────

#[test]
fn factory_ocel_generate_jidoka_andon_sequence_is_constructible() {
    // Simulate: generate part → jidoka halt → andon open → rework → generate again
    let mut log = OcelLog::default();

    log.add_object(OcelObject::new("part_ArmL_001", "MechPart"));
    log.add_object(OcelObject::new("zone_runner_wall", "ManufacturingZone"));
    log.add_object(OcelObject::new("part_ArmL_002", "MechPart"));

    let psv = PartStateVector {
        civilization_id: 1,
        frame_id: 7,
        armor_profile: 0.8,
        joint_profile: 0.6,
        mass_profile: 1.5, // intentionally over mass — will cause halt in production
        weapon_profile: 0.5,
        motion_profile: 0.9,
        part_slot: PartSlot::ArmL,
    };

    log.add_event(
        part_generated_event(&psv, "part_ArmL_001", "zone_runner_wall", "ev_gen_001", 1000)
            .into_ocel_event(),
    );

    let halt = JidokaHalt::MassExceedsFrameCapacity { mass: 150.0, capacity: 100.0 };
    log.add_event(jidoka_halt_to_event(&halt, "ev_halt_001", 2000).into_ocel_event());

    // Rework: generate corrected part
    let psv2 = PartStateVector { mass_profile: 0.72, ..psv };
    log.add_event(
        part_generated_event(&psv2, "part_ArmL_002", "zone_runner_wall", "ev_gen_002", 3000)
            .into_ocel_event(),
    );

    let violations = log.validate();
    assert!(violations.is_empty(), "Factory OCEL must be valid: {violations:?}");
    assert_eq!(log.events.len(), 3, "Must have 3 events: gen → halt → rework gen");
}

// ── Event taxonomy completeness check ──────────────────────────────────────

#[test]
fn all_event_kinds_have_non_empty_activity_name() {
    // Exhaustive spot-check of representative kinds
    let representative = vec![
        GmfEventKind::PartGenerated,
        GmfEventKind::JidokaHaltEmitted,
        GmfEventKind::AndonOpened,
        GmfEventKind::ReceiptIssued,
        GmfEventKind::ThermalOverload,
        GmfEventKind::DamageObserved,
        GmfEventKind::PilotEntered,
        GmfEventKind::PilotEjected,
        GmfEventKind::BatteryInserted,
        GmfEventKind::EdenGridHealthChanged,
        GmfEventKind::MissionContractAccepted,
        GmfEventKind::MapeKCycleExecuted,
        GmfEventKind::WeibullRulUpdated,
        GmfEventKind::ZoneCircuitOpened,
        GmfEventKind::MaintenanceRequested,
        GmfEventKind::ReturnToServiceAdmitted,
    ];

    for kind in representative {
        let name = kind.activity_name();
        assert!(!name.is_empty(), "activity_name must not be empty");
        assert!(name.contains('.'), "activity_name must be dot-namespaced: '{name}'");
    }
}
