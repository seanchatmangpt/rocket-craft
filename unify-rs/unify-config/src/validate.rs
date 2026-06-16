use crate::UnifyManifest;

#[derive(Debug, Clone)]
pub struct ManifestViolation {
    pub field: String,
    pub message: String,
}

impl ManifestViolation {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

pub struct ManifestValidator;

impl ManifestValidator {
    pub fn validate(manifest: &UnifyManifest) -> Vec<ManifestViolation> {
        let mut violations = Vec::new();

        // name must be non-empty
        if manifest.name.trim().is_empty() {
            violations.push(ManifestViolation::new("name", "name must not be empty"));
        }

        // version must be semver-like (digits.digits.digits)
        if !Self::is_semver_like(&manifest.version) {
            violations.push(ManifestViolation::new(
                "version",
                format!("version '{}' does not look like a semver string (expected X.Y.Z)", manifest.version),
            ));
        }

        // test coverage_threshold must be in [0.0, 1.0]
        if let Some(test) = &manifest.test {
            if !(0.0..=1.0).contains(&test.coverage_threshold) {
                violations.push(ManifestViolation::new(
                    "test.coverage_threshold",
                    format!(
                        "coverage_threshold {} is outside valid range [0.0, 1.0]",
                        test.coverage_threshold
                    ),
                ));
            }
        }

        // lsp conformance_threshold must be in [0.0, 1.0]
        if let Some(lsp) = &manifest.lsp {
            if !(0.0..=1.0).contains(&lsp.conformance_threshold) {
                violations.push(ManifestViolation::new(
                    "lsp.conformance_threshold",
                    format!(
                        "conformance_threshold {} is outside valid range [0.0, 1.0]",
                        lsp.conformance_threshold
                    ),
                ));
            }
        }

        violations
    }

    pub fn is_valid(manifest: &UnifyManifest) -> bool {
        Self::validate(manifest).is_empty()
    }

    fn is_semver_like(v: &str) -> bool {
        if v.trim().is_empty() {
            return false;
        }
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() < 2 {
            return false;
        }
        parts.iter().all(|p| {
            // strip optional pre-release/build suffix on the last part
            let core = p.split('-').next().unwrap_or(p);
            let core = core.split('+').next().unwrap_or(core);
            !core.is_empty() && core.chars().all(|c| c.is_ascii_digit())
        })
    }
}
