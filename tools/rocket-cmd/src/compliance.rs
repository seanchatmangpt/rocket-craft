use std::path::Path;
use std::fs;
use knhk::{Law, LawError, Validator};
use knhk::plugin::PluginHost;
use crate::manifest::Project;
use serde::Serialize;
use color_eyre::eyre::Result;

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
                    let law = self.plugin_host.load_law(&entry.path()).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
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
