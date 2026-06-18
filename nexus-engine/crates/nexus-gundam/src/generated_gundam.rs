#![allow(unused_imports, dead_code)]

use serde::{Serialize, Deserialize};

pub trait OntologyName {
    fn ontology_name() -> &'static str;
}

pub trait PlanetCategory {}
pub trait PlanetDimensionCategory {}
pub trait MobilityTypeCategory: Serialize {
    fn physical(&self) -> &Mobility;
    fn type_name(&self) -> &'static str;
}
pub trait MechClassCategory {}
pub trait PreservationDomainCategory {}
pub trait ExperiencePhaseCategory {}
pub trait MythologyArchetypeCategory {}
pub trait CosmologyPhaseCategory {}
pub trait ArkFormCategory {}
pub trait ArkPurposeCategory {}
pub trait HistoryTypeCategory {}
pub trait PreservationAspectCategory {}
pub trait MechPrimitiveCategory {}

pub trait CosmicCycleCategory {}
pub trait ResourceCategory {}
pub trait FrameTypeCategory {}
pub trait PowerSourceCategory {}
pub trait ArmorTypeCategory {}
pub trait WeaponTypeCategory {}
pub trait SensorTypeCategory {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0] &&
        self.min[1] <= other.max[1] && self.max[1] >= other.min[1] &&
        self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotationLimits {
    pub min_yaw: f32,
    pub max_yaw: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_roll: f32,
    pub max_roll: f32,
}

impl Default for RotationLimits {
    fn default() -> Self {
        Self {
            min_yaw: -180.0,
            max_yaw: 180.0,
            min_pitch: -90.0,
            max_pitch: 90.0,
            min_roll: -180.0,
            max_roll: 180.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub slot_count: usize,
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            id: "Standard Frame".to_string(),
            mass: 50.0,
            occupancy: AABB::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            clearance: AABB::new([-1.1, -1.1, -1.1], [1.1, 1.1, 1.1]),
            slot_count: 5,
        }
    }
}

impl MechPrimitiveCategory for Frame {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    pub name: String,
    pub parent_component_id: String,
    pub child_component_id: String,
    pub location: [f32; 3],
    pub limits: Option<RotationLimits>,
    pub mass: f32,
}

impl MechPrimitiveCategory for Joint {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Power {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub energy_capacity: f32,
    pub output: f32,
}

impl Default for Power {
    fn default() -> Self {
        Power {
            id: "Fusion Reactor".to_string(),
            mass: 20.0,
            occupancy: AABB::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
            clearance: AABB::new([-0.6, -0.6, -0.6], [0.6, 0.6, 0.6]),
            energy_capacity: 1000.0,
            output: 100.0,
        }
    }
}

impl MechPrimitiveCategory for Power {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub defense_rating: f32,
    pub material: String,
}

impl Default for Armor {
    fn default() -> Self {
        Armor {
            id: "Nano-composite Armor".to_string(),
            mass: 30.0,
            occupancy: AABB::new([-1.05, -1.05, -1.05], [1.05, 1.05, 1.05]),
            clearance: AABB::new([-1.1, -1.1, -1.1], [1.1, 1.1, 1.1]),
            defense_rating: 100.0,
            material: "Nano-composite".to_string(),
        }
    }
}

impl MechPrimitiveCategory for Armor {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub damage: f32,
    pub range: f32,
}

impl MechPrimitiveCategory for Weapon {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sensor {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub detection_range: f32,
}

impl MechPrimitiveCategory for Sensor {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtilitySystem {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub utility_type: String,
}

impl MechPrimitiveCategory for UtilitySystem {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mobility {
    pub id: String,
    pub mass: f32,
    pub occupancy: AABB,
    pub clearance: AABB,
    pub load_capacity: f32,
    pub max_speed: f32,
}

impl MechPrimitiveCategory for Mobility {}

impl Default for Mobility {
    fn default() -> Self {
        Mobility {
            id: "Standard Mobility".to_string(),
            mass: 15.0,
            occupancy: AABB::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
            clearance: AABB::new([-0.6, -0.6, -0.6], [0.6, 0.6, 0.6]),
            load_capacity: 100.0,
            max_speed: 10.0,
        }
    }
}

// --- Mobility Typestates ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Walking {
    pub physical: Mobility,
    pub leg_count: u32,
}

impl Default for Walking {
    fn default() -> Self {
        Walking {
            physical: Mobility::default(),
            leg_count: 2,
        }
    }
}

impl MobilityTypeCategory for Walking {
    fn physical(&self) -> &Mobility {
        &self.physical
    }
    fn type_name(&self) -> &'static str {
        "Walking"
    }
}

impl OntologyName for Walking {
    fn ontology_name() -> &'static str {
        "Walking"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flight {
    pub physical: Mobility,
    pub wing_span: f32,
}

impl Default for Flight {
    fn default() -> Self {
        Flight {
            physical: Mobility::default(),
            wing_span: 12.0,
        }
    }
}

impl MobilityTypeCategory for Flight {
    fn physical(&self) -> &Mobility {
        &self.physical
    }
    fn type_name(&self) -> &'static str {
        "Flight"
    }
}

impl OntologyName for Flight {
    fn ontology_name() -> &'static str {
        "Flight"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hover {
    pub physical: Mobility,
    pub ground_clearance: f32,
}

impl MobilityTypeCategory for Hover {
    fn physical(&self) -> &Mobility {
        &self.physical
    }
    fn type_name(&self) -> &'static str {
        "Hover"
    }
}

impl OntologyName for Hover {
    fn ontology_name() -> &'static str {
        "Hover"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Space {
    pub physical: Mobility,
    pub thruster_count: u32,
}

impl MobilityTypeCategory for Space {
    fn physical(&self) -> &Mobility {
        &self.physical
    }
    fn type_name(&self) -> &'static str {
        "Space"
    }
}

impl OntologyName for Space {
    fn ontology_name() -> &'static str {
        "Space"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aquatic {
    pub physical: Mobility,
    pub depth_rating: f32,
}

impl MobilityTypeCategory for Aquatic {
    fn physical(&self) -> &Mobility {
        &self.physical
    }
    fn type_name(&self) -> &'static str {
        "Aquatic"
    }
}

impl OntologyName for Aquatic {
    fn ontology_name() -> &'static str {
        "Aquatic"
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ambition;

impl OntologyName for Ambition {
    fn ontology_name() -> &'static str {
        "Ambition"
    }
}


impl PlanetDimensionCategory for Ambition {}






#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Arcades;

impl OntologyName for Arcades {
    fn ontology_name() -> &'static str {
        "Arcades"
    }
}


impl PreservationDomainCategory for Arcades {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ark;

impl OntologyName for Ark {
    fn ontology_name() -> &'static str {
        "Ark"
    }
}


impl MechClassCategory for Ark {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BeamRifle;

impl OntologyName for BeamRifle {
    fn ontology_name() -> &'static str {
        "BeamRifle"
    }
}


impl WeaponTypeCategory for BeamRifle {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BeamSaber;

impl OntologyName for BeamSaber {
    fn ontology_name() -> &'static str {
        "BeamSaber"
    }
}


impl WeaponTypeCategory for BeamSaber {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Beauty;

impl OntologyName for Beauty {
    fn ontology_name() -> &'static str {
        "Beauty"
    }
}


impl PlanetDimensionCategory for Beauty {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BecomeMythology;

impl OntologyName for BecomeMythology {
    fn ontology_name() -> &'static str {
        "BecomeMythology"
    }
}


impl ExperiencePhaseCategory for BecomeMythology {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BrowserMMOs;

impl OntologyName for BrowserMMOs {
    fn ontology_name() -> &'static str {
        "BrowserMMOs"
    }
}


impl PreservationDomainCategory for BrowserMMOs {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Build;

impl OntologyName for Build {
    fn ontology_name() -> &'static str {
        "Build"
    }
}


impl ExperiencePhaseCategory for Build {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Builder;

impl OntologyName for Builder {
    fn ontology_name() -> &'static str {
        "Builder"
    }
}


impl MechClassCategory for Builder {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BuilderArchetype;

impl OntologyName for BuilderArchetype {
    fn ontology_name() -> &'static str {
        "BuilderArchetype"
    }
}


impl MythologyArchetypeCategory for BuilderArchetype {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CityArk;

impl OntologyName for CityArk {
    fn ontology_name() -> &'static str {
        "CityArk"
    }
}


impl ArkFormCategory for CityArk {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColonyArk;

impl OntologyName for ColonyArk {
    fn ontology_name() -> &'static str {
        "ColonyArk"
    }
}


impl ArkFormCategory for ColonyArk {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Community;

impl OntologyName for Community {
    fn ontology_name() -> &'static str {
        "Community"
    }
}


impl PlanetDimensionCategory for Community {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CreateHistory;

impl OntologyName for CreateHistory {
    fn ontology_name() -> &'static str {
        "CreateHistory"
    }
}


impl ExperiencePhaseCategory for CreateHistory {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Destroyer;

impl OntologyName for Destroyer {
    fn ontology_name() -> &'static str {
        "Destroyer"
    }
}


impl MythologyArchetypeCategory for Destroyer {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Discover;

impl OntologyName for Discover {
    fn ontology_name() -> &'static str {
        "Discover"
    }
}


impl ExperiencePhaseCategory for Discover {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DualSensor;

impl OntologyName for DualSensor {
    fn ontology_name() -> &'static str {
        "DualSensor"
    }
}


impl SensorTypeCategory for DualSensor {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Earth;

impl OntologyName for Earth {
    fn ontology_name() -> &'static str {
        "Earth"
    }
}


impl PlanetCategory for Earth {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Expand;

impl OntologyName for Expand {
    fn ontology_name() -> &'static str {
        "Expand"
    }
}


impl ExperiencePhaseCategory for Expand {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Expansion;

impl OntologyName for Expansion {
    fn ontology_name() -> &'static str {
        "Expansion"
    }
}


impl PlanetDimensionCategory for Expansion {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExpansionCycle;

impl OntologyName for ExpansionCycle {
    fn ontology_name() -> &'static str {
        "ExpansionCycle"
    }
}


impl CosmicCycleCategory for ExpansionCycle {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Explore;

impl OntologyName for Explore {
    fn ontology_name() -> &'static str {
        "Explore"
    }
}


impl ExperiencePhaseCategory for Explore {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Explorer;

impl OntologyName for Explorer {
    fn ontology_name() -> &'static str {
        "Explorer"
    }
}


impl MechClassCategory for Explorer {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Faith;

impl OntologyName for Faith {
    fn ontology_name() -> &'static str {
        "Faith"
    }
}


impl PlanetDimensionCategory for Faith {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlashGames;

impl OntologyName for FlashGames {
    fn ontology_name() -> &'static str {
        "FlashGames"
    }
}


impl PreservationDomainCategory for FlashGames {}






#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FusionReactor;

impl OntologyName for FusionReactor {
    fn ontology_name() -> &'static str {
        "FusionReactor"
    }
}


impl PowerSourceCategory for FusionReactor {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GMFrame;

impl OntologyName for GMFrame {
    fn ontology_name() -> &'static str {
        "GMFrame"
    }
}


impl FrameTypeCategory for GMFrame {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Growth;

impl OntologyName for Growth {
    fn ontology_name() -> &'static str {
        "Growth"
    }
}


impl CosmicCycleCategory for Growth {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Guardian;

impl OntologyName for Guardian {
    fn ontology_name() -> &'static str {
        "Guardian"
    }
}


impl MechClassCategory for Guardian {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GundamFrame;

impl OntologyName for GundamFrame {
    fn ontology_name() -> &'static str {
        "GundamFrame"
    }
}


impl FrameTypeCategory for GundamFrame {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gundanium;

impl OntologyName for Gundanium {
    fn ontology_name() -> &'static str {
        "Gundanium"
    }
}


impl ResourceCategory for Gundanium {}
impl ArmorTypeCategory for Gundanium {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeatHawk;

impl OntologyName for HeatHawk {
    fn ontology_name() -> &'static str {
        "HeatHawk"
    }
}


impl WeaponTypeCategory for HeatHawk {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hero;

impl OntologyName for Hero {
    fn ontology_name() -> &'static str {
        "Hero"
    }
}


impl MythologyArchetypeCategory for Hero {}






#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Judgment;

impl OntologyName for Judgment {
    fn ontology_name() -> &'static str {
        "Judgment"
    }
}


impl CosmicCycleCategory for Judgment {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Knowledge;

impl OntologyName for Knowledge {
    fn ontology_name() -> &'static str {
        "Knowledge"
    }
}


impl PlanetDimensionCategory for Knowledge {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KnowledgePreservation;

impl OntologyName for KnowledgePreservation {
    fn ontology_name() -> &'static str {
        "KnowledgePreservation"
    }
}


impl ArkPurposeCategory for KnowledgePreservation {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LunaTitanium;

impl OntologyName for LunaTitanium {
    fn ontology_name() -> &'static str {
        "LunaTitanium"
    }
}


impl ResourceCategory for LunaTitanium {}
impl ArmorTypeCategory for LunaTitanium {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mars;

impl OntologyName for Mars {
    fn ontology_name() -> &'static str {
        "Mars"
    }
}


impl PlanetCategory for Mars {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Miner;

impl OntologyName for Miner {
    fn ontology_name() -> &'static str {
        "Miner"
    }
}


impl MechClassCategory for Miner {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MinovskyParticles;

impl OntologyName for MinovskyParticles {
    fn ontology_name() -> &'static str {
        "MinovskyParticles"
    }
}


impl ResourceCategory for MinovskyParticles {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MinovskyUltracompact;

impl OntologyName for MinovskyUltracompact {
    fn ontology_name() -> &'static str {
        "MinovskyUltracompact"
    }
}


impl PowerSourceCategory for MinovskyUltracompact {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Monoeye;

impl OntologyName for Monoeye {
    fn ontology_name() -> &'static str {
        "Monoeye"
    }
}


impl SensorTypeCategory for Monoeye {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NanoComposite;

impl OntologyName for NanoComposite {
    fn ontology_name() -> &'static str {
        "NanoComposite"
    }
}


impl ArmorTypeCategory for NanoComposite {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Order;

impl OntologyName for Order {
    fn ontology_name() -> &'static str {
        "Order"
    }
}


impl PlanetDimensionCategory for Order {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PreservationCycle;

impl OntologyName for PreservationCycle {
    fn ontology_name() -> &'static str {
        "PreservationCycle"
    }
}


impl CosmicCycleCategory for PreservationCycle {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preserve;

impl OntologyName for Preserve {
    fn ontology_name() -> &'static str {
        "Preserve"
    }
}


impl ExperiencePhaseCategory for Preserve {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preserver;

impl OntologyName for Preserver {
    fn ontology_name() -> &'static str {
        "Preserver"
    }
}


impl MythologyArchetypeCategory for Preserver {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Prophet;

impl OntologyName for Prophet {
    fn ontology_name() -> &'static str {
        "Prophet"
    }
}


impl MythologyArchetypeCategory for Prophet {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Renewal;

impl OntologyName for Renewal {
    fn ontology_name() -> &'static str {
        "Renewal"
    }
}


impl CosmicCycleCategory for Renewal {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Risk;

impl OntologyName for Risk {
    fn ontology_name() -> &'static str {
        "Risk"
    }
}


impl PlanetDimensionCategory for Risk {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Saint;

impl OntologyName for Saint {
    fn ontology_name() -> &'static str {
        "Saint"
    }
}


impl MythologyArchetypeCategory for Saint {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sentinel;

impl OntologyName for Sentinel {
    fn ontology_name() -> &'static str {
        "Sentinel"
    }
}


impl PlanetCategory for Sentinel {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SocialGames;

impl OntologyName for SocialGames {
    fn ontology_name() -> &'static str {
        "SocialGames"
    }
}


impl PreservationDomainCategory for SocialGames {}






#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Trader;

impl OntologyName for Trader {
    fn ontology_name() -> &'static str {
        "Trader"
    }
}


impl MechClassCategory for Trader {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Venus;

impl OntologyName for Venus {
    fn ontology_name() -> &'static str {
        "Venus"
    }
}


impl PlanetCategory for Venus {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Villain;

impl OntologyName for Villain {
    fn ontology_name() -> &'static str {
        "Villain"
    }
}


impl MythologyArchetypeCategory for Villain {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VirtualWorlds;

impl OntologyName for VirtualWorlds {
    fn ontology_name() -> &'static str {
        "VirtualWorlds"
    }
}


impl PreservationDomainCategory for VirtualWorlds {}






#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Warrior;

impl OntologyName for Warrior {
    fn ontology_name() -> &'static str {
        "Warrior"
    }
}


impl MechClassCategory for Warrior {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Worker;

impl OntologyName for Worker {
    fn ontology_name() -> &'static str {
        "Worker"
    }
}


impl MechClassCategory for Worker {}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZakuFrame;

impl OntologyName for ZakuFrame {
    fn ontology_name() -> &'static str {
        "ZakuFrame"
    }
}


impl FrameTypeCategory for ZakuFrame {}


