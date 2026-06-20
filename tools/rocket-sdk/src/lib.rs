pub mod manifest;
pub mod error;
pub mod config;
pub mod setup;
pub mod crypto;
pub mod doctor;
pub mod supabase;
pub mod audit_affidavit;
pub mod ui;
pub mod completions;
pub mod wizard;
pub mod cache;
pub mod watch;
pub mod access_control;
pub mod audit_log;
pub mod secret_scan;
pub mod sbom;

use std::path::{Path, PathBuf};
use anyhow::Result;
pub use unrdf::Project as SemanticProject;
pub use crate::manifest::Manifest;

/// Core context for the Rocket SDK, providing access to the project workspace and manifest.
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
        let uat_name = if cfg!(windows) { "RunUAT.bat" } else { "RunUAT.sh" };
        let uat_path = ue4_root.join("Engine").join("Build").join("BatchFiles").join(uat_name);

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
