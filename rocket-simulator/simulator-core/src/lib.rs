use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::Result;
use blake3::Hasher;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RocketContract {
    pub name: String,
    pub world_seed: u64,
    pub es3_enabled: bool,
    pub target_platform: String,
    pub features: Vec<String>,
}

impl RocketContract {
    pub fn new(name: String, world_seed: u64) -> Self {
        Self {
            name,
            world_seed,
            es3_enabled: true,
            target_platform: "HTML5".to_string(),
            features: vec!["WASM".to_string(), "WebGL2".to_string()],
        }
    }

    pub fn compute_hash(&self) -> String {
        let mut hasher = Hasher::new();
        hasher.update(self.name.as_bytes());
        hasher.update(&self.world_seed.to_le_bytes());
        hasher.update(&[self.es3_enabled as u8]);
        hasher.update(self.target_platform.as_bytes());
        for feature in &self.features {
            hasher.update(feature.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    pub fn generate_artifact(&self, output_dir: &Path) -> Result<PathBuf> {
        let hash = self.compute_hash();
        let artifact_name = format!("{}_{}.json", self.name, hash);
        let artifact_path = output_dir.join(artifact_name);
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&artifact_path, content)?;
        
        Ok(artifact_path)
    }
}

pub struct SimulationEngine {
    pub contract: RocketContract,
    pub workspace_root: PathBuf,
}

impl SimulationEngine {
    pub fn new(contract: RocketContract, workspace_root: PathBuf) -> Self {
        Self {
            contract,
            workspace_root,
        }
    }

    pub fn prepare_manufacturing(&self) -> Result<()> {
        let artifact_dir = self.workspace_root.join(".ggen").join("artifacts");
        std::fs::create_dir_all(&artifact_dir)?;
        
        let artifact_path = self.contract.generate_artifact(&artifact_dir)?;
        tracing::info!("Generated UE4 world artifact: {}", artifact_path.display());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rocket_contract_hash() {
        let contract = RocketContract::new("test_rocket".to_string(), 42);
        let hash = contract.compute_hash();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_generate_artifact() {
        let dir = tempdir().unwrap();
        let contract = RocketContract::new("test_artifact".to_string(), 123);
        let path = contract.generate_artifact(dir.path()).unwrap();
        assert!(path.exists());
        
        let content = std::fs::read_to_string(path).unwrap();
        let loaded: RocketContract = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.name, contract.name);
        assert_eq!(loaded.world_seed, contract.world_seed);
    }
}

