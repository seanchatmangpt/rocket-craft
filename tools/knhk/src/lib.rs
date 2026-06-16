use std::path::Path;

pub mod plugin;

/// A trait representing a semantic law or constraint that a project must satisfy.
///
/// Laws are used to enforce architectural standards, security requirements, or
/// project-specific rules across the codebase.
pub trait Law {
    /// Returns the unique name of the law.
    fn name(&self) -> &str;
    /// Returns a brief description of what the law enforces.
    fn description(&self) -> &str;
    /// Validates the given project against this law.
    ///
    /// # Errors
    ///
    /// Returns a `LawError` if the project violates the law.
    fn validate(&self, project_path: &Path) -> Result<(), LawError>;
}

/// Represents a violation of a `Law`.
#[derive(Debug)]
pub struct LawError {
    /// The name of the law that was violated.
    pub law_name: String,
    /// A detailed message describing the violation.
    pub message: String,
}

impl std::fmt::Display for LawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Law '{}' violated: {}", self.law_name, self.message)
    }
}

/// A registry and executor for multiple `Law` implementations.
pub struct Validator {
    laws: Vec<Box<dyn Law>>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    /// Creates a new, empty `Validator`.
    pub fn new() -> Self {
        Self { laws: Vec::new() }
    }

    /// Adds a new law to the validator's registry.
    pub fn add_law(&mut self, law: Box<dyn Law>) {
        self.laws.push(law);
    }

    /// Validates a project against all registered laws.
    ///
    /// Returns a vector of `LawError`s for all violations found.
    pub fn validate_all(&self, project_path: &Path) -> Vec<LawError> {
        let mut errors = Vec::new();
        for law in &self.laws {
            if let Err(err) = law.validate(project_path) {
                errors.push(err);
            }
        }
        errors
    }
}

/// A law that ensures Android-targeting projects have a valid keystore.
pub struct AndroidKeystoreLaw;

impl Law for AndroidKeystoreLaw {
    fn name(&self) -> &str {
        "AndroidKeystoreLaw"
    }

    fn description(&self) -> &str {
        "Every project must have a keystore for Android if it targets Android."
    }

    fn validate(&self, project_path: &Path) -> Result<(), LawError> {
        // Check if there are any Android-related files or configurations
        // For simplicity, let's check if there's an 'Android' directory in any project version
        // or a DefaultEngine.ini that mentions Android.
        
        // In this workspace, Android keystores are expected if there's an Android platform folder.
        // Let's check for keystore files (*.keystore or *.jks)
        
        let mut has_android = false;
        
        // Search for Android platform directories
        let walker = ignore::WalkBuilder::new(project_path).build();
        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            if entry.path().is_dir() && entry.path().to_string_lossy().contains("Android") {
                has_android = true;
                break;
            }
        }

        if has_android {
            let mut has_keystore = false;
            let walker = ignore::WalkBuilder::new(project_path)
                .standard_filters(false)
                .build();
            for entry in walker {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                
                let is_keystore = entry.path().extension().is_some_and(|ext| ext == "keystore" || ext == "jks");
                if is_keystore {
                    has_keystore = true;
                    break;
                }
            }
            
            if !has_keystore {
                return Err(LawError {
                    law_name: self.name().to_string(),
                    message: "Android target detected but no .keystore or .jks file found.".to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::plugin::PluginHost;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_plugin_host_new() {
        let host = PluginHost::new();
        assert_eq!(host.receipts().len(), 0);
    }
}
