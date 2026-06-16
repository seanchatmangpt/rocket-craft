use crate::sections::{WorkspaceConfig, CodegenConfig, LspConfig, CliConfig, TestConfig, OtelConfig, RdfConfig};
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
