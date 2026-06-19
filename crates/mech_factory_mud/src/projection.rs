use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionRow {
    pub projection_id: String,
    pub object_id: String,
    pub station_id: String,
    pub route_node_id: String,
    pub source_process_step: String,
    pub source_receipt: String,
    pub authority_inputs: String,
    pub lod_class: u8,
    pub projection_type: String,
    pub ue4_target_surface: String,
    pub admission_status: String,
}
