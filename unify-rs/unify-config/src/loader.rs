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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(ext: &str, content: &str) -> tempfile::NamedTempFile {
        let f = tempfile::Builder::new().suffix(ext).tempfile().unwrap();
        let mut f2 = f.reopen().unwrap();
        f2.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn from_str_toml_parses_name() {
        let toml = "name = \"test-proj\"\nversion = \"1.0.0\"\n\n[workspace]\nroot = \"\"\nmembers = []\ndefault_target = \"\"\n";
        let m = ConfigLoader::from_str(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(m.name, "test-proj");
    }

    #[test]
    fn from_str_json_parses_name() {
        let json = r#"{"name":"json-proj","version":"2.0.0","workspace":{"root":"","members":[],"default_target":""}}"#;
        let m = ConfigLoader::from_str(json, ConfigFormat::Json).unwrap();
        assert_eq!(m.name, "json-proj");
    }

    #[test]
    fn from_str_toml_error_on_invalid() {
        assert!(ConfigLoader::from_str("[[[[bad", ConfigFormat::Toml).is_err());
    }

    #[test]
    fn from_file_json_extension_dispatches_correctly() {
        let tmp = write_tmp(".json", r#"{"name":"file-json","version":"1.0.0","workspace":{"root":"","members":[],"default_target":""}}"#);
        let m = ConfigLoader::from_file(tmp.path()).unwrap();
        assert_eq!(m.name, "file-json");
    }

    #[test]
    fn from_file_toml_extension_dispatches_correctly() {
        let tmp = write_tmp(".toml", "name = \"file-toml\"\nversion = \"0.1.0\"\n\n[workspace]\nroot = \"\"\nmembers = []\ndefault_target = \"\"\n");
        let m = ConfigLoader::from_file(tmp.path()).unwrap();
        assert_eq!(m.name, "file-toml");
    }

    #[test]
    fn from_file_nonexistent_returns_error() {
        assert!(ConfigLoader::from_file("/tmp/no_such_file_xyz.toml").is_err());
    }

    #[test]
    fn from_env_applies_env_vars() {
        // Use unique env var names to avoid collisions with parallel tests
        std::env::set_var("UNIFY_NAME", "env-project-loader-test");
        std::env::set_var("UNIFY_VERSION", "3.0.0-loader-test");
        let m = ConfigLoader::from_env();
        std::env::remove_var("UNIFY_NAME");
        std::env::remove_var("UNIFY_VERSION");
        assert_eq!(m.name, "env-project-loader-test");
        assert_eq!(m.version, "3.0.0-loader-test");
    }
}
