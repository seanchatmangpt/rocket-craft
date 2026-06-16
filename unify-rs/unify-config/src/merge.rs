use crate::{ConfigError, UnifyManifest};
use crate::loader::ConfigLoader;
use crate::sections::WorkspaceConfig;

pub struct ConfigMerge;

impl ConfigMerge {
    /// Apply overrides on top of base. None fields in override are skipped.
    pub fn merge(base: UnifyManifest, overrides: UnifyManifest) -> UnifyManifest {
        UnifyManifest {
            name: if overrides.name.is_empty() { base.name } else { overrides.name },
            version: if overrides.version.is_empty() { base.version } else { overrides.version },
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
        root: if overrides.root.is_empty() { base.root } else { overrides.root },
        members: if overrides.members.is_empty() { base.members } else { overrides.members },
        default_target: if overrides.default_target.is_empty() {
            base.default_target
        } else {
            overrides.default_target
        },
    }
}
