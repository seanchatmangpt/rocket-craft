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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmf_object_new_stores_id_and_type() {
        let obj = GmfObject::new("part-001", GmfObjectType::MechPart);
        assert_eq!(obj.id, "part-001");
        assert_eq!(obj.object_type, GmfObjectType::MechPart);
    }

    #[test]
    fn as_str_returns_canonical_name() {
        assert_eq!(GmfObjectType::MechPart.as_str(), "MechPart");
        assert_eq!(GmfObjectType::Pilot.as_str(), "Pilot");
        assert_eq!(GmfObjectType::EdenGridNode.as_str(), "EdenGridNode");
    }

    #[test]
    fn object_types_are_distinct() {
        assert_ne!(GmfObjectType::MechPart.as_str(), GmfObjectType::Socket.as_str());
    }

    #[test]
    fn gmf_object_roundtrips_json() {
        let obj = GmfObject::new("z-1", GmfObjectType::Lance);
        let json = serde_json::to_string(&obj).unwrap();
        let back: GmfObject = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "z-1");
        assert_eq!(back.object_type, GmfObjectType::Lance);
    }

    #[test]
    fn all_variants_have_non_empty_as_str() {
        let variants = [
            GmfObjectType::MechPart, GmfObjectType::ManufacturingZone, GmfObjectType::Socket,
            GmfObjectType::MechAssembly, GmfObjectType::CircuitBreaker, GmfObjectType::HealthCheck,
            GmfObjectType::Pilot, GmfObjectType::Cockpit, GmfObjectType::Battery,
            GmfObjectType::EdenGridNode, GmfObjectType::WaterSystem, GmfObjectType::FarmOutput,
            GmfObjectType::HospitalUnit, GmfObjectType::FactoryAvailability,
            GmfObjectType::MissionContract, GmfObjectType::Lance, GmfObjectType::RepairQueue,
            GmfObjectType::SalvageItem,
        ];
        for v in &variants {
            assert!(!v.as_str().is_empty(), "{:?} has empty as_str", v);
        }
    }
}
