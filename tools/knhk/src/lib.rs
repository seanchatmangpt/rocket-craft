use std::path::Path;

pub mod plugin;
pub mod receipt;
pub use receipt::Receipt;

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

                let is_keystore = entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "keystore" || ext == "jks");
                if is_keystore {
                    has_keystore = true;
                    break;
                }
            }

            if !has_keystore {
                return Err(LawError {
                    law_name: self.name().to_string(),
                    message: "Android target detected but no .keystore or .jks file found."
                        .to_string(),
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

    struct AlwaysPassLaw;
    impl Law for AlwaysPassLaw {
        fn name(&self) -> &str { "AlwaysPass" }
        fn description(&self) -> &str { "Never fails." }
        fn validate(&self, _: &Path) -> Result<(), LawError> { Ok(()) }
    }

    struct AlwaysFailLaw;
    impl Law for AlwaysFailLaw {
        fn name(&self) -> &str { "AlwaysFail" }
        fn description(&self) -> &str { "Always fails." }
        fn validate(&self, _: &Path) -> Result<(), LawError> {
            Err(LawError { law_name: "AlwaysFail".into(), message: "intentional".into() })
        }
    }

    #[test]
    fn plugin_host_new_empty() {
        let host = PluginHost::new();
        assert!(host.receipts().is_empty());
    }

    #[test]
    fn plugin_host_record_and_retrieve_receipts() {
        let mut host = PluginHost::new();
        host.record_receipt(Receipt::new("law1.wasm".into(), "Law1", true, "ok"));
        host.record_receipt(Receipt::new("law2.wasm".into(), "Law2", false, "fail"));
        assert_eq!(host.receipts().len(), 2);
        assert_eq!(host.receipts()[0].law_name, "Law1");
        assert!(host.receipts()[0].passed);
        assert_eq!(host.receipts()[1].law_name, "Law2");
        assert!(!host.receipts()[1].passed);
    }

    #[test]
    fn law_error_display_format() {
        let e = LawError { law_name: "MyLaw".into(), message: "broken".into() };
        let s = format!("{}", e);
        assert!(s.contains("MyLaw"));
        assert!(s.contains("broken"));
    }

    #[test]
    fn validator_empty_returns_no_errors() {
        let v = Validator::new();
        let errors = v.validate_all(Path::new("/tmp"));
        assert!(errors.is_empty());
    }

    #[test]
    fn validator_passing_law_returns_no_errors() {
        let mut v = Validator::new();
        v.add_law(Box::new(AlwaysPassLaw));
        assert!(v.validate_all(Path::new("/tmp")).is_empty());
    }

    #[test]
    fn validator_failing_law_returns_one_error() {
        let mut v = Validator::new();
        v.add_law(Box::new(AlwaysFailLaw));
        let errors = v.validate_all(Path::new("/tmp"));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].law_name, "AlwaysFail");
    }

    #[test]
    fn validator_collects_all_errors_from_multiple_laws() {
        let mut v = Validator::new();
        v.add_law(Box::new(AlwaysPassLaw));
        v.add_law(Box::new(AlwaysFailLaw));
        v.add_law(Box::new(AlwaysFailLaw));
        let errors = v.validate_all(Path::new("/tmp"));
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn android_keystore_law_name_and_description() {
        let law = AndroidKeystoreLaw;
        assert_eq!(law.name(), "AndroidKeystoreLaw");
        assert!(!law.description().is_empty());
    }

    #[test]
    fn android_keystore_law_passes_on_nonexistent_project() {
        let law = AndroidKeystoreLaw;
        // A nonexistent path has no Android directory → law passes
        assert!(law.validate(Path::new("/tmp/no_such_project_xyz")).is_ok());
    }
}
