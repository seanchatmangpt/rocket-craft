use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;
use color_eyre::eyre::{eyre, Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};

/// The Unified Lockfile representing all dependencies across all 7 workspaces.
/// Enforces a single combinatorial boundary for deterministic Rust builds (A = μ(O*)).
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedLock {
    pub version: usize,
    pub law: String,
    pub workspaces: BTreeSet<String>,
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedDependency {
    pub name: String,
    pub version: String,
    pub source: String,
    pub expected_features: BTreeSet<String>,
}

pub struct EcosystemLocker {
    root_path: PathBuf,
}

impl EcosystemLocker {
    /// Binds the locker to the provided absolute path of the rocket-craft repository.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root.into(),
        }
    }

    /// Recursively maps dependencies of the 7 specific local workspaces and enforces a unified `rocket.lock`.
    /// Rejects any divergence immediately (Death of MVP, Bounded Construction).
    pub fn enforce_unified_lock(&self) -> Result<()> {
        let cells = [
            "tools",
            "nexus-engine",
            "blueprint-rs",
            "unify-rs",
            "infinity-blade-4/mud",
            "chicago-tdd-tools",
            "asset-pipeline",
        ];

        let mut unified_deps: BTreeMap<String, LockedDependency> = BTreeMap::new();
        let mut processed_cells = BTreeSet::new();
        let mut violations = Vec::new();

        println!(
            "{}",
            "==> Bounding the Rust Ecosystem (A=μ(O*))".bold().blue()
        );

        for cell in cells.iter() {
            let manifest = self.root_path.join(cell).join("Cargo.toml");
            if !manifest.exists() {
                return Err(eyre!(
                    "Cell {} missing manifest at {}",
                    cell,
                    manifest.display()
                ));
            }

            println!("    {} Mapping cell: {}", "[O*]".cyan(), cell);
            processed_cells.insert(cell.to_string());

            let mut cmd = MetadataCommand::new();
            cmd.manifest_path(&manifest);

            let metadata = cmd
                .exec()
                .wrap_err_with(|| format!("cargo_metadata failed on cell: {}", cell))?;

            if let Some(resolve) = metadata.resolve {
                for node in resolve.nodes {
                    let pkg_id = node.id.clone();
                    if let Some(pkg) = metadata.packages.iter().find(|p| p.id == pkg_id) {
                        let source = pkg
                            .source
                            .as_ref()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "local".to_string());

                        let incoming_dep = LockedDependency {
                            name: pkg.name.clone(),
                            version: pkg.version.to_string(),
                            source: source.clone(),
                            expected_features: node.features.into_iter().collect(),
                        };

                        if let Some(existing) = unified_deps.get(&pkg.name) {
                            if existing.version != incoming_dep.version {
                                violations.push(format!(
                                    "Version divergence in package {}: {} (existing) vs {} (cell {})",
                                    pkg.name, existing.version, incoming_dep.version, cell
                                ));
                            } else {
                                // Combinatorially merge required features across cells
                                let mut merged = existing.clone();
                                merged
                                    .expected_features
                                    .extend(incoming_dep.expected_features);
                                unified_deps.insert(pkg.name.clone(), merged);
                            }
                        } else {
                            unified_deps.insert(pkg.name.clone(), incoming_dep);
                        }
                    }
                }
            } else {
                return Err(eyre!("No resolve graph found for cell {}", cell));
            }
        }

        if !violations.is_empty() {
            println!(
                "{}",
                "DETERMINISM VIOLATION: Ecosystem divergence detected!"
                    .red()
                    .bold()
            );
            for v in violations {
                println!("  - {}", v.yellow());
            }
            return Err(eyre!(
                "Ecosystem boundary broken. Unified constraints failed."
            ));
        }

        let unified = UnifiedLock {
            version: 1,
            law: "A = μ(O*)".to_string(),
            workspaces: processed_cells,
            dependencies: unified_deps,
        };

        let lock_path = self.root_path.join("rocket.lock");
        // Serialize to deterministic JSON
        let content = serde_json::to_string_pretty(&unified)?;
        fs::write(&lock_path, content).wrap_err("Failed to commit rocket.lock to disk")?;

        println!(
            "{} Unified boundary locked deterministically: {}",
            "[A]".bold().green(),
            lock_path.display()
        );
        Ok(())
    }
}

/// Helper entry point to invoke the locker script logic directly.
pub fn run_lock(repo_root: &Path) -> Result<()> {
    let locker = EcosystemLocker::new(repo_root);
    locker.enforce_unified_lock()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ecosystem_locker_new_sets_root_path() {
        let locker = EcosystemLocker::new("/tmp/fake-repo");
        assert!(locker.root_path.ends_with("fake-repo"));
    }

    #[test]
    fn unified_lock_serializes_and_deserializes() {
        let mut deps = BTreeMap::new();
        deps.insert(
            "serde".into(),
            LockedDependency {
                name: "serde".into(),
                version: "1.0.200".into(),
                source: "registry+https://github.com/rust-lang/crates.io-index".into(),
                expected_features: BTreeSet::from(["derive".into()]),
            },
        );
        let lock = UnifiedLock {
            version: 1,
            law: "A = μ(O*)".into(),
            workspaces: BTreeSet::from(["nexus-engine".into()]),
            dependencies: deps,
        };
        let json = serde_json::to_string(&lock).expect("serialize");
        let decoded: UnifiedLock = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.law, "A = μ(O*)");
        assert!(decoded.dependencies.contains_key("serde"));
    }

    #[test]
    fn locked_dependency_clone_is_independent() {
        let dep = LockedDependency {
            name: "tokio".into(),
            version: "1.38.0".into(),
            source: "registry".into(),
            expected_features: BTreeSet::from(["full".into()]),
        };
        let mut cloned = dep.clone();
        cloned.version = "1.99.0".into();
        assert_eq!(dep.version, "1.38.0"); // original unaffected
    }

    #[test]
    fn unified_lock_workspaces_are_sorted() {
        let lock = UnifiedLock {
            version: 1,
            law: "A = μ(O*)".into(),
            workspaces: BTreeSet::from([
                "nexus-engine".into(),
                "blueprint-rs".into(),
                "genie3-rs".into(),
            ]),
            dependencies: BTreeMap::new(),
        };
        let mut sorted = lock.workspaces.iter().cloned().collect::<Vec<_>>();
        sorted.sort();
        let actual: Vec<_> = lock.workspaces.iter().cloned().collect();
        assert_eq!(sorted, actual); // BTreeSet iterates in sorted order
    }

    #[test]
    fn run_lock_with_nonexistent_repo_returns_err() {
        let result = run_lock(Path::new("/tmp/does-not-exist-abc123"));
        assert!(result.is_err());
    }
}
