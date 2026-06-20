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
                format!(
                    "version '{}' does not look like a semver string (expected X.Y.Z)",
                    manifest.version
                ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::UnifyManifest;
    use crate::sections::{LspConfig, TestConfig};

    fn valid_manifest() -> UnifyManifest {
        UnifyManifest::default_for("my-project")
    }

    // ── is_semver_like ────────────────────────────────────────────────────────

    #[test]
    fn semver_major_minor_patch_valid() {
        assert!(ManifestValidator::is_semver_like("1.0.0"));
        assert!(ManifestValidator::is_semver_like("12.3.456"));
    }

    #[test]
    fn semver_major_minor_valid() {
        assert!(ManifestValidator::is_semver_like("1.0"));
    }

    #[test]
    fn semver_single_part_invalid() {
        assert!(!ManifestValidator::is_semver_like("1"));
    }

    #[test]
    fn semver_with_prerelease_suffix_valid() {
        assert!(ManifestValidator::is_semver_like("1.0.0-alpha"));
    }

    #[test]
    fn semver_empty_string_invalid() {
        assert!(!ManifestValidator::is_semver_like(""));
    }

    // ── validate / is_valid ───────────────────────────────────────────────────

    #[test]
    fn valid_manifest_has_no_violations() {
        let m = valid_manifest();
        assert!(ManifestValidator::is_valid(&m));
        assert!(ManifestValidator::validate(&m).is_empty());
    }

    #[test]
    fn empty_name_produces_violation() {
        let mut m = valid_manifest();
        m.name = "".into();
        let v = ManifestValidator::validate(&m);
        assert!(v.iter().any(|vi| vi.field == "name"));
    }

    #[test]
    fn whitespace_only_name_produces_violation() {
        let mut m = valid_manifest();
        m.name = "   ".into();
        let v = ManifestValidator::validate(&m);
        assert!(v.iter().any(|vi| vi.field == "name"));
    }

    #[test]
    fn bad_version_produces_violation() {
        let mut m = valid_manifest();
        m.version = "not-a-version".into();
        let v = ManifestValidator::validate(&m);
        assert!(v.iter().any(|vi| vi.field == "version"));
    }

    #[test]
    fn coverage_threshold_over_one_produces_violation() {
        let mut m = valid_manifest();
        m.test = Some(TestConfig { coverage_threshold: 1.5, ..Default::default() });
        let v = ManifestValidator::validate(&m);
        assert!(v.iter().any(|vi| vi.field == "test.coverage_threshold"));
    }

    #[test]
    fn coverage_threshold_zero_is_valid() {
        let mut m = valid_manifest();
        m.test = Some(TestConfig { coverage_threshold: 0.0, ..Default::default() });
        assert!(ManifestValidator::is_valid(&m));
    }

    #[test]
    fn lsp_conformance_threshold_over_one_produces_violation() {
        let mut m = valid_manifest();
        m.lsp = Some(LspConfig { conformance_threshold: 2.0, ..Default::default() });
        let v = ManifestValidator::validate(&m);
        assert!(v.iter().any(|vi| vi.field == "lsp.conformance_threshold"));
    }

    #[test]
    fn lsp_conformance_threshold_one_is_valid() {
        let mut m = valid_manifest();
        m.lsp = Some(LspConfig { conformance_threshold: 1.0, ..Default::default() });
        assert!(ManifestValidator::is_valid(&m));
    }
}
