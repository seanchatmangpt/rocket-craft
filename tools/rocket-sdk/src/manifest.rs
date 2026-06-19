use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
pub use unrdf::Project;

#[derive(Debug, Serialize, Deserialize)]
struct RawManifest {
    projects: Vec<Project>,
}

/// The Manifest Law defines the semantic constraints of the project manifest.
/// It ensures that the manifest data is mathematically consistent and follows the Ostar theorem.
pub trait ManifestLaw {
    fn validate(path: &Path) -> Result<Vec<Project>>;
    fn admit(projects: Vec<Project>) -> Result<Vec<Project>>;
}

/// Ostar implementation of the Manifest Law.
/// This implementation enforces basic consistency and schema adherence.
pub struct OstarManifestLaw;

impl ManifestLaw for OstarManifestLaw {
    fn validate(path: &Path) -> Result<Vec<Project>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read manifest at {:?}: {}", path, e))?;
        let raw: RawManifest = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {}", e))?;
        Ok(raw.projects)
    }

    fn admit(projects: Vec<Project>) -> Result<Vec<Project>> {
        if projects.is_empty() {
            return Err(anyhow::anyhow!(
                "Manifest must contain at least one project"
            ));
        }
        for (i, p) in projects.iter().enumerate() {
            if p.name.is_empty() {
                return Err(anyhow::anyhow!("Project at index {} has no name", i));
            }
        }
        Ok(projects)
    }
}

/// The state machine kernel physically enforces the operational theorem: Input -> Validated -> Admitted.
/// Each state transition consumes the previous state, ensuring linear progress and preventing illegal aliasing.
pub struct Machine<L, P> {
    _law: std::marker::PhantomData<L>,
    pub phase: P,
}

impl<L, P: std::fmt::Debug> std::fmt::Debug for Machine<L, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Machine").field("phase", &self.phase).finish()
    }
}

/// Phase: Input - The raw location of the manifest file.
#[derive(Debug)]
pub struct Input {
    pub path: PathBuf,
}

/// Phase: Validated - The manifest has been read and verified against the basic schema.
#[derive(Debug)]
pub struct Validated {
    pub path: PathBuf,
    pub projects: Vec<Project>,
}

/// Phase: Admitted - The manifest has passed all semantic law checks and is ready for use in the SDK.
#[derive(Debug)]
pub struct Admitted {
    pub path: PathBuf,
    pub projects: Vec<Project>,
}

impl<L: ManifestLaw> Machine<L, Input> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            _law: std::marker::PhantomData,
            phase: Input { path },
        }
    }

    pub fn validate(self) -> Result<Machine<L, Validated>> {
        let projects = L::validate(&self.phase.path)?;
        Ok(Machine {
            _law: std::marker::PhantomData,
            phase: Validated {
                path: self.phase.path,
                projects,
            },
        })
    }
}

impl<L: ManifestLaw> Machine<L, Validated> {
    pub fn admit(self) -> Result<Machine<L, Admitted>> {
        let projects = L::admit(self.phase.projects)?;
        Ok(Machine {
            _law: std::marker::PhantomData,
            phase: Admitted {
                path: self.phase.path,
                projects,
            },
        })
    }
}

/// The SDK Manifest module provides a high-level, strongly-typed interface to project-manifest.json.
/// It wraps the underlying Ostar Machine to provide a familiar API for the rest of the toolkit.
#[derive(Debug)]
pub struct Manifest {
    inner: Machine<OstarManifestLaw, Admitted>,
}

impl Manifest {
    /// Create a new Manifest from a list of projects.
    /// This bypasses the validation phase and goes straight to Admitted, assuming the projects are already valid.
    pub fn new(path: impl Into<PathBuf>, projects: Vec<Project>) -> Self {
        Self {
            inner: Machine {
                _law: std::marker::PhantomData,
                phase: Admitted {
                    path: path.into(),
                    projects,
                },
            },
        }
    }

    /// Load the manifest from a file, passing through the full law-abiding transition chain.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let machine = Machine::<OstarManifestLaw, Input>::new(path.as_ref().to_path_buf())
            .validate()?
            .admit()?;
        Ok(Self { inner: machine })
    }

    /// Persist the manifest back to disk.
    pub fn save(&self) -> Result<()> {
        let raw = RawManifest {
            projects: self.inner.phase.projects.clone(),
        };
        let content = serde_json::to_string_pretty(&raw)?;
        std::fs::write(&self.inner.phase.path, content)?;
        Ok(())
    }

    /// Access the list of projects in the manifest.
    pub fn projects(&self) -> &[Project] {
        &self.inner.phase.projects
    }

    /// Create a manifest from an existing list of projects.
    pub fn from_projects<P: AsRef<Path>>(path: P, projects: Vec<Project>) -> Self {
        Self::new(path.as_ref().to_path_buf(), projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_manifest(dir: &TempDir, json: &str) -> PathBuf {
        let path = dir.path().join("project-manifest.json");
        std::fs::write(&path, json).unwrap();
        path
    }

    fn sample_json() -> &'static str {
        r#"{
  "projects": [
    { "name": "Brm", "uproject_path": "versions/4.27.0/Brm.uproject", "targets": ["Brm", "BrmEditor"] },
    { "name": "ShooterGame", "uproject_path": "versions/4.24-Shooter/ShooterGame/ShooterGame.uproject", "targets": ["ShooterGame"] }
  ]
}"#
    }

    #[test]
    fn load_valid_manifest() {
        let dir = TempDir::new().unwrap();
        let path = write_manifest(&dir, sample_json());
        let manifest = Manifest::load(&path).unwrap();
        assert_eq!(manifest.projects().len(), 2);
        assert_eq!(manifest.projects()[0].name, "Brm");
        assert_eq!(manifest.projects()[1].name, "ShooterGame");
    }

    #[test]
    fn projects_have_correct_targets() {
        let dir = TempDir::new().unwrap();
        let path = write_manifest(&dir, sample_json());
        let manifest = Manifest::load(&path).unwrap();
        let brm = manifest.projects().iter().find(|p| p.name == "Brm").unwrap();
        assert!(brm.targets.contains(&"BrmEditor".to_string()));
    }

    #[test]
    fn empty_projects_array_rejected() {
        let dir = TempDir::new().unwrap();
        let path = write_manifest(&dir, r#"{"projects": []}"#);
        let err = Manifest::load(&path).unwrap_err();
        assert!(err.to_string().contains("at least one project"));
    }

    #[test]
    fn project_with_empty_name_rejected() {
        let dir = TempDir::new().unwrap();
        let json = r#"{"projects": [{"name": "", "uproject_path": "foo.uproject", "targets": []}]}"#;
        let path = write_manifest(&dir, json);
        let err = Manifest::load(&path).unwrap_err();
        assert!(err.to_string().contains("no name"));
    }

    #[test]
    fn malformed_json_rejected() {
        let dir = TempDir::new().unwrap();
        let path = write_manifest(&dir, "{ not valid json }");
        let err = Manifest::load(&path).unwrap_err();
        assert!(err.to_string().contains("parse manifest"));
    }

    #[test]
    fn missing_file_rejected() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        let err = Manifest::load(&path).unwrap_err();
        assert!(err.to_string().contains("Failed to read manifest"));
    }

    #[test]
    fn save_and_reload_roundtrips() {
        let dir = TempDir::new().unwrap();
        let path = write_manifest(&dir, sample_json());
        let manifest = Manifest::load(&path).unwrap();
        manifest.save().unwrap();
        let reloaded = Manifest::load(&path).unwrap();
        assert_eq!(manifest.projects().len(), reloaded.projects().len());
        for (a, b) in manifest.projects().iter().zip(reloaded.projects()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.uproject_path, b.uproject_path);
        }
    }

    #[test]
    fn typestate_machine_rejects_empty_at_admit() {
        let projects: Vec<Project> = vec![];
        let err = OstarManifestLaw::admit(projects).unwrap_err();
        assert!(err.to_string().contains("at least one project"));
    }
}
