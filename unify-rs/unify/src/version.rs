/// Version metadata for a single crate in the unify-rs workspace.
pub struct VersionInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Returns version info for all 20 crates in the unify-rs workspace.
pub fn crate_versions() -> Vec<VersionInfo> {
    vec![
        VersionInfo {
            name: "unify",
            version: env!("CARGO_PKG_VERSION"),
        },
        VersionInfo {
            name: "unify-core",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-sem",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-admission",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-receipts",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-macros",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-cli",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-rdf",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-lsp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-test",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-pm",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-codegen",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-otel",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-ocel",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-wasm",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-ffi",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-config",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-mcp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-bp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-integration-tests",
            version: "0.1.0",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_versions_returns_20_entries() {
        let versions = crate_versions();
        assert_eq!(versions.len(), 20);
    }

    #[test]
    fn all_entries_have_non_empty_name_and_version() {
        for v in crate_versions() {
            assert!(!v.name.is_empty(), "entry has empty name");
            assert!(!v.version.is_empty(), "entry '{}' has empty version", v.name);
        }
    }

    #[test]
    fn first_entry_is_unify_crate() {
        let v = &crate_versions()[0];
        assert_eq!(v.name, "unify");
    }

    #[test]
    fn all_names_are_unique() {
        let versions = crate_versions();
        let names: std::collections::HashSet<&str> =
            versions.iter().map(|v| v.name).collect();
        assert_eq!(names.len(), versions.len(), "duplicate crate name detected");
    }

    #[test]
    fn versions_follow_semver_pattern() {
        // All versions should be parseable as "major.minor.patch"
        for v in crate_versions() {
            let parts: Vec<&str> = v.version.split('.').collect();
            assert_eq!(parts.len(), 3, "version '{}' is not semver for '{}'", v.version, v.name);
            for part in &parts {
                assert!(part.parse::<u32>().is_ok(),
                    "non-numeric semver component '{}' in '{}'", part, v.name);
            }
        }
    }
}
