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

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_surface_prefix(prefix: &str) -> AntiLlmConfig {
        AntiLlmConfig {
            surface: SurfaceConfig { non_blocking_path_prefixes: vec![prefix.into()] },
            ..Default::default()
        }
    }

    fn config_with_structural_path(path: &str) -> AntiLlmConfig {
        AntiLlmConfig {
            test: TestConfig { structural_check_paths: vec![path.into()] },
            ..Default::default()
        }
    }

    #[test]
    fn default_config_surface_not_blocking_for_any_path() {
        let cfg = AntiLlmConfig::default();
        assert!(!cfg.surface_is_non_blocking("src/lib.rs"));
        assert!(!cfg.surface_is_non_blocking("docs/changelog.md"));
    }

    #[test]
    fn surface_is_non_blocking_matches_prefix() {
        let cfg = config_with_surface_prefix("docs/");
        assert!(cfg.surface_is_non_blocking("docs/changelog.md"));
    }

    #[test]
    fn surface_is_non_blocking_false_when_path_does_not_contain_prefix() {
        let cfg = config_with_surface_prefix("docs/");
        assert!(!cfg.surface_is_non_blocking("src/lib.rs"));
    }

    #[test]
    fn test_is_structural_path_matches_configured_paths() {
        let cfg = config_with_structural_path("tests/structural");
        assert!(cfg.test_is_structural_path("tests/structural/foo.rs"));
    }

    #[test]
    fn test_is_structural_path_false_when_not_configured() {
        let cfg = AntiLlmConfig::default();
        assert!(!cfg.test_is_structural_path("tests/structural/foo.rs"));
    }

    #[test]
    fn load_from_nonexistent_dir_returns_default() {
        let cfg = AntiLlmConfig::load_from_dir("/nonexistent/path/that/does/not/exist");
        assert!(cfg.surface.non_blocking_path_prefixes.is_empty());
    }
}
