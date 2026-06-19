use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelData {
    pub objects: Vec<String>,
    pub events: Vec<String>,
}
