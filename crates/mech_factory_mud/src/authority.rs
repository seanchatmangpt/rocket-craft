use crate::generated_constants::{
    MAX_DAMAGE_CLASS, MAX_GRIP_CLASS, MAX_HEAT_CLASS, MAX_LOD_CLASS, MAX_PROJECTION_STATE_CLASS,
    MAX_RECEIPT_STATE_CLASS, MAX_SOCKET_HEALTH_CLASS, MAX_STATION_STATE_CLASS, MAX_STRESS_CLASS,
    MAX_WALKTHROUGH_STATE_CLASS,
};

#[derive(Debug, Clone, Default)]
pub struct AuthorityState {
    pub damage_class: u8,
    pub heat_class: u8,
    pub stress_class: u8,
    pub grip_class: u8,
    pub socket_health_class: u8,
    pub lod_class: u8,
    pub walkthrough_state_class: u8,
    pub station_state_class: u8,
    pub projection_state_class: u8,
    pub receipt_state_class: u8,
}

impl AuthorityState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate all 10 authority byte fields against ontology-generated bounds.
    /// Hardcoded literals are forbidden. All bounds come from generated_constants.rs.
    pub fn validate_classes(&self) -> bool {
        self.damage_class <= MAX_DAMAGE_CLASS
            && self.heat_class <= MAX_HEAT_CLASS
            && self.stress_class <= MAX_STRESS_CLASS
            && self.grip_class <= MAX_GRIP_CLASS
            && self.socket_health_class <= MAX_SOCKET_HEALTH_CLASS
            && self.lod_class <= MAX_LOD_CLASS
            && self.walkthrough_state_class <= MAX_WALKTHROUGH_STATE_CLASS
            && self.station_state_class <= MAX_STATION_STATE_CLASS
            && self.projection_state_class <= MAX_PROJECTION_STATE_CLASS
            && self.receipt_state_class <= MAX_RECEIPT_STATE_CLASS
    }
}
