pub mod audit_affidavit;
pub mod signing;
pub mod config;
pub mod crypto;
pub mod doctor;
pub mod error;
pub mod html5;
pub mod manifest;
pub mod setup;
pub mod supabase;
pub mod wasm;
pub use html5::{discover_emsdk_python, Html5Cook, Html5PackageVerifier, Html5PackageReport, Html5Setup, WasmVerdict, CookLogEvent, parse_cook_log};

pub use crate::manifest::Manifest;
use anyhow::Result;
use std::path::{Path, PathBuf};
pub use unrdf::Project as SemanticProject;

/// Core context for the Rocket SDK, providing access to the project workspace and manifest.
#[derive(Debug)]
pub struct RocketContext {
    pub root: PathBuf,
    pub manifest: Manifest,
}

impl RocketContext {
    /// Loads the RocketContext from the given root directory.
    /// Expects a 'project-manifest.json' to exist in the root.
    pub fn load(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let manifest_path = root.join("project-manifest.json");
        let manifest = Manifest::load(manifest_path)?;
        Ok(Self { root, manifest })
    }

    /// Returns a list of projects defined in the manifest, wrapped in the SDK's Project abstraction.
    pub fn projects(&self) -> Vec<Project> {
        self.manifest
            .projects()
            .iter()
            .map(|p| Project::new(p.clone(), self.root.clone()))
            .collect()
    }
}

/// A high-level abstraction of a project within the Rocket workspace.
/// Leverages unrdf::Project for semantic metadata.
pub struct Project {
    pub inner: SemanticProject,
    pub root: PathBuf,
}

impl Project {
    pub fn new(inner: SemanticProject, root: PathBuf) -> Self {
        Self { inner, root }
    }

    /// Returns the absolute path to the .uproject file.
    pub fn absolute_uproject_path(&self) -> PathBuf {
        self.root.join(&self.inner.uproject_path)
    }

    /// Returns the project name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Returns the list of targets for this project.
    pub fn targets(&self) -> &[String] {
        &self.inner.targets
    }

    /// Creates a Build request for a specific target and platform.
    pub fn build(&self, target: String, platform: String) -> Build {
        Build {
            project_path: self.absolute_uproject_path(),
            target,
            platform,
        }
    }
}

/// Represents a build request for an Unreal project.
pub struct Build {
    pub project_path: PathBuf,
    pub target: String,
    pub platform: String,
}

impl Build {
    /// Executes the build using the provided executor and UE4 root path.
    pub fn run<E: BuildExecutor>(&self, executor: &E, ue4_root: &Path) -> Result<()> {
        executor.execute(self, ue4_root)
    }
}

/// Trait for executing Unreal builds.
/// This allows for mocking the build process in tests.
pub trait BuildExecutor {
    fn execute(&self, build: &Build, ue4_root: &Path) -> Result<()>;
}

/// Default implementation of BuildExecutor that calls RunUAT.
pub struct UatBuildExecutor;

impl BuildExecutor for UatBuildExecutor {
    fn execute(&self, build: &Build, ue4_root: &Path) -> Result<()> {
        let uat_name = if cfg!(windows) {
            "RunUAT.bat"
        } else {
            "RunUAT.sh"
        };
        let uat_path = ue4_root
            .join("Engine")
            .join("Build")
            .join("BatchFiles")
            .join(uat_name);

        let status = std::process::Command::new(&uat_path)
            .arg("BuildCookRun")
            .arg(format!("-project={}", build.project_path.display()))
            .arg(format!("-target={}", build.target))
            .arg(format!("-platform={}", build.platform))
            .arg("-cook")
            .arg("-build")
            .arg("-stage")
            .arg("-archive")
            .arg("-archivedirectory=Builds")
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Build failed with status: {}", status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use unrdf::Project as UnrdfProject;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn write_manifest(dir: &TempDir, projects: &[(&str, &str, &[&str])]) -> std::path::PathBuf {
        let entries: Vec<serde_json::Value> = projects
            .iter()
            .map(|(name, path, targets)| {
                serde_json::json!({
                    "name": name,
                    "uproject_path": path,
                    "targets": targets
                })
            })
            .collect();
        let json = serde_json::to_string(&serde_json::json!({"projects": entries})).unwrap();
        let path = dir.path().join("project-manifest.json");
        std::fs::write(&path, json).unwrap();
        path
    }

    /// A build executor that records invocations instead of spawning UAT.
    #[derive(Clone, Default)]
    struct MockExecutor {
        calls: Arc<Mutex<Vec<(String, String, String)>>>,
        should_fail: bool,
    }

    impl MockExecutor {
        fn recorded(&self) -> Vec<(String, String, String)> {
            self.calls.lock().unwrap().clone()
        }
        fn failing() -> Self {
            Self {
                calls: Arc::new(Mutex::new(vec![])),
                should_fail: true,
            }
        }
    }

    impl BuildExecutor for MockExecutor {
        fn execute(&self, build: &Build, ue4_root: &Path) -> Result<()> {
            // Record (project_path, target, ue4_root) — platform tested separately
            self.calls.lock().unwrap().push((
                build.project_path.display().to_string(),
                build.target.clone(),
                ue4_root.display().to_string(),
            ));
            if self.should_fail {
                Err(anyhow::anyhow!("mock failure"))
            } else {
                Ok(())
            }
        }
    }

    // ── RocketContext ─────────────────────────────────────────────────────────

    #[test]
    fn context_loads_projects_from_manifest() {
        let dir = TempDir::new().unwrap();
        write_manifest(
            &dir,
            &[
                ("Brm", "versions/4.27.0/Brm.uproject", &["Brm", "BrmEditor"]),
                (
                    "ShooterGame",
                    "versions/4.24/ShooterGame.uproject",
                    &["ShooterGame"],
                ),
            ],
        );
        let ctx = RocketContext::load(dir.path()).unwrap();
        assert_eq!(ctx.projects().len(), 2);
    }

    #[test]
    fn context_missing_manifest_errors() {
        let dir = TempDir::new().unwrap();
        let err = RocketContext::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("manifest") || err.to_string().contains("No such file"));
    }

    // ── Project ───────────────────────────────────────────────────────────────

    #[test]
    fn project_absolute_path_joins_root() {
        let dir = TempDir::new().unwrap();
        write_manifest(&dir, &[("Brm", "versions/4.27.0/Brm.uproject", &["Brm"])]);
        let ctx = RocketContext::load(dir.path()).unwrap();
        let proj = &ctx.projects()[0];
        let abs = proj.absolute_uproject_path();
        assert!(
            abs.starts_with(dir.path()),
            "absolute path must be under the workspace root"
        );
        assert!(abs.ends_with("Brm.uproject"));
    }

    #[test]
    fn project_name_and_targets_accessible() {
        let inner = UnrdfProject {
            name: "TestProj".into(),
            uproject_path: "test/TestProj.uproject".into(),
            targets: vec!["TestProj".into(), "TestProjEditor".into()],
        };
        let proj = Project::new(inner, "/workspace".into());
        assert_eq!(proj.name(), "TestProj");
        assert_eq!(proj.targets(), &["TestProj", "TestProjEditor"]);
    }

    // ── Build + BuildExecutor ─────────────────────────────────────────────────

    #[test]
    fn build_dispatches_to_executor() {
        let dir = TempDir::new().unwrap();
        write_manifest(&dir, &[("Brm", "versions/4.27.0/Brm.uproject", &["Brm"])]);
        let ctx = RocketContext::load(dir.path()).unwrap();
        let proj = &ctx.projects()[0];
        let build = proj.build("Brm".into(), "HTML5".into());

        let mock = MockExecutor::default();
        let fake_ue4 = std::path::Path::new("/fake/ue4-root");
        build.run(&mock, fake_ue4).unwrap();

        let calls = mock.recorded();
        assert_eq!(calls.len(), 1);
        assert!(
            calls[0].0.contains("Brm.uproject"),
            "project path must contain Brm.uproject"
        );
        assert_eq!(calls[0].1, "Brm", "target must be 'Brm'");
        assert_eq!(calls[0].2, "/fake/ue4-root", "ue4 root must match");
    }

    #[test]
    fn failing_executor_propagates_error() {
        let dir = TempDir::new().unwrap();
        write_manifest(&dir, &[("Brm", "versions/4.27.0/Brm.uproject", &["Brm"])]);
        let ctx = RocketContext::load(dir.path()).unwrap();
        let build = ctx.projects()[0].build("Brm".into(), "HTML5".into());
        let err = build
            .run(&MockExecutor::failing(), std::path::Path::new("/fake"))
            .unwrap_err();
        assert!(err.to_string().contains("mock failure"));
    }

    #[test]
    fn build_platform_is_passed_through_to_executor() {
        // Platform is stored on Build but not recorded by MockExecutor's tuple.
        // Access it via the Build struct directly before running.
        let inner = UnrdfProject {
            name: "Brm".into(),
            uproject_path: "Brm.uproject".into(),
            targets: vec![],
        };
        let proj = Project::new(inner, "/workspace".into());
        let build = proj.build("Brm".into(), "HTML5".into());
        assert_eq!(build.platform, "HTML5");
        assert_eq!(build.target, "Brm");
    }

    #[test]
    fn multiple_projects_accessible_by_index() {
        let dir = TempDir::new().unwrap();
        write_manifest(
            &dir,
            &[
                ("Alpha", "a/Alpha.uproject", &["Alpha"]),
                ("Beta", "b/Beta.uproject", &["Beta"]),
                ("Gamma", "c/Gamma.uproject", &["Gamma"]),
            ],
        );
        let ctx = RocketContext::load(dir.path()).unwrap();
        let projects = ctx.projects();
        assert_eq!(projects.len(), 3);
        let names: Vec<&str> = projects.iter().map(|p| p.name()).collect();
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
        assert!(names.contains(&"Gamma"));
    }
}
