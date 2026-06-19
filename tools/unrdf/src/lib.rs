pub mod manifest;
pub mod pipeline;
pub mod project_bridge;
pub mod shacl;
pub mod sparql;
pub mod store;
pub mod triple;

pub use store::TripleStore;
pub use triple::{Term, Triple};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during UNRDF operations.
#[derive(Error, Debug)]
pub enum UnrdfError {
    /// An input/output error occurred while accessing a manifest or project file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// An error occurred while parsing or serializing JSON data.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The manifest or project data failed validation.
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Information about an Unreal Engine project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    /// The name of the project.
    pub name: String,
    /// The relative or absolute path to the .uproject file.
    pub uproject_path: PathBuf,
    /// A list of build targets associated with the project.
    pub targets: Vec<String>,
}

impl From<(String, PathBuf, Vec<String>)> for Project {
    fn from(data: (String, PathBuf, Vec<String>)) -> Self {
        Self {
            name: data.0,
            uproject_path: data.1,
            targets: data.2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct RawManifest {
    projects: Vec<Project>,
}

/// A typestate-pattern manifest container.
///
/// This structure represents a collection of Unreal Engine projects in different
/// states of processing (e.g., Pending, Ingested, Validated).
pub struct Manifest<S> {
    state: S,
}

/// Represents a manifest that has a path but has not been loaded into memory yet.
pub struct Pending {
    path: PathBuf,
}

/// Represents a manifest that has been successfully read and parsed from disk.
pub struct Ingested {
    /// The path to the manifest file on disk.
    pub path: PathBuf,
    /// The list of projects defined in the manifest.
    pub projects: Vec<Project>,
}

/// Represents a manifest where all referenced project files have been verified to exist.
pub struct Validated {
    /// The path to the manifest file on disk.
    pub path: PathBuf,
    /// The list of verified projects.
    pub projects: Vec<Project>,
}

impl Manifest<Pending> {
    /// Creates a new `Manifest` in the `Pending` state for the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            state: Pending { path: path.into() },
        }
    }

    /// Reads the manifest from disk and transitions to the `Ingested` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the JSON is malformed.
    pub fn ingest(self) -> Result<Manifest<Ingested>, UnrdfError> {
        let content = std::fs::read_to_string(&self.state.path)?;
        let raw: RawManifest = serde_json::from_str(&content)?;

        Ok(Manifest {
            state: Ingested {
                path: self.state.path,
                projects: raw.projects,
            },
        })
    }
}

impl Manifest<Ingested> {
    /// Creates an `Ingested` manifest directly from a list of projects.
    pub fn from_projects(path: impl Into<PathBuf>, projects: Vec<Project>) -> Self {
        Self {
            state: Ingested {
                path: path.into(),
                projects,
            },
        }
    }

    /// Returns a reference to the list of projects in the manifest.
    pub fn projects(&self) -> &[Project] {
        &self.state.projects
    }

    /// Returns a reference to the manifest's file path.
    pub fn path(&self) -> &Path {
        &self.state.path
    }

    /// Saves the current manifest state to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file writing fails.
    pub fn save(&self) -> Result<(), UnrdfError> {
        let raw = RawManifest {
            projects: self.state.projects.clone(),
        };
        let content = serde_json::to_string_pretty(&raw)?;
        std::fs::write(&self.state.path, content)?;
        Ok(())
    }

    /// Verifies that all project files referenced in the manifest exist and transitions to `Validated`.
    ///
    /// # Errors
    ///
    /// Returns an error if any project file is missing or if the manifest path is invalid.
    pub fn validate(self) -> Result<Manifest<Validated>, UnrdfError> {
        let root = self.state.path.parent().unwrap_or(Path::new("."));
        for project in &self.state.projects {
            let full_path = root.join(&project.uproject_path);
            if !full_path.exists() {
                return Err(UnrdfError::Validation(format!(
                    "uproject file not found for project '{}' at {:?}",
                    project.name, full_path
                )));
            }
        }

        Ok(Manifest {
            state: Validated {
                path: self.state.path,
                projects: self.state.projects,
            },
        })
    }
}

impl Manifest<Validated> {
    /// Returns a reference to the list of verified projects.
    pub fn projects(&self) -> &[Project] {
        &self.state.projects
    }

    /// Returns a reference to the manifest's file path.
    pub fn path(&self) -> &Path {
        &self.state.path
    }

    /// Saves the validated manifest state to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file writing fails.
    pub fn save(&self) -> Result<(), UnrdfError> {
        let raw = RawManifest {
            projects: self.state.projects.clone(),
        };
        let content = serde_json::to_string_pretty(&raw)?;
        std::fs::write(&self.state.path, content)?;
        Ok(())
    }
}
