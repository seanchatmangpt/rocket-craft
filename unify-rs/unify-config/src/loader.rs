use crate::{ConfigError, UnifyManifest};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Toml,
    Json,
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn from_file(path: impl AsRef<Path>) -> Result<UnifyManifest, ConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let format = if path.extension().and_then(|e| e.to_str()) == Some("json") {
            ConfigFormat::Json
        } else {
            ConfigFormat::Toml
        };
        Self::from_str(&content, format)
    }

    pub fn from_str(s: &str, format: ConfigFormat) -> Result<UnifyManifest, ConfigError> {
        match format {
            ConfigFormat::Toml => UnifyManifest::from_toml(s),
            ConfigFormat::Json => UnifyManifest::from_json(s),
        }
    }

    pub fn from_env() -> UnifyManifest {
        let mut manifest = UnifyManifest::default();
        if let Ok(name) = std::env::var("UNIFY_NAME") {
            manifest.name = name;
        }
        if let Ok(version) = std::env::var("UNIFY_VERSION") {
            manifest.version = version;
        }
        if let Ok(root) = std::env::var("UNIFY_WORKSPACE_ROOT") {
            manifest.workspace.root = root;
        }
        if let Ok(target) = std::env::var("UNIFY_WORKSPACE_DEFAULT_TARGET") {
            manifest.workspace.default_target = target;
        }
        manifest
    }

    pub fn find_and_load() -> Result<UnifyManifest, ConfigError> {
        let candidates = [
            ("./unify.toml", ConfigFormat::Toml),
            ("./unify.json", ConfigFormat::Json),
            ("./.unify/config.toml", ConfigFormat::Toml),
        ];
        for (path, format) in &candidates {
            let p = Path::new(path);
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                return Self::from_str(&content, *format);
            }
        }
        Err(ConfigError::Parse(
            "No unify configuration file found (tried unify.toml, unify.json, .unify/config.toml)"
                .to_string(),
        ))
    }
}
