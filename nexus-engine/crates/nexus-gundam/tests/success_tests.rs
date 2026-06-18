use nexus_gundam::builder::{MechBuilder, CivilizationBuilder, ValidationError};
use nexus_gundam::generated_gundam::{
    Walking, Earth, Mars, Venus, Frame, Power, Armor, Weapon, Sensor, UtilitySystem, Joint, Mobility, AABB, RotationLimits
};
use nexus_gundam::preservation::{PreservationArtifact, PreservationLayer, GundamPreservationManager};
use nexus_gundam::simulation::{SimulationInterface, GundamNexusSimulation, ExperiencePhase};
use std::collections::HashMap;

fn get_valid_mobility() -> Walking {
    Walking {
        physical: Mobility {
            id: "mobility_system".to_string(),
            mass: 15.0,
            occupancy: AABB::new([-0.5, 0.0, -0.5], [0.5, 0.9, 0.5]),
            clearance: AABB::new([-0.52, -0.02, -0.52], [0.52, 0.92, 0.52]),
            load_capacity: 500.0,
            max_speed: 50.0,
        },
        leg_count: 2,
    }
}

fn get_valid_frame() -> Frame {
    Frame {
        id: "frame_system".to_string(),
        mass: 50.0,
        occupancy: AABB::new([-0.5, 1.0, -0.5], [0.5, 2.0, 0.5]),
        clearance: AABB::new([-0.52, 0.98, -0.52], [0.52, 2.02, 0.52]),
        slot_count: 6,
    }
}

fn get_valid_power() -> Power {
    Power {
        id: "power_system".to_string(),
        mass: 20.0,
        occupancy: AABB::new([-0.5, 2.1, -0.5], [0.5, 3.0, 0.5]),
        clearance: AABB::new([-0.52, 2.08, -0.52], [0.52, 3.02, 0.52]),
        energy_capacity: 1000.0,
        output: 100.0,
    }
}

fn get_valid_armor() -> Armor {
    Armor {
        id: "armor_system".to_string(),
        mass: 30.0,
        occupancy: AABB::new([-0.5, 3.1, -0.5], [0.5, 4.0, 0.5]),
        clearance: AABB::new([-0.52, 3.08, -0.52], [0.52, 4.02, 0.52]),
        defense_rating: 150.0,
        material: "Luna Titanium".to_string(),
    }
}

#[test]
fn test_mech_success() {
    let walking_mobility = get_valid_mobility();

    let mech = MechBuilder::new()
        .with_frame(get_valid_frame())
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .add_weapon(Weapon {
            id: "beam_rifle".to_string(),
            mass: 5.0,
            occupancy: AABB::new([0.6, 1.0, -0.5], [1.0, 2.0, 0.5]),
            clearance: AABB::new([0.55, 0.95, -0.52], [1.05, 2.02, 0.52]),
            damage: 80.0,
            range: 300.0,
        })
        .add_sensor(Sensor {
            id: "dual_sensors".to_string(),
            mass: 2.0,
            occupancy: AABB::new([-0.5, 4.1, -0.5], [0.5, 4.5, 0.5]),
            clearance: AABB::new([-0.52, 4.08, -0.52], [0.52, 4.52, 0.52]),
            detection_range: 500.0,
        })
        .add_utility(UtilitySystem {
            id: "core_block".to_string(),
            mass: 10.0,
            occupancy: AABB::new([-0.5, 4.6, -0.5], [0.5, 5.0, 0.5]),
            clearance: AABB::new([-0.52, 4.58, -0.52], [0.52, 5.02, 0.52]),
            utility_type: "Escape Pod".to_string(),
        })
        .add_joint(Joint {
            name: "Waist".to_string(),
            parent_component_id: "frame_system".to_string(),
            child_component_id: "mobility_system".to_string(),
            location: [0.0, 0.0, 0.0],
            limits: Some(RotationLimits {
                min_yaw: -90.0,
                max_yaw: 90.0,
                min_pitch: -30.0,
                max_pitch: 45.0,
                min_roll: -10.0,
                max_roll: 10.0,
            }),
            mass: 3.0,
        })
        .with_class("Warrior")
        .build();

    assert_eq!(mech.frame.id, "frame_system");
    assert_eq!(mech.power.id, "power_system");
    assert_eq!(mech.armor.id, "armor_system");
    assert_eq!(mech.weapons[0].id, "beam_rifle");
    assert_eq!(mech.sensors[0].id, "dual_sensors");
    assert_eq!(mech.utility[0].id, "core_block");
    assert_eq!(mech.class, "Warrior");

    assert!(mech.validate().is_ok());

    let receipt = mech.generate_receipt().unwrap();
    assert!(!receipt.lineage_hash.is_empty());
    assert_eq!(receipt.mobility_type, "Walking");
}

#[test]
fn test_missing_joint_limits() {
    let walking_mobility = get_valid_mobility();
    let mech = MechBuilder::new()
        .with_frame(get_valid_frame())
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .add_joint(Joint {
            name: "Waist".to_string(),
            parent_component_id: "frame_system".to_string(),
            child_component_id: "mobility_system".to_string(),
            location: [0.0, 0.0, 0.0],
            limits: None,
            mass: 3.0,
        })
        .build();

    let res = mech.validate();
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ValidationError::MissingJointLimits { .. }));
}

#[test]
fn test_invalid_joint_limits() {
    let walking_mobility = get_valid_mobility();
    let mech = MechBuilder::new()
        .with_frame(get_valid_frame())
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .add_joint(Joint {
            name: "Waist".to_string(),
            parent_component_id: "frame_system".to_string(),
            child_component_id: "mobility_system".to_string(),
            location: [0.0, 0.0, 0.0],
            limits: Some(RotationLimits {
                min_yaw: 90.0,
                max_yaw: -90.0,
                min_pitch: -30.0,
                max_pitch: 45.0,
                min_roll: -10.0,
                max_roll: 10.0,
            }),
            mass: 3.0,
        })
        .build();

    let res = mech.validate();
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ValidationError::InvalidJointLimits { .. }));
}

#[test]
fn test_collision_detected() {
    let walking_mobility = get_valid_mobility();
    let mech = MechBuilder::new()
        .with_frame(Frame {
            id: "frame_system".to_string(),
            mass: 50.0,
            occupancy: AABB::new([-0.5, 1.0, -0.5], [0.5, 2.5, 0.5]),
            clearance: AABB::new([-0.52, 0.98, -0.52], [0.52, 2.52, 0.52]),
            slot_count: 6,
        })
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .build();

    let res = mech.validate();
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ValidationError::CollisionDetected { .. }));
}

#[test]
fn test_clearance_violation() {
    let walking_mobility = get_valid_mobility();
    let mech = MechBuilder::new()
        .with_frame(Frame {
            id: "frame_system".to_string(),
            mass: 50.0,
            occupancy: AABB::new([-0.5, 1.0, -0.5], [0.5, 2.0, 0.5]),
            clearance: AABB::new([-0.52, 0.98, -0.52], [0.52, 2.15, 0.52]),
            slot_count: 6,
        })
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .build();

    let res = mech.validate();
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ValidationError::ClearanceViolation { .. }));
}

#[test]
fn test_load_capacity_exceeded() {
    let walking_mobility = Walking {
        physical: Mobility {
            id: "mobility_system".to_string(),
            mass: 15.0,
            occupancy: AABB::new([-0.5, 0.0, -0.5], [0.5, 0.9, 0.5]),
            clearance: AABB::new([-0.52, -0.02, -0.52], [0.52, 0.92, 0.52]),
            load_capacity: 50.0,
            max_speed: 50.0,
        },
        leg_count: 2,
    };
    let mech = MechBuilder::new()
        .with_frame(get_valid_frame())
        .with_mobility(walking_mobility)
        .with_power(get_valid_power())
        .with_armor(get_valid_armor())
        .build();

    let res = mech.validate();
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ValidationError::LoadCapacityExceeded { .. }));
}

#[test]
fn test_planetary_incompatibility() {
    use nexus_gundam::builder::PlanetMechCompatibility;
    use nexus_gundam::generated_gundam::{Earth, Mars, Venus};

    let earth = Earth;
    let mars = Mars;
    let venus = Venus;

    assert!(mars.validate_compatibility("Warrior", "Walking").is_err());
    assert!(mars.validate_compatibility("Guardian", "Walking").is_ok());

    assert!(venus.validate_compatibility("Warrior", "Walking").is_err());
    assert!(venus.validate_compatibility("Warrior", "Flight").is_ok());

    assert!(earth.validate_compatibility("Warrior", "Walking").is_ok());
}

#[test]
fn test_civilization_spawns() {
    let earth_civ = CivilizationBuilder::new()
        .with_planet(Earth)
        .build();
    let earth_builder = earth_civ.spawn_mech_builder();
    assert_eq!(earth_builder.class.unwrap(), "Warrior");
    assert_eq!(earth_builder.frame.unwrap().id, "Earth_Balanced_Frame");
    assert_eq!(earth_builder.armor.unwrap().id, "Luna_Titanium_Armor");

    let mars_civ = CivilizationBuilder::new()
        .with_planet(Mars)
        .build();
    let mars_builder = mars_civ.spawn_mech_builder();
    assert_eq!(mars_builder.class.unwrap(), "Guardian");
    assert_eq!(mars_builder.frame.unwrap().id, "Mars_Heavy_Frame");
    assert_eq!(mars_builder.armor.unwrap().id, "Mars_Heavy_Chobham_Armor");

    let venus_civ = CivilizationBuilder::new()
        .with_planet(Venus)
        .build();
    let venus_builder = venus_civ.spawn_mech_builder();
    assert_eq!(venus_builder.class.unwrap(), "Explorer");
    assert_eq!(venus_builder.frame.unwrap().id, "Venus_Lightweight_Frame");
    assert_eq!(venus_builder.power.unwrap().id, "Venus_High_Output_Reactor");
    assert_eq!(venus_builder.armor.unwrap().id, "Venus_Aerodynamic_Composite_Armor");
}

#[test]
fn test_civilization_success() {
    let civ = CivilizationBuilder::new()
        .with_planet(Earth)
        .with_name("Universal Century")
        .add_history("One Year War")
        .add_value("Expansion")
        .with_environment("Space Colonies")
        .add_resource("Minovsky Particles")
        .build();

    assert_eq!(civ.name, "Universal Century");
    assert_eq!(civ.history[0], "One Year War");
    assert_eq!(civ.environment, "Space Colonies");
}

#[test]
fn test_preservation_success() {
    use nexus_gundam::generated_gundam::FlashGames;

    let manager = GundamPreservationManager::new();
    let mut metadata = HashMap::new();
    metadata.insert("year".to_string(), "2002".to_string());

    let artifact = PreservationArtifact {
        id: "save-01".to_string(),
        domain: FlashGames,
        name: "Gundam Battle Royale".to_string(),
        original_url: Some("http://example.com/gundam".to_string()),
        metadata,
        assets_blob: vec![1, 2, 3, 4, 5],
        hash: String::new(),
    };

    let hash = manager.preserve_artifact(artifact.clone()).unwrap();
    assert!(!hash.is_empty());

    let retrieved = manager.retrieve_artifact::<FlashGames>("save-01").unwrap();
    assert_eq!(retrieved.name, "Gundam Battle Royale");
    assert_eq!(retrieved.hash, hash);

    let listed = manager.list_artifacts_by_domain(FlashGames).unwrap();
    assert_eq!(listed.len(), 1);

    let verified = manager.verify_integrity("save-01").unwrap();
    assert!(verified);
}

#[test]
fn test_simulation_success() {
    let mut sim = GundamNexusSimulation::new();
    let state = sim.get_state();
    assert_eq!(state.current_phase, ExperiencePhase::Explore);

    // Spawn mech
    sim.spawn_mech("Zaku II", "Warrior").unwrap();
    // Form civilization
    sim.form_civilization("Zeon", "Mars").unwrap();

    let state = sim.get_state();
    assert_eq!(state.active_mechs, 1);
    assert_eq!(state.civilization_count, 1);

    // Step phase
    let next_phase = sim.step_phase().unwrap();
    assert_eq!(next_phase, ExperiencePhase::Discover);

    let updated_state = sim.run_simulation_cycle().unwrap();
    assert_eq!(updated_state.current_phase, ExperiencePhase::Discover);
}
