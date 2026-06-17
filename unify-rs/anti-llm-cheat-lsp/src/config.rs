use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct AntiLlmConfig {
    #[serde(default)]
    pub claim: ClaimConfig,
    #[serde(default)]
    pub surface: SurfaceConfig,
    #[serde(default)]
    pub test: TestConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct ClaimConfig {
    #[serde(default)]
    pub domain_terms: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SurfaceConfig {
    #[serde(default)]
    pub non_blocking_path_prefixes: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TestConfig {
    #[serde(default)]
    pub structural_check_paths: Vec<String>,
}

impl AntiLlmConfig {
    pub fn load_from_dir(dirpath: &str) -> Self {
        let config_path = Path::new(dirpath).join("anti-llm.toml");
        if !config_path.is_file() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn surface_is_non_blocking(&self, file_path: &str) -> bool {
        self.surface
            .non_blocking_path_prefixes
            .iter()
            .any(|prefix| file_path.contains(prefix.as_str()))
    }

    pub fn test_is_structural_path(&self, file_path: &str) -> bool {
        self.test
            .structural_check_paths
            .iter()
            .any(|prefix| file_path.contains(prefix.as_str()))
    }
}
