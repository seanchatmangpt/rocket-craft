use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use nexus_tps::{PartSlot, PartStateVector};
// These enums mirror wasm4pm::self_healing but are defined locally to respect
// the wasm4pm-compat boundary. nexus-mud must not cross into full wasm4pm authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    HalfOpen,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Nine factory zones representing the GMF physical spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Zone {
    MissionRoom = 0,
    MaterialsLab = 1,
    PrimitiveFoundry = 2,
    RunnerWall = 3,
    AssemblyGantry = 4,
    FitBay = 5,
    CollisionBay = 6,
    ProvingGround = 7,
    RevealPlatform = 8,
}

impl Zone {
    pub fn all() -> Vec<Zone> {
        vec![
            Zone::MissionRoom,
            Zone::MaterialsLab,
            Zone::PrimitiveFoundry,
            Zone::RunnerWall,
            Zone::AssemblyGantry,
            Zone::FitBay,
            Zone::CollisionBay,
            Zone::ProvingGround,
            Zone::RevealPlatform,
        ]
    }

    pub fn parse(s: &str) -> Option<Self> {
        let clean = s.trim().to_lowercase().replace("_", " ").replace("-", " ");
        match clean.as_str() {
            "mission room" | "mission_room" | "mission" => Some(Zone::MissionRoom),
            "materials lab" | "materials_lab" | "materials" => Some(Zone::MaterialsLab),
            "primitive foundry" | "primitive_foundry" | "primitive" | "foundry" => {
                Some(Zone::PrimitiveFoundry)
            }
            "runner wall" | "runner_wall" | "runner" => Some(Zone::RunnerWall),
            "assembly gantry" | "assembly_gantry" | "assembly" | "gantry" => {
                Some(Zone::AssemblyGantry)
            }
            "fit bay" | "fit_bay" | "fit" => Some(Zone::FitBay),
            "collision bay" | "collision_bay" | "collision" => Some(Zone::CollisionBay),
            "proving ground" | "proving_ground" | "proving" => Some(Zone::ProvingGround),
            "reveal platform" | "reveal_platform" | "reveal" => Some(Zone::RevealPlatform),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Zone::MissionRoom => "mission_room",
            Zone::MaterialsLab => "materials_lab",
            Zone::PrimitiveFoundry => "primitive_foundry",
            Zone::RunnerWall => "runner_wall",
            Zone::AssemblyGantry => "assembly_gantry",
            Zone::FitBay => "fit_bay",
            Zone::CollisionBay => "collision_bay",
            Zone::ProvingGround => "proving_ground",
            Zone::RevealPlatform => "reveal_platform",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Zone::MissionRoom => "Mission Room",
            Zone::MaterialsLab => "Materials Lab",
            Zone::PrimitiveFoundry => "Primitive Foundry",
            Zone::RunnerWall => "Runner Wall",
            Zone::AssemblyGantry => "Assembly Gantry",
            Zone::FitBay => "Fit Bay",
            Zone::CollisionBay => "Collision Bay",
            Zone::ProvingGround => "Proving Ground",
            Zone::RevealPlatform => "Reveal Platform",
        }
    }

    pub fn elevation(&self) -> i32 {
        *self as i32
    }

    pub fn description(&self) -> &'static str {
        match self {
            Zone::MissionRoom => "Zone 0: The Command and Mission briefing room. The Strategic blueprint of the mech's operational journey begins here.",
            Zone::MaterialsLab => "Zone 1: A lab where metallurgy and chemical composition of the armor plates are verified.",
            Zone::PrimitiveFoundry => "Zone 2: Thermal forge where raw materials are cast into basic parts (bones, armor plates, and joints).",
            Zone::RunnerWall => "Zone 3: Rack systems holding physicalized part runners and templates for the assembly line.",
            Zone::AssemblyGantry => "Zone 4: Giant mechanical rig where robotic arms assemble parts onto the central torso frame.",
            Zone::FitBay => "Zone 5: Physical alignment bay checking socket mating and structural completeness.",
            Zone::CollisionBay => "Zone 6: Laser scanning bay detecting physical bounds overlap and geometry clearances.",
            Zone::ProvingGround => "Zone 7: Testing fields checking multi-pose motion kinematics (stand, step, turn, kneel).",
            Zone::RevealPlatform => "Zone 8: Final ceremonial launchpad where the admitted mech is presented under open sky.",
        }
    }
}

/// 3D Axis-Aligned Bounding Box (AABB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AABB {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
            && self.min_z < other.max_z
            && self.max_z > other.min_z
    }
}

/// Socket Connection specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketConnection {
    pub name: String,
    pub kind: String,
}

/// Mech Part specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechPart {
    pub part_id: String,
    pub part_kind: String,
    pub mass: f32,
    pub bounds: AABB,
    pub sockets_required: Vec<SocketConnection>,
    pub sockets_provided: Vec<SocketConnection>,
    pub health_status: f32,
    pub admission_status: String,
}

/// Joint definition between parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub name: String,
    pub parent_part: String,
    pub child_part: String,
    pub rotation_limits: Option<[f32; 2]>,
}

/// Gate Status representing verified progress
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateStatus {
    pub mission: bool,
    pub materials: bool,
    pub primitive: bool,
    pub runner_wall: bool,
    pub assembly: bool,
    pub fit: bool,
    pub collision: bool,
    pub motion: bool,
    pub reveal: bool,
}

/// Monotonic Event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub object_id: String,
    pub details: String,
}

/// Dynamic Twin Entity representing the physical/digital state of a manufactured component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinPart {
    pub name: String,
    pub geometry: u64,
    pub socket_fit: u64,
    pub motion_clearance: u64,
    pub collision_volume: u64,
    pub mass_balance: u64,
    pub physics_role: u64,
    pub assembly_compatibility: u64,
    pub health_score: f32,
    pub wear_rate: f32,
    pub thermal_load: f32,
    pub fault_status: String,
}

/// Socket Mating details tracked in the Digital Twin state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinSocket {
    pub name: String,
    pub mated: bool,
    pub stress_level: f32,
    pub mated_to: Option<String>,
    pub mating_rule: String,
}

/// Joint Wear details tracked in the Digital Twin state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinJoint {
    pub name: String,
    pub wear_level: f32,
    pub motion_failures: u32,
    pub status: String,
}

/// PHM (Prognostics and Health Management) Metrics for parts, sockets, joints, and the overall Mech.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhmMetrics {
    pub overall_health: f32,
    pub remaining_useful_life: f32,
    pub active_faults: Vec<String>,
    pub degradation_state: String,
    pub maintenance_state: String,
    pub socket_stress: f32,
    pub joint_wear: f32,
    pub collision_incidents: u32,
    pub thermal_load: f32,
    pub motion_failures: u32,
}

/// The Validation Pipeline mapping raw geometry to final Jidoka action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPipeline {
    pub geometry_readings: Vec<String>,
    pub clearance_features: Vec<String>,
    pub violation_masks: u32,
    pub classification: String,
    pub andon_action: String,
}

/// Custom Errors for the MUD Walkthrough engine.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum MudError {
    #[error("Assembly failure: {reason}")]
    AssemblyFailure { reason: String },

    #[error("Kinetics verification required before entering the Reveal Platform")]
    KineticsRequired,

    #[error("Invalid room transition: cannot move {direction} from {current_zone}")]
    InvalidTransition {
        direction: String,
        current_zone: String,
    },

    #[error("Invalid direct travel: cannot go to {destination} from {current_zone}")]
    InvalidDirectTravel {
        destination: String,
        current_zone: String,
    },

    #[error("Gate blocked: {0}")]
    GateBlocked(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid inspect target: {0}")]
    InvalidInspectTarget(String),

    #[error("Operation error: {0}")]
    OperationError(String),
}

/// Text Commands supported by the MUD parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Go(String),
    Look,
    Inspect(String),
    WatchAssembly,
    VerifyClearance,
    VerifyKinetics,
    StressPart(String),
}

/// The walkthrough replay receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkthroughReceipt {
    pub prompt: String,
    pub command_history: Vec<String>,
    pub command_trace: Vec<String>,
    pub room_transitions: Vec<(Zone, Zone)>,
    pub final_verdict: String,
    pub final_assembly_receipt_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub cryptographic_signature: String,
    pub signature_hash: String,
}

/// Assembly Receipt mapping lineage and admission hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyReceipt {
    pub mech_id: String,
    pub parts_hashes: HashMap<String, String>,
    pub lineage_hash: String,
    pub timestamp: DateTime<Utc>,
}

/// Gantry state details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GantryState {
    pub verification_status: String,
    pub current_state: String,
}

/// PHM Report for Mech
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechPhmReport {
    pub degradation_state: String,
    pub fault_state: String,
}

/// Non-interactive command parsed representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MudCommand {
    Look,
    Go(String),
    Inspect(String),
    Health(String),
    Diagnose(String),
    Verify(String),
    Assemble(String),
    Preview(String),
    Receipt(String),
    Inventory,
    Exits,
}

/// The Gundam Nexus Manufacturing Facility MUD Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudEngine {
    pub current_zone: Zone,
    pub command_history: Vec<String>,
    pub room_history: Vec<Zone>,
    pub room_transitions: Vec<(Zone, Zone)>,
    pub assembly_complete: bool,
    pub assembly_completed: bool,
    pub clearance_verified: bool,
    pub kinetics_verified: bool,

    // Gantry state accessed by tests
    pub gantry: GantryState,

    // State vectors for the 8 parts (backwards compatibility)
    pub state_vectors: [PartStateVector; 8],

    // Walkthrough receipt (updated dynamically)
    pub walkthrough_receipt: Option<WalkthroughReceipt>,

    // Digital Twin States (backwards compatibility)
    pub twin_parts: HashMap<String, TwinPart>,
    pub twin_sockets: HashMap<String, TwinSocket>,
    pub twin_joints: HashMap<String, TwinJoint>,
    pub overall_degradation_state: String,
    pub collision_incidents: u32,

    // Validation Pipeline State
    pub validation_pipeline: Option<ValidationPipeline>,

    // 8-Part assembly receipt mapping (backwards compatibility)
    pub assembly_receipt_legacy: Option<nexus_tps::MechTpsReceipt>,

    // The new 9-part MUD model
    pub parts: HashMap<String, MechPart>,
    pub joints: HashMap<String, Joint>,
    pub gates: GateStatus,
    pub event_log: Vec<MudEvent>,
    pub diagnostics: HashMap<String, String>,
    pub assembly_receipt: Option<AssemblyReceipt>,
}

impl Default for MudEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MudEngine {
    /// Creates a new MudEngine with healthy/compliant default state.
    pub fn new() -> Self {
        let state_vectors = [
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::Head,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::Torso,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::Waist,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::ArmL,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::ArmR,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::LegL,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::LegR,
            },
            PartStateVector {
                civilization_id: 0,
                frame_id: 1,
                armor_profile: 0.5,
                joint_profile: 0.5,
                mass_profile: 0.5,
                weapon_profile: 0.5,
                motion_profile: 50.0,
                part_slot: PartSlot::Backpack,
            },
        ];

        let mut twin_parts = HashMap::new();
        let part_names = vec![
            "Head", "Torso", "Waist", "ArmL", "ArmR", "LegL", "LegR", "Backpack",
        ];
        for name in part_names {
            twin_parts.insert(
                name.to_string(),
                TwinPart {
                    name: name.to_string(),
                    geometry: 500,
                    socket_fit: 0xFF,
                    motion_clearance: 1,
                    collision_volume: 1,
                    mass_balance: 100,
                    physics_role: 1,
                    assembly_compatibility: 0xFFFF,
                    health_score: 1.0,
                    wear_rate: 0.0,
                    thermal_load: 35.0,
                    fault_status: "None".to_string(),
                },
            );
        }

        let mut twin_sockets = HashMap::new();
        let socket_names = vec!["head", "torso", "left_arm", "right_arm", "legs"];
        for name in socket_names {
            twin_sockets.insert(
                name.to_string(),
                TwinSocket {
                    name: name.to_string(),
                    mated: false,
                    stress_level: 0.05,
                    mated_to: None,
                    mating_rule: format!("{} socket bitwise check", name),
                },
            );
        }

        let mut twin_joints = HashMap::new();
        let joint_names = vec![
            "Neck",
            "ShoulderL",
            "ShoulderR",
            "HipL",
            "HipR",
            "AnkleL",
            "AnkleR",
        ];
        for name in joint_names {
            twin_joints.insert(
                name.to_string(),
                TwinJoint {
                    name: name.to_string(),
                    wear_level: 0.0,
                    motion_failures: 0,
                    status: "Operational".to_string(),
                },
            );
        }

        let mut parts = HashMap::new();
        let default_parts = vec![
            (
                "torso_frame",
                "frame",
                100.0,
                AABB {
                    min_x: -0.9,
                    max_x: 0.9,
                    min_y: -0.9,
                    max_y: 0.9,
                    min_z: -0.9,
                    max_z: 0.9,
                },
                vec![],
                vec![
                    "head",
                    "left_arm",
                    "right_arm",
                    "left_leg",
                    "right_leg",
                    "backpack",
                ],
            ),
            (
                "head",
                "head",
                10.0,
                AABB {
                    min_x: -0.3,
                    max_x: 0.3,
                    min_y: -0.3,
                    max_y: 0.3,
                    min_z: 1.0,
                    max_z: 1.6,
                },
                vec!["head"],
                vec![],
            ),
            (
                "left_arm",
                "arm",
                20.0,
                AABB {
                    min_x: -2.0,
                    max_x: -1.0,
                    min_y: -0.4,
                    max_y: 0.4,
                    min_z: -0.5,
                    max_z: 0.5,
                },
                vec!["left_arm"],
                vec![],
            ),
            (
                "right_arm",
                "arm",
                20.0,
                AABB {
                    min_x: 1.0,
                    max_x: 2.0,
                    min_y: -0.4,
                    max_y: 0.4,
                    min_z: -0.5,
                    max_z: 0.5,
                },
                vec!["right_arm"],
                vec![],
            ),
            (
                "left_leg",
                "leg",
                30.0,
                AABB {
                    min_x: -0.8,
                    max_x: -0.2,
                    min_y: -0.5,
                    max_y: 0.5,
                    min_z: -2.5,
                    max_z: -1.0,
                },
                vec!["left_leg"],
                vec![],
            ),
            (
                "right_leg",
                "leg",
                30.0,
                AABB {
                    min_x: 0.2,
                    max_x: 0.8,
                    min_y: -0.5,
                    max_y: 0.5,
                    min_z: -2.5,
                    max_z: -1.0,
                },
                vec!["right_leg"],
                vec![],
            ),
            (
                "backpack",
                "backpack",
                15.0,
                AABB {
                    min_x: -0.5,
                    max_x: 0.5,
                    min_y: -2.0,
                    max_y: -1.0,
                    min_z: -0.5,
                    max_z: 0.5,
                },
                vec!["backpack"],
                vec!["left_thruster", "right_thruster"],
            ),
            (
                "left_thruster",
                "thruster",
                8.0,
                AABB {
                    min_x: -0.8,
                    max_x: -0.4,
                    min_y: -3.5,
                    max_y: -2.5,
                    min_z: -0.8,
                    max_z: 0.4,
                },
                vec!["left_thruster"],
                vec![],
            ),
            (
                "right_thruster",
                "thruster",
                8.0,
                AABB {
                    min_x: 0.4,
                    max_x: 0.8,
                    min_y: -3.5,
                    max_y: -2.5,
                    min_z: -0.8,
                    max_z: 0.4,
                },
                vec!["right_thruster"],
                vec![],
            ),
        ];

        for (id, kind, mass, bounds, req, prov) in default_parts {
            parts.insert(
                id.to_string(),
                MechPart {
                    part_id: id.to_string(),
                    part_kind: kind.to_string(),
                    mass,
                    bounds,
                    sockets_required: req
                        .into_iter()
                        .map(|s| SocketConnection {
                            name: s.to_string(),
                            kind: s.to_string(),
                        })
                        .collect(),
                    sockets_provided: prov
                        .into_iter()
                        .map(|s| SocketConnection {
                            name: s.to_string(),
                            kind: s.to_string(),
                        })
                        .collect(),
                    health_status: 1.0,
                    admission_status: "pending".to_string(),
                },
            );
        }

        let mut joints = HashMap::new();
        let default_joints = vec![
            ("Neck", "torso_frame", "head", Some([-45.0, 45.0])),
            ("ShoulderL", "torso_frame", "left_arm", Some([-90.0, 90.0])),
            ("ShoulderR", "torso_frame", "right_arm", Some([-90.0, 90.0])),
            ("HipL", "torso_frame", "left_leg", Some([-60.0, 60.0])),
            ("HipR", "torso_frame", "right_leg", Some([-60.0, 60.0])),
            ("BackpackMount", "torso_frame", "backpack", Some([0.0, 0.0])),
            (
                "ThrusterL",
                "backpack",
                "left_thruster",
                Some([-30.0, 30.0]),
            ),
            (
                "ThrusterR",
                "backpack",
                "right_thruster",
                Some([-30.0, 30.0]),
            ),
        ];

        for (name, parent, child, lims) in default_joints {
            joints.insert(
                name.to_string(),
                Joint {
                    name: name.to_string(),
                    parent_part: parent.to_string(),
                    child_part: child.to_string(),
                    rotation_limits: lims,
                },
            );
        }

        let mut engine = MudEngine {
            current_zone: Zone::MissionRoom,
            command_history: Vec::new(),
            room_history: vec![Zone::MissionRoom],
            room_transitions: Vec::new(),
            assembly_complete: false,
            assembly_completed: false,
            clearance_verified: false,
            kinetics_verified: false,
            gantry: GantryState {
                verification_status: "NOMINAL".to_string(),
                current_state: "Idle".to_string(),
            },
            state_vectors,
            walkthrough_receipt: None,
            twin_parts,
            twin_sockets,
            twin_joints,
            overall_degradation_state: "Healthy".to_string(),
            collision_incidents: 0,
            validation_pipeline: None,
            assembly_receipt_legacy: None,
            parts,
            joints,
            gates: GateStatus::default(),
            event_log: Vec::new(),
            diagnostics: HashMap::new(),
            assembly_receipt: None,
        };

        engine.emit_event(
            "factory.entered",
            "mission_room",
            "Agent entered the Gundam Manufacturing Facility MUD strategic space.",
        );
        engine
    }

    /// Creates a MudEngine with a specific injected fault.
    pub fn new_with_fault(fault_type: &str) -> Self {
        let mut engine = Self::new();
        engine
            .diagnostics
            .insert("fault_type".to_string(), fault_type.to_string());

        if fault_type == "socket" {
            // Mismatched socket: head and torso socket mismatch
            if let Some(head) = engine.parts.get_mut("head") {
                head.sockets_required = vec![SocketConnection {
                    name: "head_mismatched".to_string(),
                    kind: "head_mismatched".to_string(),
                }];
                head.health_status = 0.4;
            }
            if let Some(head) = engine.twin_parts.get_mut("Head") {
                head.socket_fit = 0xF0; // torso ends in 0x0F, mismatch!
                head.fault_status = "Socket Mating Defect".to_string();
                head.health_score = 0.4;
            }
            if let Some(socket) = engine.twin_sockets.get_mut("head") {
                socket.stress_level = 0.95;
            }
            engine.overall_degradation_state = "FailDegraded".to_string();
        } else if fault_type == "collision" {
            // Induce a collision overlap
            if let Some(head) = engine.parts.get_mut("head") {
                head.bounds.min_z = 0.5;
                head.bounds.max_z = 0.9; // Torso max_z is 0.9, so they overlap!
                head.health_status = 0.3;
            }
            if let Some(head) = engine.twin_parts.get_mut("Head") {
                head.collision_volume = 200; // Overlaps torso
                head.fault_status = "Collision Sweep Defect".to_string();
                head.health_score = 0.3;
            }
            engine.collision_incidents = 1;
            engine.overall_degradation_state = "FailSafe".to_string();
        } else if fault_type == "overload" {
            // Induce total mass exceeding leg capacity
            if let Some(torso) = engine.parts.get_mut("torso_frame") {
                torso.mass = 500.0;
            }
            if let Some(legs) = engine.twin_parts.get_mut("LegL") {
                legs.geometry = 5;
                legs.fault_status = "Load Overcapacity Defect".to_string();
                legs.health_score = 0.2;
            }
            engine.overall_degradation_state = "FailSafe".to_string();
        } else if fault_type == "com" {
            // Induce center of mass deviation (unbalanced arms)
            if let Some(left_arm) = engine.parts.get_mut("left_arm") {
                left_arm.mass = 200.0;
            }
            if let Some(arm_l) = engine.twin_parts.get_mut("ArmL") {
                arm_l.mass_balance = 300;
                arm_l.fault_status = "COM Disbalance Defect".to_string();
                arm_l.health_score = 0.5;
            }
            engine.overall_degradation_state = "FailDegraded".to_string();
        } else if fault_type == "joint" {
            // Induce Neck joint rotation limits to None
            if let Some(joint) = engine.joints.get_mut("Neck") {
                joint.rotation_limits = None;
            }
            if let Some(joint) = engine.twin_joints.get_mut("HipL") {
                joint.wear_level = 0.85;
                joint.motion_failures = 3;
                joint.status = "Degraded".to_string();
            }
            engine.overall_degradation_state = "FailOperational".to_string();
        }
        engine
    }

    /// Emits a unique, strictly monotonic event with referential integrity.
    pub fn emit_event(&mut self, event_type: &str, object_id: &str, details: &str) {
        let now = Utc::now();
        let mut timestamp = now;
        if let Some(last_evt) = self.event_log.last() {
            if timestamp <= last_evt.timestamp {
                timestamp = last_evt.timestamp + chrono::Duration::milliseconds(1);
            }
        }

        let event_id = format!("evt_{}", self.event_log.len() + 1);
        let event = MudEvent {
            event_id,
            event_type: event_type.to_string(),
            timestamp,
            object_id: object_id.to_string(),
            details: details.to_string(),
        };
        self.event_log.push(event);
    }

    /// Performs referential integrity check on the event log.
    pub fn verify_referential_integrity(&self) -> Result<(), String> {
        let valid_zones = vec![
            "mission_room",
            "materials_lab",
            "primitive_foundry",
            "runner_wall",
            "assembly_gantry",
            "fit_bay",
            "collision_bay",
            "proving_ground",
            "reveal_platform",
        ];

        let valid_parts = vec![
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

        let valid_sockets = [
            "head_socket",
            "left_arm_socket",
            "right_arm_socket",
            "left_leg_socket",
            "right_leg_socket",
            "backpack_socket",
            "left_thruster_socket",
            "right_thruster_socket",
        ];

        let valid_receipts = ["assembly_receipt", "walkthrough_receipt"];

        let allowed_meta = vec![
            "inventory",
            "exits",
            "preview",
            "all",
            "mech",
            "assembly",
            "overall",
            "mission_room",
            "materials_lab",
            "primitive_foundry",
            "runner_wall",
            "assembly_gantry",
            "fit_bay",
            "collision_bay",
            "proving_ground",
            "reveal_platform",
            "socket",
            "head",
            "torso",
            "left_arm",
            "right_arm",
            "left_leg",
            "right_leg",
            "backpack",
            "left_thruster",
            "right_thruster",
            "mission",
            "materials",
            "primitive",
            "runner",
            "fit",
            "collision",
            "motion",
            "kinetics",
            "reveal",
            "standard",
            "walkthrough",
            "command",
        ];

        for event in &self.event_log {
            let id = event.object_id.to_lowercase();
            let matches_zone = valid_zones.contains(&id.as_str());
            let matches_part = valid_parts.contains(&id.as_str());
            let matches_socket = valid_sockets.contains(&id.as_str());
            let matches_receipt = valid_receipts.contains(&id.as_str());
            let matches_meta = allowed_meta.contains(&id.as_str());

            if !matches_zone
                && !matches_part
                && !matches_socket
                && !matches_receipt
                && !matches_meta
            {
                return Err(format!(
                    "Referential integrity failure: event ID {} refers to invalid object_id '{}'",
                    event.event_id, event.object_id
                ));
            }
        }
        Ok(())
    }

    /// Parses a raw text command into a typed Command enum.
    pub fn parse_command(input: &str) -> Result<Command, MudError> {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();

        if lower == "look" {
            Ok(Command::Look)
        } else if lower == "watch assembly" || lower == "assemble standard" {
            Ok(Command::WatchAssembly)
        } else if lower == "verify clearance" || lower == "verify collision" {
            Ok(Command::VerifyClearance)
        } else if lower == "verify kinetics" || lower == "verify motion" {
            Ok(Command::VerifyKinetics)
        } else if lower.starts_with("go ") {
            let dest = trimmed[3..].trim().to_string();
            if dest.is_empty() {
                Err(MudError::InvalidCommand(
                    "Go command requires a direction or zone name".into(),
                ))
            } else {
                Ok(Command::Go(dest))
            }
        } else if lower.starts_with("inspect ") {
            let target = trimmed[8..].trim().to_string();
            if target.is_empty() {
                Err(MudError::InvalidCommand(
                    "Inspect command requires a target".into(),
                ))
            } else {
                Ok(Command::Inspect(target))
            }
        } else if lower.starts_with("stress part ") {
            let part_name = trimmed[12..].trim().to_string();
            if part_name.is_empty() {
                Err(MudError::InvalidCommand(
                    "Stress part command requires a component name".into(),
                ))
            } else {
                Ok(Command::StressPart(part_name))
            }
        } else {
            Err(MudError::InvalidCommand(format!(
                "Unknown command: '{}'",
                trimmed
            )))
        }
    }

    /// Parses a raw string command and executes it, updating state and returning response string.
    pub fn execute_command(&mut self, input: &str) -> Result<String, MudError> {
        self.run_mape_k_loop();
        let cmd_parsed = parse_command(input)?;

        self.command_history.push(input.trim().to_string());

        let object_id = match &cmd_parsed {
            MudCommand::Look => self.current_zone.to_str(),
            MudCommand::Go(dest) => dest.as_str(),
            MudCommand::Inspect(target) => target.as_str(),
            MudCommand::Health(target) => target.as_str(),
            MudCommand::Diagnose(target) => {
                if target.is_empty() {
                    "all"
                } else {
                    target.as_str()
                }
            }
            MudCommand::Verify(gate) => gate.as_str(),
            MudCommand::Assemble(spec) => spec.as_str(),
            MudCommand::Preview(op) => op.as_str(),
            MudCommand::Receipt(target) => target.as_str(),
            MudCommand::Inventory => "inventory",
            MudCommand::Exits => "exits",
        };
        self.emit_event(
            "object.inspected",
            object_id,
            &format!("Executed command: {}", input),
        );

        let result = match cmd_parsed {
            MudCommand::Look => self.cmd_look(),
            MudCommand::Go(dest) => self.cmd_go(&dest),
            MudCommand::Inspect(target) => self.cmd_inspect(&target),
            MudCommand::Health(target) => self.cmd_health(&target),
            MudCommand::Diagnose(target) => self.cmd_diagnose(&target),
            MudCommand::Verify(gate) => self.cmd_verify(&gate),
            MudCommand::Assemble(spec) => self.cmd_assemble(&spec),
            MudCommand::Preview(op) => self.cmd_preview(&op),
            MudCommand::Receipt(target) => self.cmd_receipt(&target),
            MudCommand::Inventory => self.cmd_inventory(),
            MudCommand::Exits => self.cmd_exits(),
        };

        if self.current_zone == Zone::RevealPlatform
            && self.gates.motion
            && self.walkthrough_receipt.is_none()
        {
            let _ = self.cmd_verify("reveal");
        }
        result
    }

    /// Executes a parsed command against the engine state, returning the resulting text output.
    pub fn execute(&mut self, cmd: Command) -> Result<String, MudError> {
        let cmd_str = match &cmd {
            Command::Go(d) => format!("go {}", d),
            Command::Look => "look".to_string(),
            Command::Inspect(t) => format!("inspect {}", t),
            Command::WatchAssembly => "assemble standard".to_string(),
            Command::VerifyClearance => "verify collision".to_string(),
            Command::VerifyKinetics => "verify motion".to_string(),
            Command::StressPart(p) => format!("stress part {}", p),
        };
        self.command_history.push(cmd_str.clone());

        let object_id = match &cmd {
            Command::Look => self.current_zone.to_str(),
            Command::Go(d) => d.as_str(),
            Command::Inspect(t) => t.as_str(),
            Command::WatchAssembly => "standard",
            Command::VerifyClearance => "collision",
            Command::VerifyKinetics => "motion",
            Command::StressPart(p) => p.as_str(),
        };
        self.emit_event(
            "object.inspected",
            object_id,
            &format!("Executed legacy command: {}", cmd_str),
        );

        match cmd {
            Command::Look => self.cmd_look(),
            Command::Go(dest) => self.cmd_go(&dest),
            Command::Inspect(target) => self.cmd_inspect(&target),
            Command::WatchAssembly => self.cmd_assemble("standard"),
            Command::VerifyClearance => {
                if !self.gates.fit {
                    let _ = self.cmd_verify("fit")?;
                }
                self.cmd_verify("collision")
            }
            Command::VerifyKinetics => self.cmd_verify("motion"),
            Command::StressPart(part) => self.cmd_stress_part(&part),
        }
    }

    fn cmd_look(&self) -> Result<String, MudError> {
        let mut exits = Vec::new();
        match self.current_zone {
            Zone::MissionRoom => exits.push("east (materials_lab)"),
            Zone::MaterialsLab => {
                exits.push("west (mission_room)");
                exits.push("east (primitive_foundry)");
            }
            Zone::PrimitiveFoundry => {
                exits.push("west (materials_lab)");
                exits.push("east (runner_wall)");
            }
            Zone::RunnerWall => {
                exits.push("west (primitive_foundry)");
                exits.push("east (assembly_gantry)");
            }
            Zone::AssemblyGantry => {
                exits.push("west (runner_wall)");
                exits.push("east (fit_bay)");
            }
            Zone::FitBay => {
                exits.push("west (assembly_gantry)");
                exits.push("east (collision_bay)");
            }
            Zone::CollisionBay => {
                exits.push("west (fit_bay)");
                exits.push("east (proving_ground)");
            }
            Zone::ProvingGround => {
                exits.push("west (collision_bay)");
                exits.push("east (reveal_platform)");
            }
            Zone::RevealPlatform => {
                exits.push("west (proving_ground)");
            }
        }

        Ok(format!(
            "Room: {}\nDescription: {}\nElevation: {}\nExits: {}",
            self.current_zone.name(),
            self.current_zone.description(),
            self.current_zone.elevation(),
            exits.join(", ")
        ))
    }

    fn cmd_go(&mut self, dest: &str) -> Result<String, MudError> {
        let target_zone = if let Some(z) = Zone::parse(dest) {
            z
        } else {
            let clean = dest.trim().to_lowercase();
            match (self.current_zone, clean.as_str()) {
                (Zone::MissionRoom, "east") => Zone::MaterialsLab,
                (Zone::MaterialsLab, "west") => Zone::MissionRoom,
                (Zone::MaterialsLab, "east") => Zone::PrimitiveFoundry,
                (Zone::PrimitiveFoundry, "west") => Zone::MaterialsLab,
                (Zone::PrimitiveFoundry, "east") => Zone::RunnerWall,
                (Zone::RunnerWall, "west") => Zone::PrimitiveFoundry,
                (Zone::RunnerWall, "east") => Zone::AssemblyGantry,
                (Zone::AssemblyGantry, "west") => Zone::RunnerWall,
                (Zone::AssemblyGantry, "east") => Zone::FitBay,
                (Zone::FitBay, "west") => Zone::AssemblyGantry,
                (Zone::FitBay, "east") => Zone::CollisionBay,
                (Zone::CollisionBay, "west") => Zone::FitBay,
                (Zone::CollisionBay, "east") => Zone::ProvingGround,
                (Zone::ProvingGround, "west") => Zone::CollisionBay,
                (Zone::ProvingGround, "east") => Zone::RevealPlatform,
                (Zone::RevealPlatform, "west") => Zone::ProvingGround,
                _ => {
                    return Err(MudError::InvalidTransition {
                        direction: dest.to_string(),
                        current_zone: self.current_zone.name().to_string(),
                    });
                }
            }
        };

        let from = self.current_zone;
        let to = target_zone;

        let diff = (to.elevation() - from.elevation()).abs();
        if diff > 1 {
            return Err(MudError::InvalidDirectTravel {
                destination: to.name().to_string(),
                current_zone: from.name().to_string(),
            });
        }

        if to.elevation() > from.elevation() {
            let allowed = match from {
                Zone::MissionRoom => self.gates.mission,
                Zone::MaterialsLab => self.gates.materials,
                Zone::PrimitiveFoundry => self.gates.primitive,
                Zone::RunnerWall => self.gates.runner_wall,
                Zone::AssemblyGantry => self.gates.assembly,
                Zone::FitBay => self.gates.fit,
                Zone::CollisionBay => self.gates.collision,
                Zone::ProvingGround => self.gates.motion,
                Zone::RevealPlatform => self.gates.reveal,
            };
            if !allowed {
                return Err(MudError::GateBlocked(format!(
                    "Admittance refused: gate check for {} must pass before advancing.",
                    from.name()
                )));
            }
        }

        self.room_transitions.push((from, to));
        self.room_history.push(to);
        self.current_zone = to;

        self.emit_event(
            "zone.entered",
            to.to_str(),
            &format!("Player walked into {}", to.name()),
        );

        Ok(format!("Moved to room: {}", to.name()))
    }

    fn cmd_inspect(&mut self, target: &str) -> Result<String, MudError> {
        let clean = target.trim().to_lowercase();
        self.emit_event("object.inspected", target, &format!("Inspected {}", target));

        if let Some(part) = self.parts.get(&clean) {
            return Ok(format!(
                "Part: {}\nKind: {}\nMass: {} kg\nBounds: min_x={}, max_x={}, min_y={}, max_y={}, min_z={}, max_z={}\nSockets Required: {}\nSockets Provided: {}\nHealth: {:.2}\nAdmission: {}",
                part.part_id,
                part.part_kind,
                part.mass,
                part.bounds.min_x, part.bounds.max_x,
                part.bounds.min_y, part.bounds.max_y,
                part.bounds.min_z, part.bounds.max_z,
                part.sockets_required.iter().map(|s| &s.kind).cloned().collect::<Vec<String>>().join(", "),
                part.sockets_provided.iter().map(|s| &s.kind).cloned().collect::<Vec<String>>().join(", "),
                part.health_status,
                part.admission_status
            ));
        }

        if let Some(zone) = Zone::parse(target) {
            return Ok(format!(
                "Zone: {}\nElevation: {}\nDescription: {}",
                zone.name(),
                zone.elevation(),
                zone.description()
            ));
        }

        for (name, part) in &self.twin_parts {
            if name.to_lowercase() == clean {
                return Ok(format!(
                    "Component: {}\nGeometry: {}\nSocket Fit: 0x{:X}\nCollision Volume: {}\nMass Balance: {}\nHealth Score: {:.2}\nRemaining Useful Life: {:.1} hours\nFault Status: {}",
                    part.name,
                    part.geometry,
                    part.socket_fit,
                    part.collision_volume,
                    part.mass_balance,
                    part.health_score,
                    part.health_score * 800.0,
                    part.fault_status
                ));
            }
        }

        if clean == "runner_wall" || clean == "runner wall" {
            return self.cmd_inventory();
        }

        if clean == "assembly" {
            if !self.assembly_complete {
                return Ok("Mech assembly has not been performed yet.".into());
            }
            let mut report = vec![
                "=== CURRENT CONSTRUCTED ASSEMBLY ===".to_string(),
                "Attached Components:".to_string(),
            ];
            let parts_order = vec![
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
            for p in parts_order {
                if let Some(part) = self.parts.get(p) {
                    report.push(format!("  - {} (Health: {:.2})", p, part.health_status));
                }
            }
            return Ok(report.join("\n"));
        }

        if clean == "motion-domain" || clean == "motion_domain" {
            let mut report = vec![
                "=== MOTION KINEMATICS DOMAIN REPORT ===".to_string(),
                format!(
                    "Overall Joint Status: {}",
                    if self.overall_degradation_state == "FailSafe" {
                        "Locked/Kneeling"
                    } else {
                        "Active"
                    }
                ),
                "Joint Wear levels:".to_string(),
            ];
            for (name, joint) in &self.twin_joints {
                report.push(format!(
                    "  - Joint {}: Wear: {:.2}, Failures: {}, Status: {}",
                    name, joint.wear_level, joint.motion_failures, joint.status
                ));
            }
            report.push(format!(
                "Active Degradation Profile: {}",
                self.overall_degradation_state
            ));
            if self.overall_degradation_state == "FailSafe" {
                report.push(
                    "  -> FAIL-SAFE MODE: Joints locked, weapons deactivated, kneeling down."
                        .into(),
                );
            } else if self.overall_degradation_state == "FailDegraded" {
                report.push(
                    "  -> FAIL-DEGRADED MODE: Walking speed limited, thrusters reduced to 50%."
                        .into(),
                );
            } else {
                report.push(
                    "  -> FAIL-OPERATIONAL MODE: Auxiliary pathways routing, 100% mission cap."
                        .into(),
                );
            }
            return Ok(report.join("\n"));
        }

        Err(MudError::InvalidInspectTarget(format!(
            "Cannot inspect '{}'. Target not found.",
            target
        )))
    }

    fn cmd_health(&mut self, target: &str) -> Result<String, MudError> {
        let clean = target.trim().to_lowercase();
        self.emit_event(
            "object.inspected",
            target,
            &format!("Checked health for {}", target),
        );
        if clean == "mech" || clean == "assembly" || clean == "overall" {
            let phm = self.get_phm_metrics();
            return Ok(format!("Overall health status: {:.2}", phm.overall_health));
        }
        if let Some(part) = self.parts.get(&clean) {
            return Ok(format!(
                "Part: {}, Health Score: {:.2}",
                part.part_id, part.health_status
            ));
        }
        for (name, part) in &self.twin_parts {
            if name.to_lowercase() == clean {
                return Ok(format!(
                    "Part: {}, Health Score: {:.2}",
                    name, part.health_score
                ));
            }
        }
        Err(MudError::InvalidInspectTarget(format!(
            "Unknown health target '{}'",
            target
        )))
    }

    fn cmd_diagnose(&mut self, target: &str) -> Result<String, MudError> {
        let clean = target.trim().to_lowercase();
        self.emit_event("object.inspected", target, &format!("Diagnosed {}", target));

        let mut reports = Vec::new();
        if clean.is_empty() || clean == "all" || clean == "mech" || clean == "assembly" {
            for (k, v) in &self.diagnostics {
                reports.push(format!("{}: {}", k, v));
            }
        } else {
            if let Some(v) = self.diagnostics.get(&clean) {
                reports.push(format!("{}: {}", clean, v));
            } else if let Some(part) = self.parts.get(&clean) {
                reports.push(format!(
                    "Part {} is healthy with score {:.2}.",
                    part.part_id, part.health_status
                ));
            } else {
                for (name, part) in &self.twin_parts {
                    if name.to_lowercase() == clean {
                        reports.push(format!("Part {} status: {}", name, part.fault_status));
                    }
                }
            }
        }

        if reports.is_empty() {
            Ok("System diagnostics nominal. No faults detected.".to_string())
        } else {
            Ok(reports.join("\n"))
        }
    }

    /// Dispatch verify gate commands to dedicated sub-verifiers.
    fn cmd_verify(&mut self, gate_name: &str) -> Result<String, MudError> {
        let clean = gate_name.trim().to_lowercase();
        match clean.as_str() {
            "mission" => {
                if self.current_zone != Zone::MissionRoom {
                    return Err(MudError::OperationError(
                        "Mission gate can only be verified in Mission Room".into(),
                    ));
                }
                self.gates.mission = true;
                self.emit_event("fit.checked", "mission_room", "Mission gate verified.");
                Ok("Mission gate compliance check passed.".to_string())
            }
            "materials" => {
                if self.current_zone != Zone::MaterialsLab {
                    return Err(MudError::OperationError(
                        "Materials gate can only be verified in Materials Lab".into(),
                    ));
                }
                self.gates.materials = true;
                self.emit_event("fit.checked", "materials_lab", "Materials gate verified.");
                Ok("Materials gate compliance check passed.".to_string())
            }
            "primitive" => {
                if self.current_zone != Zone::PrimitiveFoundry {
                    return Err(MudError::OperationError(
                        "Primitive gate can only be verified in Primitive Foundry".into(),
                    ));
                }
                self.gates.primitive = true;
                self.emit_event(
                    "fit.checked",
                    "primitive_foundry",
                    "Primitive gate verified.",
                );
                Ok("Primitive gate compliance check passed.".to_string())
            }
            "runner_wall" | "runner" => {
                if self.current_zone != Zone::RunnerWall {
                    return Err(MudError::OperationError(
                        "Runner Wall gate can only be verified in Runner Wall".into(),
                    ));
                }
                self.gates.runner_wall = true;
                self.emit_event("fit.checked", "runner_wall", "Runner Wall gate verified.");
                Ok("Runner Wall gate compliance check passed.".to_string())
            }
            "assembly" => {
                if self.current_zone != Zone::AssemblyGantry {
                    return Err(MudError::OperationError(
                        "Assembly gate can only be verified in Assembly Gantry".into(),
                    ));
                }
                if !self.assembly_complete {
                    return Err(MudError::OperationError(
                        "You must assemble the mech first using 'assemble standard'".into(),
                    ));
                }
                self.gates.assembly = true;
                self.emit_event("fit.checked", "assembly_gantry", "Assembly gate verified.");
                Ok("Assembly gate compliance check passed.".to_string())
            }
            "fit" => self.cmd_verify_fit(),
            "collision" => self.cmd_verify_collision(),
            "motion" | "kinetics" => self.cmd_verify_kinetics(),
            "reveal" => self.cmd_verify_reveal(),
            _ => Err(MudError::InvalidCommand(format!(
                "Unknown gate: {}",
                gate_name
            ))),
        }
    }

    /// Verify socket mating and mass capacity in the Fit Bay.
    fn cmd_verify_fit(&mut self) -> Result<String, MudError> {
        if self.current_zone != Zone::FitBay {
            return Err(MudError::OperationError(
                "Fit gate can only be verified in Fit Bay".into(),
            ));
        }
        if !self.assembly_complete {
            return Err(MudError::OperationError(
                "No active assembly found. Complete assembly first.".into(),
            ));
        }

        let mut mismatch = false;
        let torso = self.parts.get("torso_frame").unwrap();
        let head = self.parts.get("head").unwrap();

        let head_req = head.sockets_required.first().map(|s| &s.kind);
        let torso_prov_head = torso.sockets_provided.iter().find(|s| s.kind == "head");

        if let (Some(req), Some(_prov)) = (head_req, torso_prov_head) {
            if req != "head" {
                mismatch = true;
            }
        } else {
            mismatch = true;
        }

        if mismatch {
            self.emit_event(
                "socket.mismatched",
                "head",
                "Socket mating failure at Neck.",
            );
            self.diagnostics.insert(
                "fit_fail".to_string(),
                "Neck socket mating mismatch: torso expected 'head', got 'head_mismatched'"
                    .to_string(),
            );
            return Err(MudError::GateBlocked(
                "Fit check failed: head and torso socket mating mismatch.".into(),
            ));
        }

        let total_mass: f32 = self.parts.values().map(|p| p.mass).sum();
        let leg_capacity = 250.0;
        if total_mass > leg_capacity {
            self.diagnostics.insert(
                "fit_fail".to_string(),
                format!(
                    "Total mass {} exceeds leg capacity {}",
                    total_mass, leg_capacity
                ),
            );
            return Err(MudError::GateBlocked(format!(
                "Fit check failed: total mass {} exceeds leg capacity {}.",
                total_mass, leg_capacity
            )));
        }

        self.gates.fit = true;
        self.emit_event("fit.checked", "fit_bay", "Fit gate verified successfully.");
        Ok("Fit gate compliance check passed.".to_string())
    }

    /// Sweep all part pairs for AABB intersection and re-verify mass in the Collision Bay.
    fn cmd_verify_collision(&mut self) -> Result<String, MudError> {
        if self.current_zone != Zone::CollisionBay {
            return Err(MudError::OperationError(
                "Collision gate can only be verified in Collision Bay".into(),
            ));
        }
        if !self.gates.fit {
            return Err(MudError::GateBlocked(
                "Fit gate must be verified before Collision gate".into(),
            ));
        }

        let mut intersection_detected = false;
        let mut intersecting_parts = (String::new(), String::new());

        let part_keys: Vec<String> = self.parts.keys().cloned().collect();
        'outer: for i in 0..part_keys.len() {
            for j in (i + 1)..part_keys.len() {
                let part_a = self.parts.get(&part_keys[i]).unwrap();
                let part_b = self.parts.get(&part_keys[j]).unwrap();
                if part_a.bounds.intersects(&part_b.bounds) {
                    intersection_detected = true;
                    intersecting_parts = (part_a.part_id.clone(), part_b.part_id.clone());
                    break 'outer;
                }
            }
        }

        if intersection_detected {
            self.emit_event(
                "collision.checked",
                &intersecting_parts.0,
                &format!(
                    "Collision sweep detected intersection between {} and {}",
                    intersecting_parts.0, intersecting_parts.1
                ),
            );
            self.diagnostics.insert(
                "collision_fail".to_string(),
                format!(
                    "Collision sweep defect: {} and {} bounds overlap.",
                    intersecting_parts.0, intersecting_parts.1
                ),
            );
            return Err(MudError::GateBlocked(format!(
                "Collision check failed: {} intersects with {}.",
                intersecting_parts.0, intersecting_parts.1
            )));
        }

        let total_mass: f32 = self.parts.values().map(|p| p.mass).sum();
        let leg_capacity = 250.0;
        if total_mass > leg_capacity {
            self.diagnostics.insert(
                "collision_fail".to_string(),
                format!(
                    "Total mass {} exceeds leg capacity {}",
                    total_mass, leg_capacity
                ),
            );
            return Err(MudError::GateBlocked(format!(
                "Collision check failed: total mass {} exceeds leg capacity {}.",
                total_mass, leg_capacity
            )));
        }

        self.gates.collision = true;
        self.clearance_verified = true;
        self.emit_event(
            "collision.checked",
            "collision_bay",
            "Collision sweep passed cleanly.",
        );
        Ok("Collision gate compliance check passed.".to_string())
    }

    /// Verify joint rotation limits and center-of-mass balance in the Proving Ground.
    fn cmd_verify_kinetics(&mut self) -> Result<String, MudError> {
        if self.current_zone != Zone::ProvingGround {
            return Err(MudError::OperationError(
                "Motion gate can only be verified in Proving Ground".into(),
            ));
        }
        if !self.gates.collision {
            return Err(MudError::GateBlocked(
                "Collision gate must be verified before Motion gate".into(),
            ));
        }

        self.emit_event(
            "motion.sweep_started",
            "proving_ground",
            "Kinetic motion sweep started for 4 poses: stand, step, turn, kneel.",
        );

        for joint in self.joints.values() {
            if joint.rotation_limits.is_none() {
                self.diagnostics.insert(
                    "motion_fail".to_string(),
                    format!(
                        "Joint '{}' rotation limits are unbounded (None).",
                        joint.name
                    ),
                );
                return Err(MudError::GateBlocked(format!(
                    "Motion sweep check failed: Joint '{}' limits are unbounded.",
                    joint.name
                )));
            }
        }

        let left_arm = self.parts.get("left_arm").unwrap();
        let right_arm = self.parts.get("right_arm").unwrap();
        let com_dev = (left_arm.mass - right_arm.mass).abs();
        if com_dev > 10.0 {
            self.diagnostics.insert(
                "motion_fail".to_string(),
                format!(
                    "Center of Mass deviation ({:.1}) exceeds threshold (10.0)",
                    com_dev
                ),
            );
            return Err(MudError::GateBlocked(format!(
                "Motion sweep check failed: COM deviation ({:.1}) too high.",
                com_dev
            )));
        }

        self.gates.motion = true;
        self.kinetics_verified = true;
        self.emit_event(
            "motion.sweep_passed",
            "proving_ground",
            "Kinetic motion sweep passed for all 4 poses.",
        );
        Ok("Motion gate compliance check passed.".to_string())
    }

    /// Generate assembly receipt and walkthrough receipt on the Reveal Platform.
    fn cmd_verify_reveal(&mut self) -> Result<String, MudError> {
        if self.current_zone != Zone::RevealPlatform {
            return Err(MudError::OperationError(
                "Reveal gate can only be verified in Reveal Platform".into(),
            ));
        }
        if !self.gates.motion {
            return Err(MudError::GateBlocked(
                "Motion gate must be verified before Reveal gate".into(),
            ));
        }

        self.gates.reveal = true;

        let parts_hashes: HashMap<String, String> = self
            .parts
            .iter()
            .map(|(k, v)| {
                let serialized = serde_json::to_string(v).unwrap_or_default();
                (
                    k.clone(),
                    blake3::hash(serialized.as_bytes()).to_hex().to_string(),
                )
            })
            .collect();

        let mut hashes_sorted: Vec<(&String, &String)> = parts_hashes.iter().collect();
        hashes_sorted.sort_by_key(|a| a.0);

        let mut hasher = blake3::Hasher::new();
        for (k, v) in hashes_sorted {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }
        let lineage_hash = hasher.finalize().to_hex().to_string();

        let assembly_rec = AssemblyReceipt {
            mech_id: "Gundam-Nexus-Admitted-01".to_string(),
            parts_hashes,
            lineage_hash: lineage_hash.clone(),
            timestamp: Utc::now(),
        };
        self.assembly_receipt = Some(assembly_rec);

        let mut walkthrough_rec = WalkthroughReceipt {
            prompt: "Happy Path Playthrough".to_string(),
            command_history: self.command_history.clone(),
            command_trace: self.command_history.clone(),
            room_transitions: self.room_transitions.clone(),
            final_verdict: "PASS".to_string(),
            final_assembly_receipt_hash: Some(lineage_hash),
            timestamp: Utc::now(),
            cryptographic_signature: String::new(),
            signature_hash: String::new(),
        };

        let serialized = serde_json::to_string(&walkthrough_rec).unwrap_or_default();
        let sig = blake3::hash(serialized.as_bytes()).to_hex().to_string();
        walkthrough_rec.cryptographic_signature = sig.clone();
        walkthrough_rec.signature_hash = sig;

        self.walkthrough_receipt = Some(walkthrough_rec);
        self.emit_event(
            "assembly.admitted",
            "assembly_receipt",
            "Mech assembly admitted and registered.",
        );
        self.emit_event(
            "receipt.issued",
            "walkthrough_receipt",
            "Cryptographic walkthrough receipt issued.",
        );

        Ok("Reveal gate compliance check passed. Assembly receipt generated.".to_string())
    }

    fn cmd_assemble(&mut self, _spec: &str) -> Result<String, MudError> {
        if self.current_zone != Zone::AssemblyGantry {
            return Err(MudError::OperationError(
                "Assembly must be performed in Assembly Gantry".into(),
            ));
        }

        if !self.gates.mission
            || !self.gates.materials
            || !self.gates.primitive
            || !self.gates.runner_wall
        {
            self.emit_event(
                "assembly.refused",
                "assembly_gantry",
                "Assembly refused due to missing prior gates.",
            );
            return Err(MudError::GateBlocked(
                "Prior gates must be verified before assembly.".into(),
            ));
        }

        self.emit_event(
            "assembly.started",
            "assembly_gantry",
            "Starting mechanical assembly...",
        );

        let mating_steps = vec![
            ("torso_frame", "central frame"),
            ("head", "Neck socket"),
            ("left_arm", "Left Arm socket"),
            ("right_arm", "Right Arm socket"),
            ("left_leg", "Left Leg socket"),
            ("right_leg", "Right Leg socket"),
            ("backpack", "Backpack mount"),
            ("left_thruster", "Backpack Left Thruster mount"),
            ("right_thruster", "Backpack Right Thruster mount"),
        ];

        let mut outputs = vec!["=== Gantry Assembly started ===".to_string()];
        for (id, desc) in mating_steps {
            if !self.parts.contains_key(id) {
                self.emit_event(
                    "assembly.refused",
                    id,
                    "Missing required part for assembly.",
                );
                return Err(MudError::AssemblyFailure {
                    reason: format!("Missing part {}", id),
                });
            }

            if id == "head" {
                let head = self.parts.get("head").unwrap();
                let head_req = head.sockets_required.first().map(|s| &s.kind);
                if let Some(req) = head_req {
                    if req != "head" {
                        self.emit_event(
                            "socket.mismatched",
                            "head",
                            "Socket mismatch: torso expected 'head', got mismatch.",
                        );
                        self.emit_event("assembly.refused", "head", "Socket mismatch at head.");
                        return Err(MudError::AssemblyFailure {
                            reason: "Socket mating mismatch at Neck socket.".into(),
                        });
                    }
                }
            }

            self.emit_event(
                "socket.matched",
                id,
                &format!("Mating socket for {} to {}", id, desc),
            );
            self.emit_event("part.placed", id, &format!("Placed part {}", id));
            self.emit_event(
                "assembly.step_completed",
                id,
                &format!("Step completed: {}", desc),
            );
            outputs.push(format!("  [MATE] Placed and secured {} to {}", id, desc));
        }

        self.assembly_complete = true;
        self.assembly_completed = true;
        self.gantry.verification_status = "NOMINAL".to_string();
        self.gantry.current_state = "Completed".to_string();

        for socket in self.twin_sockets.values_mut() {
            socket.mated = true;
            socket.mated_to = Some(format!("{}_connector", socket.name));
        }

        for part in self.twin_parts.values_mut() {
            part.thermal_load += 12.5;
            part.wear_rate += 0.02;
        }

        outputs.push("Assembly completed successfully.".to_string());
        Ok(outputs.join("\n"))
    }

    fn cmd_preview(&mut self, op: &str) -> Result<String, MudError> {
        let clean = op.trim().to_lowercase();
        self.emit_event(
            "object.inspected",
            "preview",
            &format!("Previewing operation: {}", op),
        );
        match clean.as_str() {
            "assembly" => {
                Ok("Previewing assembly steps: torso_frame -> head -> left_arm -> right_arm -> left_leg -> right_leg -> backpack -> left_thruster -> right_thruster.".to_string())
            }
            "motion" | "kinetics" => {
                Ok("Previewing motion sweep for poses: stand, step, turn, kneel.".to_string())
            }
            _ => {
                Ok(format!("Preview of '{}' is clear. No conflicts predicted.", op))
            }
        }
    }

    fn cmd_receipt(&mut self, target: &str) -> Result<String, MudError> {
        let clean = target.trim().to_lowercase();
        self.emit_event(
            "object.inspected",
            target,
            &format!("Requested receipt for {}", target),
        );
        if clean == "walkthrough" || clean == "assembly" || clean == "mech" {
            if let Some(rec) = &self.walkthrough_receipt {
                return Ok(format!(
                    "Walkthrough Receipt:\nVerdict: {}\nSignature: {}\nTimestamp: {}",
                    rec.final_verdict, rec.cryptographic_signature, rec.timestamp
                ));
            } else {
                return Err(MudError::OperationError(
                    "No receipt has been generated yet. Complete the reveal gate first.".into(),
                ));
            }
        }
        Err(MudError::InvalidInspectTarget(format!(
            "Unknown receipt target '{}'",
            target
        )))
    }

    fn cmd_inventory(&mut self) -> Result<String, MudError> {
        self.emit_event(
            "object.inspected",
            "inventory",
            "Checked inventory at runner wall.",
        );
        let mut parts_list = vec!["=== Parts inventory at Runner Wall ===".to_string()];
        let mut keys: Vec<&String> = self.parts.keys().collect();
        keys.sort();
        for k in keys {
            let part = self.parts.get(k).unwrap();
            parts_list.push(format!(
                "  - {} (kind: {}, mass: {}, health: {:.2})",
                part.part_id, part.part_kind, part.mass, part.health_status
            ));
        }
        Ok(parts_list.join("\n"))
    }

    fn cmd_exits(&mut self) -> Result<String, MudError> {
        self.emit_event("object.inspected", "exits", "Checked exits list.");
        let mut exits_list = vec![format!("Legal exits from {}:", self.current_zone.name())];

        match self.current_zone {
            Zone::MissionRoom => exits_list.push("  - east -> Materials Lab".to_string()),
            Zone::MaterialsLab => {
                exits_list.push("  - west -> Mission Room".to_string());
                exits_list.push("  - east -> Primitive Foundry".to_string());
            }
            Zone::PrimitiveFoundry => {
                exits_list.push("  - west -> Materials Lab".to_string());
                exits_list.push("  - east -> Runner Wall".to_string());
            }
            Zone::RunnerWall => {
                exits_list.push("  - west -> Primitive Foundry".to_string());
                exits_list.push("  - east -> Assembly Gantry".to_string());
            }
            Zone::AssemblyGantry => {
                exits_list.push("  - west -> Runner Wall".to_string());
                exits_list.push("  - east -> Fit Bay".to_string());
            }
            Zone::FitBay => {
                exits_list.push("  - west -> Assembly Gantry".to_string());
                exits_list.push("  - east -> Collision Bay".to_string());
            }
            Zone::CollisionBay => {
                exits_list.push("  - west -> Fit Bay".to_string());
                exits_list.push("  - east -> Proving Ground".to_string());
            }
            Zone::ProvingGround => {
                exits_list.push("  - west -> Collision Bay".to_string());
                exits_list.push("  - east -> Reveal Platform".to_string());
            }
            Zone::RevealPlatform => {
                exits_list.push("  - west -> Proving Ground".to_string());
            }
        }

        Ok(exits_list.join("\n"))
    }

    fn cmd_stress_part(&mut self, part_name: &str) -> Result<String, MudError> {
        let name_lower = part_name.to_lowercase();
        self.emit_event(
            "object.inspected",
            part_name,
            &format!("Stressed part {}", part_name),
        );

        if let Some(part) = self.parts.get_mut(&name_lower) {
            part.health_status -= 0.4;
            if part.health_status < 0.0 {
                part.health_status = 0.0;
            }
        }

        if name_lower == "head" {
            if let Some(joint) = self.twin_joints.get_mut("Neck") {
                joint.wear_level += 0.4;
                if joint.wear_level > 0.8 {
                    joint.status = "Degraded".to_string();
                    joint.motion_failures += 1;
                    self.overall_degradation_state = "FailDegraded".to_string();
                }
            }
            if let Some(part) = self.parts.get_mut("head") {
                part.health_status = 0.4;
            }
            if let Some(head) = self.twin_parts.get_mut("Head") {
                head.health_score = 0.4;
            }
            Ok("Stressed Head Neck joint".to_string())
        } else if name_lower == "torso" || name_lower == "torso_frame" {
            if let Some(socket) = self.twin_sockets.get_mut("torso") {
                socket.stress_level += 0.3;
            }
            if let Some(part) = self.parts.get_mut("torso_frame") {
                part.health_status = 0.7;
            }
            if let Some(torso) = self.twin_parts.get_mut("Torso") {
                torso.health_score = 0.7;
            }
            Ok("Stressed Torso socket".to_string())
        } else {
            Ok(format!("Stressed part {}.", part_name))
        }
    }

    /// Retrieve the overall PHM metrics for tests
    pub fn query_mech_phm(&self) -> MechPhmReport {
        let phm = self.get_phm_metrics();
        let fault_state =
            if !phm.active_faults.is_empty() || phm.degradation_state == "FailDegraded" {
                "KineticsFailure".to_string()
            } else {
                "Nominal".to_string()
            };

        MechPhmReport {
            degradation_state: phm.degradation_state.clone(),
            fault_state,
        }
    }

    /// Retrieve the overall PHM Metrics.
    pub fn get_phm_metrics(&self) -> PhmMetrics {
        let mut overall_health = 1.0f32;
        let mut active_faults = Vec::new();
        let mut max_wear = 0.0f32;
        let mut total_stress = 0.0f32;
        let mut max_temp = 0.0f32;
        let mut total_motion_failures = 0;

        let mut nan_detected = false;
        for part in self.twin_parts.values() {
            if part.health_score.is_nan() {
                nan_detected = true;
                active_faults.push(format!("Part {}: NaN Health Score Defect", part.name));
            } else if part.health_score < overall_health {
                overall_health = part.health_score;
            }
            if part.fault_status != "None" {
                active_faults.push(format!("Part {}: {}", part.name, part.fault_status));
            }
            if part.thermal_load > max_temp {
                max_temp = part.thermal_load;
            }
        }

        if nan_detected {
            overall_health = 0.0;
        }

        for socket in self.twin_sockets.values() {
            total_stress += socket.stress_level;
            if socket.stress_level > 0.8 && !socket.mated {
                active_faults.push(format!("Socket {} high stress / unmated", socket.name));
            }
        }

        for joint in self.twin_joints.values() {
            if joint.wear_level > max_wear {
                max_wear = joint.wear_level;
            }
            total_motion_failures += joint.motion_failures;
            if joint.status != "Operational" {
                active_faults.push(format!("Joint {}: {}", joint.name, joint.status));
            }
        }

        let mut maintenance_state = if overall_health < 0.5 || max_wear > 0.7 {
            "Required".to_string()
        } else if overall_health < 0.85 || max_wear > 0.3 {
            "Scheduled".to_string()
        } else {
            "None".to_string()
        };

        if nan_detected {
            maintenance_state = "Required".to_string();
        }

        let mut part_healths = Vec::new();
        for part in self.twin_parts.values() {
            let h = if part.health_score.is_nan() {
                0.0
            } else {
                part.health_score.max(0.0)
            };
            part_healths.push(h);
        }
        let remaining_useful_life = compute_weibull_rul(overall_health, &part_healths);

        PhmMetrics {
            overall_health,
            remaining_useful_life,
            active_faults,
            degradation_state: self.overall_degradation_state.clone(),
            maintenance_state,
            socket_stress: total_stress / 5.0,
            joint_wear: max_wear,
            collision_incidents: self.collision_incidents,
            thermal_load: max_temp,
            motion_failures: total_motion_failures,
        }
    }

    pub fn run_mape_k_loop(&mut self) {
        let phm = self.get_phm_metrics();

        let _health_status = if phm.overall_health >= 0.85 {
            HealthStatus::Healthy
        } else if phm.overall_health >= 0.5 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let circuit_state = match self.overall_degradation_state.as_str() {
            "Healthy" | "Nominal" => CircuitState::Closed,
            "FailDegraded" | "FailOperational" | "Degraded" => CircuitState::HalfOpen,
            "FailSafe" | "Fault" => CircuitState::Open,
            _ => CircuitState::Closed,
        };

        if circuit_state == CircuitState::Open {
            self.gantry.verification_status = "FAULTY".to_string();
        }
    }

    pub fn generate_walkthrough_receipt(
        &self,
        prompt: &str,
    ) -> Result<WalkthroughReceipt, MudError> {
        if self.current_zone != Zone::RevealPlatform {
            return Err(MudError::OperationError(
                "Replay receipts can only be generated on the Reveal Platform upon playthrough completion.".into()
            ));
        }

        let final_verdict = if self.gates.motion && self.gates.collision && self.gates.fit {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        };

        let final_assembly_receipt_hash = self
            .assembly_receipt
            .as_ref()
            .map(|r| r.lineage_hash.clone());

        let actual_timestamp = Utc::now();
        let mut temp_receipt = WalkthroughReceipt {
            prompt: prompt.to_string(),
            command_history: self.command_history.clone(),
            command_trace: self.command_history.clone(),
            room_transitions: self.room_transitions.clone(),
            final_verdict,
            final_assembly_receipt_hash,
            timestamp: DateTime::<Utc>::default(),
            cryptographic_signature: String::new(),
            signature_hash: String::new(),
        };

        let serialized = serde_json::to_string(&temp_receipt).unwrap_or_default();
        let cryptographic_signature = blake3::hash(serialized.as_bytes()).to_hex().to_string();

        temp_receipt.timestamp = actual_timestamp;
        temp_receipt.cryptographic_signature = cryptographic_signature.clone();
        temp_receipt.signature_hash = cryptographic_signature;
        Ok(temp_receipt)
    }
}

fn gamma_approx(x: f64) -> f64 {
    const P: [f64; 8] = [
        676.5203681218851,
        -1259.1392167224028,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507343278686905,
        -0.13857109526572012,
        9.984_369_578_019_572e-6,
        1.5056327351493116e-7,
    ];
    if x < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_approx(1.0 - x))
    } else {
        let x = x - 1.0;
        let mut a = 0.999_999_999_999_809_9_f64;
        for (i, &p) in P.iter().enumerate() {
            a += p / (x + i as f64 + 1.0);
        }
        let t = x + 7.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
    }
}

fn compute_weibull_rul(overall_health: f32, part_healths: &[f32]) -> f32 {
    if overall_health <= 0.0 {
        return 0.0;
    }
    let n = part_healths.len() as f64;
    if n == 0.0 {
        return overall_health * 800.0;
    }
    let mean = part_healths.iter().map(|&x| x as f64).sum::<f64>() / n;
    let variance = if n > 1.0 {
        part_healths
            .iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 0.2 };
    let shape = if cv <= 0.0 || !cv.is_finite() {
        1.0
    } else {
        cv.powf(-1.086).clamp(0.1, 20.0)
    };
    let mean_life = (overall_health * 800.0) as f64;
    let g = gamma_approx(1.0 + 1.0 / shape);
    let scale = if g > 0.0 { mean_life / g } else { mean_life };
    (scale * (overall_health as f64).powf(1.0 / shape)) as f32
}

pub fn parse_command(input: &str) -> Result<MudCommand, MudError> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    if lower == "look" {
        Ok(MudCommand::Look)
    } else if lower == "inventory" {
        Ok(MudCommand::Inventory)
    } else if lower == "exits" {
        Ok(MudCommand::Exits)
    } else if lower.starts_with("go ") {
        let dest = trimmed[3..].trim().to_string();
        if dest.is_empty() {
            Err(MudError::InvalidCommand(
                "Go command requires a direction or zone name".into(),
            ))
        } else {
            Ok(MudCommand::Go(dest))
        }
    } else if lower.starts_with("inspect ") {
        let target = trimmed[8..].trim().to_string();
        if target.is_empty() {
            Err(MudError::InvalidCommand(
                "Inspect command requires a target".into(),
            ))
        } else {
            Ok(MudCommand::Inspect(target))
        }
    } else if lower.starts_with("health ") {
        let target = trimmed[7..].trim().to_string();
        if target.is_empty() {
            Err(MudError::InvalidCommand(
                "Health command requires a target".into(),
            ))
        } else {
            Ok(MudCommand::Health(target))
        }
    } else if lower.starts_with("diagnose ") {
        let target = trimmed[9..].trim().to_string();
        Ok(MudCommand::Diagnose(target))
    } else if lower == "diagnose" {
        Ok(MudCommand::Diagnose(String::new()))
    } else if lower.starts_with("verify ") {
        let gate = trimmed[7..].trim().to_string();
        if gate.is_empty() {
            Err(MudError::InvalidCommand(
                "Verify command requires a gate name".into(),
            ))
        } else {
            Ok(MudCommand::Verify(gate))
        }
    } else if lower.starts_with("assemble ") {
        let spec = trimmed[9..].trim().to_string();
        if spec.is_empty() {
            Err(MudError::InvalidCommand(
                "Assemble command requires a spec".into(),
            ))
        } else {
            Ok(MudCommand::Assemble(spec))
        }
    } else if lower.starts_with("preview ") {
        let op = trimmed[8..].trim().to_string();
        if op.is_empty() {
            Err(MudError::InvalidCommand(
                "Preview command requires an operation".into(),
            ))
        } else {
            Ok(MudCommand::Preview(op))
        }
    } else if lower.starts_with("receipt ") {
        let target = trimmed[8..].trim().to_string();
        if target.is_empty() {
            Err(MudError::InvalidCommand(
                "Receipt command requires a target".into(),
            ))
        } else {
            Ok(MudCommand::Receipt(target))
        }
    } else {
        parse_alias_command(&lower, trimmed)
    }
}

/// Resolve shorthand / alias commands that don't match the primary parse rules.
fn parse_alias_command(lower: &str, original: &str) -> Result<MudCommand, MudError> {
    match lower {
        "watch assembly" => Ok(MudCommand::Assemble("standard".to_string())),
        "verify clearance" => Ok(MudCommand::Verify("collision".to_string())),
        "verify kinetics" => Ok(MudCommand::Verify("motion".to_string())),
        _ => Err(MudError::InvalidCommand(format!(
            "Unknown command: '{}'",
            original
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path_walkthrough() {
        let mut engine = MudEngine::new();
        assert_eq!(engine.current_zone, Zone::MissionRoom);

        let out_look = engine.execute_command("look").unwrap();
        assert!(out_look.contains("Strategic blueprint"));

        let verify_mission = engine.execute_command("verify mission").unwrap();
        assert!(verify_mission.contains("compliance check passed"));

        let _ = engine.execute_command("go materials_lab").unwrap();
        assert_eq!(engine.current_zone, Zone::MaterialsLab);

        let verify_materials = engine.execute_command("verify materials").unwrap();
        assert!(verify_materials.contains("compliance check passed"));

        let _ = engine.execute_command("go primitive_foundry").unwrap();
        let verify_primitive = engine.execute_command("verify primitive").unwrap();
        assert!(verify_primitive.contains("compliance check passed"));

        let _ = engine.execute_command("go runner_wall").unwrap();
        let inventory_res = engine.execute_command("inventory").unwrap();
        assert!(inventory_res.contains("torso_frame"));
        assert!(inventory_res.contains("head"));

        let inspect_res = engine.execute_command("inspect head").unwrap();
        assert!(inspect_res.contains("Mass: 10 kg"));

        let verify_runner = engine.execute_command("verify runner_wall").unwrap();
        assert!(verify_runner.contains("compliance check passed"));

        let _ = engine.execute_command("go assembly_gantry").unwrap();
        let preview_res = engine.execute_command("preview assembly").unwrap();
        assert!(preview_res.contains("Previewing assembly steps"));

        let assemble_res = engine.execute_command("assemble standard").unwrap();
        assert!(assemble_res.contains("Assembly completed successfully"));
        assert!(engine.assembly_complete);

        let verify_assembly = engine.execute_command("verify assembly").unwrap();
        assert!(verify_assembly.contains("compliance check passed"));

        let _ = engine.execute_command("go fit_bay").unwrap();
        let verify_fit = engine.execute_command("verify fit").unwrap();
        assert!(verify_fit.contains("compliance check passed"));

        let _ = engine.execute_command("go collision_bay").unwrap();
        let verify_collision = engine.execute_command("verify collision").unwrap();
        assert!(verify_collision.contains("compliance check passed"));

        let _ = engine.execute_command("go proving_ground").unwrap();
        let preview_motion = engine.execute_command("preview motion").unwrap();
        assert!(preview_motion.contains("Previewing motion sweep"));

        let verify_motion = engine.execute_command("verify motion").unwrap();
        assert!(verify_motion.contains("compliance check passed"));

        let _ = engine.execute_command("go reveal_platform").unwrap();
        let verify_reveal = engine.execute_command("verify reveal").unwrap();
        assert!(verify_reveal.contains("compliance check passed"));

        let receipt_res = engine.execute_command("receipt walkthrough").unwrap();
        assert!(receipt_res.contains("Walkthrough Receipt:"));
        assert!(receipt_res.contains("Verdict: PASS"));
    }

    #[test]
    fn test_gate_rejections_and_invalid_movement() {
        let mut engine = MudEngine::new();
        // West from MissionRoom is invalid transition
        let move_fail = engine.execute_command("go west");
        assert!(move_fail.is_err());

        // Skipping rooms (direct travel) is rejected
        let skip_fail = engine.execute_command("go reveal_platform");
        assert!(skip_fail.is_err());

        // Moving to MaterialsLab without verify mission is blocked
        let blocked = engine.execute_command("go materials_lab");
        assert!(blocked.is_err());
        assert!(matches!(blocked.unwrap_err(), MudError::GateBlocked(_)));
    }

    // ── Zone unit tests ─────────────────────────────────────────────────────

    #[test]
    fn zone_all_returns_nine_zones() {
        assert_eq!(Zone::all().len(), 9, "GMF has exactly 9 factory zones");
    }

    #[test]
    fn zone_parse_canonical_names() {
        assert_eq!(Zone::parse("mission room"), Some(Zone::MissionRoom));
        assert_eq!(Zone::parse("materials lab"), Some(Zone::MaterialsLab));
        assert_eq!(Zone::parse("assembly gantry"), Some(Zone::AssemblyGantry));
        assert_eq!(Zone::parse("reveal platform"), Some(Zone::RevealPlatform));
    }

    #[test]
    fn zone_parse_short_aliases() {
        assert_eq!(Zone::parse("mission"), Some(Zone::MissionRoom));
        assert_eq!(Zone::parse("runner"), Some(Zone::RunnerWall));
        assert_eq!(Zone::parse("reveal"), Some(Zone::RevealPlatform));
    }

    #[test]
    fn zone_parse_unknown_returns_none() {
        assert_eq!(Zone::parse("hangar"), None);
        assert_eq!(Zone::parse(""), None);
        assert_eq!(Zone::parse("zone 0"), None);
    }

    #[test]
    fn zone_elevation_is_its_ordinal() {
        assert_eq!(Zone::MissionRoom.elevation(), 0);
        assert_eq!(Zone::RevealPlatform.elevation(), 8);
        assert_eq!(Zone::AssemblyGantry.elevation(), 4);
    }

    #[test]
    fn zone_to_str_round_trips_through_parse() {
        for zone in Zone::all() {
            let s = zone.to_str();
            assert_eq!(Zone::parse(s), Some(zone), "to_str/parse round-trip failed for {:?}", zone);
        }
    }

    #[test]
    fn zone_description_is_non_empty_for_all_zones() {
        for zone in Zone::all() {
            assert!(!zone.description().is_empty(), "description missing for {:?}", zone);
        }
    }

    // ── AABB unit tests ─────────────────────────────────────────────────────

    #[test]
    fn aabb_overlapping_boxes_intersect() {
        let a = AABB { min_x: 0.0, max_x: 2.0, min_y: 0.0, max_y: 2.0, min_z: 0.0, max_z: 2.0 };
        let b = AABB { min_x: 1.0, max_x: 3.0, min_y: 1.0, max_y: 3.0, min_z: 1.0, max_z: 3.0 };
        assert!(a.intersects(&b));
    }

    #[test]
    fn aabb_non_overlapping_boxes_do_not_intersect() {
        let a = AABB { min_x: 0.0, max_x: 1.0, min_y: 0.0, max_y: 1.0, min_z: 0.0, max_z: 1.0 };
        let b = AABB { min_x: 2.0, max_x: 3.0, min_y: 2.0, max_y: 3.0, min_z: 2.0, max_z: 3.0 };
        assert!(!a.intersects(&b));
    }

    #[test]
    fn aabb_touching_at_face_does_not_intersect() {
        // Touching exactly at face: max_x == min_x (open interval)
        let a = AABB { min_x: 0.0, max_x: 1.0, min_y: 0.0, max_y: 1.0, min_z: 0.0, max_z: 1.0 };
        let b = AABB { min_x: 1.0, max_x: 2.0, min_y: 0.0, max_y: 1.0, min_z: 0.0, max_z: 1.0 };
        assert!(!a.intersects(&b), "touching at face only should not intersect");
    }

    // ── parse_command unit tests ────────────────────────────────────────────

    #[test]
    fn parse_command_look() {
        let cmd = parse_command("look").unwrap();
        assert!(matches!(cmd, MudCommand::Look));
    }

    #[test]
    fn parse_command_inventory() {
        let cmd = parse_command("inventory").unwrap();
        assert!(matches!(cmd, MudCommand::Inventory));
    }

    #[test]
    fn parse_command_go_zone() {
        let cmd = parse_command("go mission_room").unwrap();
        assert!(matches!(cmd, MudCommand::Go(ref s) if s == "mission_room"));
    }

    #[test]
    fn parse_command_unknown_returns_error() {
        let result = parse_command("fly to moon");
        assert!(result.is_err());
    }
}
