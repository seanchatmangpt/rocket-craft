use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::RocketError;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RocketConfig {
    pub ue4_root: Option<PathBuf>,
}

impl RocketConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = PathBuf::from(".rocket.json");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(RocketError::Io)?;
            let config: Self = serde_json::from_str(&content).map_err(RocketError::Json)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = PathBuf::from(".rocket.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
