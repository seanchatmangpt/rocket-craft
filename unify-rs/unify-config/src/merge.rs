use crate::loader::ConfigLoader;
use crate::sections::WorkspaceConfig;
use crate::{ConfigError, UnifyManifest};

pub struct ConfigMerge;

impl ConfigMerge {
    /// Apply overrides on top of base. None fields in override are skipped.
    pub fn merge(base: UnifyManifest, overrides: UnifyManifest) -> UnifyManifest {
        UnifyManifest {
            name: if overrides.name.is_empty() {
                base.name
            } else {
                overrides.name
            },
            version: if overrides.version.is_empty() {
                base.version
            } else {
                overrides.version
            },
            workspace: merge_workspace(base.workspace, overrides.workspace),
            codegen: overrides.codegen.or(base.codegen),
            lsp: overrides.lsp.or(base.lsp),
            cli: overrides.cli.or(base.cli),
            test: overrides.test.or(base.test),
            otel: overrides.otel.or(base.otel),
            rdf: overrides.rdf.or(base.rdf),
        }
    }

    /// Load a base config and apply an overrides file on top.
    pub fn load_with_overrides(
        base_path: &str,
        override_path: Option<&str>,
    ) -> Result<UnifyManifest, ConfigError> {
        let base = ConfigLoader::from_file(base_path)?;
        match override_path {
            None => Ok(base),
            Some(path) => {
                let overrides = ConfigLoader::from_file(path)?;
                Ok(Self::merge(base, overrides))
            }
        }
    }
}

fn merge_workspace(base: WorkspaceConfig, overrides: WorkspaceConfig) -> WorkspaceConfig {
    WorkspaceConfig {
        root: if overrides.root.is_empty() {
            base.root
        } else {
            overrides.root
        },
        members: if overrides.members.is_empty() {
            base.members
        } else {
            overrides.members
        },
        default_target: if overrides.default_target.is_empty() {
            base.default_target
        } else {
            overrides.default_target
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> UnifyManifest {
        let mut m = UnifyManifest::default_for("base");
        m.version = "1.0.0".into();
        m
    }

    fn overrides_empty() -> UnifyManifest {
        let mut m = UnifyManifest::default_for("");
        m.version = String::new(); // force empty so merge falls back to base
        m
    }

    #[test]
    fn empty_override_name_keeps_base_name() {
        let merged = ConfigMerge::merge(base(), overrides_empty());
        assert_eq!(merged.name, "base");
    }

    #[test]
    fn non_empty_override_name_wins() {
        let mut ov = overrides_empty();
        ov.name = "override_project".into();
        let merged = ConfigMerge::merge(base(), ov);
        assert_eq!(merged.name, "override_project");
    }

    #[test]
    fn empty_override_version_keeps_base_version() {
        let merged = ConfigMerge::merge(base(), overrides_empty());
        assert_eq!(merged.version, "1.0.0");
    }

    #[test]
    fn override_version_wins() {
        let mut ov = overrides_empty();
        ov.version = "2.0.0".into();
        let merged = ConfigMerge::merge(base(), ov);
        assert_eq!(merged.version, "2.0.0");
    }

    #[test]
    fn none_option_in_override_keeps_base_option() {
        use crate::sections::TestConfig;
        let mut b = base();
        b.test = Some(TestConfig { coverage_threshold: 0.8, ..Default::default() });
        let merged = ConfigMerge::merge(b, overrides_empty());
        assert!((merged.test.unwrap().coverage_threshold - 0.8).abs() < 1e-9);
    }

    #[test]
    fn some_option_in_override_wins() {
        use crate::sections::TestConfig;
        let mut b = base();
        b.test = Some(TestConfig { coverage_threshold: 0.5, ..Default::default() });
        let mut ov = overrides_empty();
        ov.test = Some(TestConfig { coverage_threshold: 0.9, ..Default::default() });
        let merged = ConfigMerge::merge(b, ov);
        assert!((merged.test.unwrap().coverage_threshold - 0.9).abs() < 1e-9);
    }
}
