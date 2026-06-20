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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_manifest(dir: &std::path::Path, content: &str) -> PathBuf {
        let path = dir.join("project-manifest.json");
        fs::write(&path, content).unwrap();
        path
    }

    // ── Pending → Ingested ─────────────────────────────────────────────────────

    #[test]
    fn ingest_reads_projects_from_json() {
        let dir = tempdir().unwrap();
        let path = write_manifest(dir.path(), r#"{"projects":[{"name":"Brm","uproject_path":"Brm.uproject","targets":[]}]}"#);
        let manifest = Manifest::new(path).ingest().unwrap();
        assert_eq!(manifest.projects().len(), 1);
        assert_eq!(manifest.projects()[0].name, "Brm");
    }

    #[test]
    fn ingest_returns_err_on_missing_file() {
        let result = Manifest::new("/nonexistent/project-manifest.json").ingest();
        assert!(result.is_err());
    }

    #[test]
    fn ingest_returns_err_on_invalid_json() {
        let dir = tempdir().unwrap();
        let path = write_manifest(dir.path(), "not json");
        assert!(Manifest::new(path).ingest().is_err());
    }

    #[test]
    fn ingest_empty_projects_array_ok() {
        let dir = tempdir().unwrap();
        let path = write_manifest(dir.path(), r#"{"projects":[]}"#);
        let manifest = Manifest::new(path).ingest().unwrap();
        assert!(manifest.projects().is_empty());
    }

    // ── Ingested helpers ───────────────────────────────────────────────────────

    #[test]
    fn from_projects_exposes_projects_and_path() {
        let manifest = Manifest::from_projects(
            "/some/path/manifest.json",
            vec![Project { name: "Test".into(), uproject_path: "T.uproject".into(), targets: vec![] }],
        );
        assert_eq!(manifest.projects().len(), 1);
        assert_eq!(manifest.path(), std::path::Path::new("/some/path/manifest.json"));
    }

    #[test]
    fn ingested_save_roundtrips_to_disk() {
        let dir = tempdir().unwrap();
        let path = write_manifest(dir.path(), r#"{"projects":[]}"#);
        let manifest = Manifest::from_projects(
            &path,
            vec![Project { name: "X".into(), uproject_path: "X.uproject".into(), targets: vec!["XEditor".into()] }],
        );
        manifest.save().unwrap();
        let reloaded = Manifest::new(&path).ingest().unwrap();
        assert_eq!(reloaded.projects()[0].name, "X");
        assert_eq!(reloaded.projects()[0].targets, vec!["XEditor"]);
    }

    // ── Ingested → Validated ───────────────────────────────────────────────────

    #[test]
    fn validate_passes_when_all_uproject_files_exist() {
        let dir = tempdir().unwrap();
        let uproject = dir.path().join("Game.uproject");
        fs::write(&uproject, b"{}").unwrap();
        let manifest = Manifest::from_projects(
            dir.path().join("project-manifest.json"),
            vec![Project { name: "Game".into(), uproject_path: "Game.uproject".into(), targets: vec![] }],
        );
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn validate_fails_when_uproject_missing() {
        let dir = tempdir().unwrap();
        let manifest = Manifest::from_projects(
            dir.path().join("project-manifest.json"),
            vec![Project { name: "Ghost".into(), uproject_path: "Ghost.uproject".into(), targets: vec![] }],
        );
        let err = match manifest.validate() { Ok(_) => panic!("expected Err"), Err(e) => e };
        assert!(err.to_string().contains("Ghost"));
    }

    #[test]
    fn validate_empty_projects_always_passes() {
        let manifest = Manifest::from_projects("/tmp/manifest.json", vec![]);
        assert!(manifest.validate().is_ok());
    }

    // ── Project::from tuple ────────────────────────────────────────────────────

    #[test]
    fn project_from_tuple_converts_fields() {
        let p = Project::from(("Brm".to_string(), PathBuf::from("Brm.uproject"), vec!["BrmEditor".to_string()]));
        assert_eq!(p.name, "Brm");
        assert_eq!(p.uproject_path, PathBuf::from("Brm.uproject"));
        assert_eq!(p.targets, vec!["BrmEditor"]);
    }
}
