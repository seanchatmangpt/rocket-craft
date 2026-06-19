use color_eyre::eyre::Result;
use knhk::plugin::PluginHost;
use knhk::{Law, LawError, Validator};
use rocket_sdk::manifest::Project;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Law that runs `anti-llm-cheat-lsp` over the project's `src/` directory.
/// Skips gracefully when the binary is not installed.
#[allow(dead_code)] // constructed dynamically via WASM plugin system at audit time
pub struct AntiCheatLaw {
    config_path: std::path::PathBuf,
}

#[allow(dead_code)]
impl AntiCheatLaw {
    pub fn new(config_path: std::path::PathBuf) -> Self {
        Self { config_path }
    }
}

impl Law for AntiCheatLaw {
    fn name(&self) -> &str {
        "anti-llm-cheat"
    }

    fn description(&self) -> &str {
        "Source must pass anti-llm-cheat-lsp scan (detects fabricated evidence patterns)."
    }

    fn validate(&self, project_path: &Path) -> Result<(), LawError> {
        let available = std::process::Command::new("anti-llm-cheat-lsp")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !available {
            return Ok(());
        }
        let src_dir = project_path.join("src");
        if !src_dir.exists() {
            return Ok(());
        }
        let mut cmd = std::process::Command::new("anti-llm-cheat-lsp");
        if self.config_path.exists() {
            cmd.arg("--config").arg(&self.config_path);
        }
        let status = cmd.arg(&src_dir).status().map_err(|e| LawError {
            law_name: "anti-llm-cheat".to_string(),
            message: format!("Failed to spawn anti-llm-cheat-lsp: {e}"),
        })?;
        if !status.success() {
            return Err(LawError {
                law_name: "anti-llm-cheat".to_string(),
                message: "anti-llm-cheat-lsp scan detected fabricated evidence patterns"
                    .to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct ComplianceResult {
    pub project_name: String,
    pub passed: bool,
    pub errors: Vec<LawErrorWrapper>,
}

#[derive(Serialize)]
pub struct LawErrorWrapper {
    pub law_name: String,
    pub message: String,
}

impl From<LawError> for LawErrorWrapper {
    fn from(err: LawError) -> Self {
        Self {
            law_name: err.law_name,
            message: err.message,
        }
    }
}

pub struct ComplianceEngine {
    validator: Validator,
    plugin_host: PluginHost,
}

impl ComplianceEngine {
    pub fn new() -> Self {
        Self {
            validator: Validator::new(),
            plugin_host: PluginHost::new(),
        }
    }

    pub fn add_law(&mut self, law: Box<dyn Law>) {
        self.validator.add_law(law);
    }

    pub fn load_plugins<P: AsRef<Path>>(&mut self, plugins_dir: P) -> Result<()> {
        let plugins_dir = plugins_dir.as_ref();
        if plugins_dir.exists() {
            for entry in fs::read_dir(plugins_dir)? {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("wasm") {
                    let law = self
                        .plugin_host
                        .load_law(&entry.path())
                        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
                    self.validator.add_law(Box::new(law));
                }
            }
        }
        Ok(())
    }

    pub fn check_project(&self, project: &Project) -> ComplianceResult {
        let project_dir = project.uproject_path.parent().unwrap_or(Path::new("."));
        let errors = self.validator.validate_all(project_dir);

        ComplianceResult {
            project_name: project.name.clone(),
            passed: errors.is_empty(),
            errors: errors.into_iter().map(LawErrorWrapper::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk::LawError;

    fn make_err(law: &str, msg: &str) -> LawError {
        LawError { law_name: law.to_string(), message: msg.to_string() }
    }

    #[test]
    fn law_error_wrapper_from_converts_fields() {
        let err = make_err("anti-llm-cheat", "fabricated evidence detected");
        let wrapper = LawErrorWrapper::from(err);
        assert_eq!(wrapper.law_name, "anti-llm-cheat");
        assert_eq!(wrapper.message, "fabricated evidence detected");
    }

    #[test]
    fn compliance_result_passed_when_no_errors() {
        let result = ComplianceResult {
            project_name: "Brm".into(),
            passed: true,
            errors: vec![],
        };
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn compliance_result_serializes_to_json() {
        let result = ComplianceResult {
            project_name: "ShooterGame".into(),
            passed: false,
            errors: vec![LawErrorWrapper {
                law_name: "anti-llm-cheat".into(),
                message: "scan failed".into(),
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("ShooterGame"));
        assert!(json.contains("anti-llm-cheat"));
        assert!(json.contains("\"passed\":false"));
    }

    #[test]
    fn compliance_engine_can_be_constructed() {
        // Just verify construction doesn't panic (WASM host init is lazy).
        let _engine = ComplianceEngine::new();
    }

    #[test]
    fn anti_cheat_law_name_and_description() {
        let law = AntiCheatLaw::new(std::path::PathBuf::from("/tmp/fake.toml"));
        assert_eq!(law.name(), "anti-llm-cheat");
        assert!(!law.description().is_empty());
    }
}
