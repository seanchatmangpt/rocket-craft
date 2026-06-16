use std::path::{Path, PathBuf};
use anyhow::Result;
pub use unrdf::Project;
use serde::{Deserialize, Serialize};

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
            return Err(anyhow::anyhow!("Manifest must contain at least one project"));
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

/// Phase: Input - The raw location of the manifest file.
pub struct Input {
    pub path: PathBuf,
}

/// Phase: Validated - The manifest has been read and verified against the basic schema.
pub struct Validated {
    pub path: PathBuf,
    pub projects: Vec<Project>,
}

/// Phase: Admitted - The manifest has passed all semantic law checks and is ready for use in the SDK.
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
