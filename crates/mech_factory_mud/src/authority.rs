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
    pub fn validate_classes(&self) -> bool {
        self.damage_class <= 15 && self.heat_class <= 15
    }
}
