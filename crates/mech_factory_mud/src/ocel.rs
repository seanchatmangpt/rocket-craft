use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelData {
    pub objects: Vec<String>,
    pub events: Vec<String>,
}
