use serde::{Deserialize, Serialize};

/// Object types in the GMF digital twin.
/// Every object that participates in manufacturing events must have a type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GmfObjectType {
    // Manufacturing objects
    MechPart,
    ManufacturingZone,
    Socket,
    MechAssembly,
    // Reliability objects
    CircuitBreaker,
    HealthCheck,
    // Human-scale objects (Titanfall-inspired pilot/mech bond)
    Pilot,
    Cockpit,
    Battery,
    // Infrastructure (Into the Breach — protect civilization, not scoreboard)
    EdenGridNode,
    WaterSystem,
    FarmOutput,
    HospitalUnit,
    FactoryAvailability,
    // Mission objects
    MissionContract,
    Lance,
    RepairQueue,
    SalvageItem,
}

impl GmfObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MechPart => "MechPart",
            Self::ManufacturingZone => "ManufacturingZone",
            Self::Socket => "Socket",
            Self::MechAssembly => "MechAssembly",
            Self::CircuitBreaker => "CircuitBreaker",
            Self::HealthCheck => "HealthCheck",
            Self::Pilot => "Pilot",
            Self::Cockpit => "Cockpit",
            Self::Battery => "Battery",
            Self::EdenGridNode => "EdenGridNode",
            Self::WaterSystem => "WaterSystem",
            Self::FarmOutput => "FarmOutput",
            Self::HospitalUnit => "HospitalUnit",
            Self::FactoryAvailability => "FactoryAvailability",
            Self::MissionContract => "MissionContract",
            Self::Lance => "Lance",
            Self::RepairQueue => "RepairQueue",
            Self::SalvageItem => "SalvageItem",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmfObject {
    pub id: String,
    pub object_type: GmfObjectType,
}

impl GmfObject {
    pub fn new(id: impl Into<String>, object_type: GmfObjectType) -> Self {
        Self { id: id.into(), object_type }
    }
}
