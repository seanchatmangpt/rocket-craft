use crate::sections::{
    CliConfig, CodegenConfig, LspConfig, OtelConfig, RdfConfig, TestConfig, WorkspaceConfig,
};
use crate::ConfigError;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnifyManifest {
    pub name: String,
    pub version: String,
    pub workspace: WorkspaceConfig,
    pub codegen: Option<CodegenConfig>,
    pub lsp: Option<LspConfig>,
    pub cli: Option<CliConfig>,
    pub test: Option<TestConfig>,
    pub otel: Option<OtelConfig>,
    pub rdf: Option<RdfConfig>,
}

impl UnifyManifest {
    pub fn default_for(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            workspace: WorkspaceConfig::default(),
            codegen: None,
            lsp: None,
            cli: None,
            test: None,
            otel: None,
            rdf: None,
        }
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string(self).map_err(|e| ConfigError::Serialize(e.to_string()))
    }

    pub fn from_toml(s: &str) -> Result<Self, ConfigError> {
        toml::from_str(s).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self).map_err(|e| ConfigError::Serialize(e.to_string()))
    }

    pub fn from_json(s: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(s).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn merge(base: Self, overrides: Self) -> Self {
        crate::merge::ConfigMerge::merge(base, overrides)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_for_sets_name_and_version() {
        let m = UnifyManifest::default_for("my-proj");
        assert_eq!(m.name, "my-proj");
        assert_eq!(m.version, "0.1.0");
    }

    #[test]
    fn default_for_all_optional_sections_are_none() {
        let m = UnifyManifest::default_for("x");
        assert!(m.codegen.is_none());
        assert!(m.lsp.is_none());
        assert!(m.cli.is_none());
        assert!(m.test.is_none());
        assert!(m.otel.is_none());
        assert!(m.rdf.is_none());
    }

    #[test]
    fn to_json_and_from_json_roundtrip() {
        let orig = UnifyManifest::default_for("roundtrip-test");
        let json = orig.to_json().unwrap();
        let back = UnifyManifest::from_json(&json).unwrap();
        assert_eq!(back.name, "roundtrip-test");
        assert_eq!(back.version, "0.1.0");
    }

    #[test]
    fn to_toml_and_from_toml_roundtrip() {
        let orig = UnifyManifest::default_for("toml-test");
        let toml = orig.to_toml().unwrap();
        let back = UnifyManifest::from_toml(&toml).unwrap();
        assert_eq!(back.name, "toml-test");
    }

    #[test]
    fn from_json_error_on_invalid_input() {
        assert!(UnifyManifest::from_json("{not valid json}").is_err());
    }

    #[test]
    fn from_toml_error_on_invalid_input() {
        assert!(UnifyManifest::from_toml("[[[[invalid").is_err());
    }

    #[test]
    fn merge_delegates_to_config_merge() {
        let base = UnifyManifest::default_for("base");
        let mut override_m = UnifyManifest::default_for("");
        override_m.name = "override-name".into();
        let merged = UnifyManifest::merge(base, override_m);
        assert_eq!(merged.name, "override-name");
    }
}
