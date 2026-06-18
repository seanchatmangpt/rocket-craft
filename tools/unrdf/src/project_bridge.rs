use crate::{triple::{Term, Triple}, store::TripleStore};

/// Lifecycle state: manifest path known but not yet loaded.
#[derive(Debug)]
pub struct Pending {
    pub path: std::path::PathBuf,
}

/// Lifecycle state: manifest has been read and parsed from disk.
#[derive(Debug)]
pub struct Ingested {
    pub path: std::path::PathBuf,
    pub projects: Vec<UeProject>,
}

/// Lifecycle state: all referenced .uproject files verified to exist on disk.
#[derive(Debug)]
pub struct Validated {
    pub path: std::path::PathBuf,
    pub projects: Vec<UeProject>,
}

/// A UE4 project entry (mirrors unrdf::Project).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UeProject {
    pub name: String,
    pub uproject_path: std::path::PathBuf,
    pub targets: Vec<String>,
}

/// Typestate manifest container (mirrors unrdf::Manifest<S>).
pub struct ProjectManifest<S: std::fmt::Debug> {
    state: S,
}

impl<S: std::fmt::Debug> std::fmt::Debug for ProjectManifest<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectManifest")
            .field("state", &self.state)
            .finish()
    }
}

impl ProjectManifest<Pending>
where
    Pending: std::fmt::Debug,
{
    /// Create a new manifest in the `Pending` state for the given path.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            state: Pending { path: path.into() },
        }
    }

    /// Load and parse the JSON manifest, transitioning `Pending → Ingested`.
    ///
    /// The manifest file must be a JSON object with a `projects` array whose
    /// items each have `name`, `uproject_path`, and `targets` fields.
    pub fn ingest(self) -> Result<ProjectManifest<Ingested>, ManifestError> {
        let content = std::fs::read_to_string(&self.state.path).map_err(ManifestError::Io)?;
        let raw: RawManifest = serde_json::from_str(&content).map_err(ManifestError::Json)?;
        Ok(ProjectManifest {
            state: Ingested {
                path: self.state.path,
                projects: raw.projects,
            },
        })
    }
}

impl ProjectManifest<Ingested>
where
    Ingested: std::fmt::Debug,
{
    /// Return the parsed list of projects.
    pub fn projects(&self) -> &[UeProject] {
        &self.state.projects
    }

    /// Verify all `.uproject` files exist on disk, transitioning `Ingested → Validated`.
    ///
    /// Paths are readdressed relative to the directory that contains the manifest
    /// file (matching `unrdf::Manifest<Ingested>::validate` behaviour).
    pub fn validate(self) -> Result<ProjectManifest<Validated>, ManifestError> {
        let root = self
            .state
            .path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        for project in &self.state.projects {
            let full_path = root.join(&project.uproject_path);
            if !full_path.exists() {
                return Err(ManifestError::Validation(format!(
                    "uproject file not found for project '{}' at {:?}",
                    project.name, full_path
                )));
            }
        }
        Ok(ProjectManifest {
            state: Validated {
                path: self.state.path,
                projects: self.state.projects,
            },
        })
    }
}

impl ProjectManifest<Validated>
where
    Validated: std::fmt::Debug,
{
    /// Return the validated list of projects.
    pub fn projects(&self) -> &[UeProject] {
        &self.state.projects
    }

    /// Convert all validated projects into an RDF [`TripleStore`] ready for
    /// SPARQL-style queries.
    pub fn to_triple_store(&self) -> TripleStore {
        let mut store = TripleStore::new();
        for project in &self.state.projects {
            for triple in project_to_triples(project) {
                store.add(triple);
            }
        }
        store
    }

    /// Serialize the validated project list to a pretty-printed JSON string.
    pub fn to_json(&self) -> String {
        let raw = RawManifest {
            projects: self.state.projects.clone(),
        };
        serde_json::to_string_pretty(&raw).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Internal raw deserialization helper (mirrors unrdf::RawManifest).
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct RawManifest {
    projects: Vec<UeProject>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur while loading or validating a project manifest.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// An I/O error while reading the manifest file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A JSON parse error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// A validation failure (e.g. missing .uproject file).
    #[error("Validation error: {0}")]
    Validation(String),
}

// ---------------------------------------------------------------------------
// RDF conversion helpers
// ---------------------------------------------------------------------------

/// UE namespace prefix used for all generated IRIs.
const UE_NS: &str = "ue:";

/// Convert a [`UeProject`] into a set of RDF triples.
///
/// The subject IRI is `ue:project:<NAME>`.  Predicates emitted:
/// - `ue:name` — the project name (xsd:string literal)
/// - `ue:uprojectPath` — the path to the .uproject file (xsd:string literal)
/// - `ue:target` — one triple per build target (xsd:string literal)
pub fn project_to_triples(project: &UeProject) -> Vec<Triple> {
    let subject = Term::Named(format!("{}project:{}", UE_NS, project.name));
    let pred_name = Term::Named(format!("{}name", UE_NS));
    let pred_path = Term::Named(format!("{}uprojectPath", UE_NS));
    let pred_target = Term::Named(format!("{}target", UE_NS));

    let mut triples = Vec::new();

    // ue:name literal
    triples.push(Triple::new(
        subject.clone(),
        pred_name,
        Term::Literal {
            value: project.name.clone(),
            datatype: Some("xsd:string".into()),
            lang: None,
        },
    ));

    // ue:uprojectPath literal
    triples.push(Triple::new(
        subject.clone(),
        pred_path,
        Term::Literal {
            value: project.uproject_path.to_string_lossy().into_owned(),
            datatype: Some("xsd:string".into()),
            lang: None,
        },
    ));

    // ue:target — one triple per target
    for target in &project.targets {
        triples.push(Triple::new(
            subject.clone(),
            pred_target.clone(),
            Term::Literal {
                value: target.clone(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        ));
    }

    triples
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use tempfile::TempDir;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Write a manifest JSON to a temp file and return (dir, path).
    fn write_manifest(dir: &TempDir, json: &str) -> std::path::PathBuf {
        let path = dir.path().join("manifest.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
        path
    }

    fn minimal_manifest_json(rel_path: &str) -> String {
        serde_json::json!({
            "projects": [{
                "name": "TestGame",
                "uproject_path": rel_path,
                "targets": ["TestGameEditor"]
            }]
        })
        .to_string()
    }

    // ------------------------------------------------------------------
    // 1. ProjectManifest::new creates Pending state
    // ------------------------------------------------------------------

    #[test]
    fn test_new_creates_pending_with_correct_path() {
        let manifest = ProjectManifest::new("/some/path/manifest.json");
        assert_eq!(
            manifest.state.path,
            std::path::PathBuf::from("/some/path/manifest.json")
        );
    }

    // ------------------------------------------------------------------
    // 2. ingest() fails on missing file
    // ------------------------------------------------------------------

    #[test]
    fn test_ingest_fails_on_missing_file() {
        let manifest = ProjectManifest::new("/nonexistent/path/manifest.json");
        let result = manifest.ingest();
        assert!(result.is_err());
        match result.unwrap_err() {
            ManifestError::Io(_) => {}
            other => panic!("expected Io error, got {:?}", other),
        }
    }

    // ------------------------------------------------------------------
    // 3. ingest() fails on malformed JSON
    // ------------------------------------------------------------------

    #[test]
    fn test_ingest_fails_on_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_manifest(&dir, "not valid json { }}");
        let result = ProjectManifest::new(path).ingest();
        assert!(result.is_err());
        match result.unwrap_err() {
            ManifestError::Json(_) => {}
            other => panic!("expected Json error, got {:?}", other),
        }
    }

    // ------------------------------------------------------------------
    // 4. ingest() parses a JSON manifest with a projects array
    // ------------------------------------------------------------------

    #[test]
    fn test_ingest_parses_projects_array() {
        let dir = tempfile::tempdir().unwrap();
        let json = serde_json::json!({
            "projects": [
                {"name": "Alpha", "uproject_path": "Alpha/Alpha.uproject", "targets": ["AlphaEditor"]},
                {"name": "Beta",  "uproject_path": "Beta/Beta.uproject",   "targets": []}
            ]
        })
        .to_string();
        let path = write_manifest(&dir, &json);
        let ingested = ProjectManifest::new(path).ingest().unwrap();
        let projects = ingested.projects();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "Alpha");
        assert_eq!(projects[1].name, "Beta");
    }

    // ------------------------------------------------------------------
    // 5. ingest() with the real project-manifest.json data
    // ------------------------------------------------------------------

    #[test]
    fn test_ingest_real_manifest_data() {
        let dir = tempfile::tempdir().unwrap();
        let json = serde_json::json!({
            "projects": [
                {
                    "name": "SurvivalGame",
                    "uproject_path": "versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/SurvivalGame.uproject",
                    "targets": ["SurvivalGameEditor", "SurvivalGameServer", "SurvivalGame"]
                },
                {
                    "name": "ShooterGame",
                    "uproject_path": "versions/4.24-Shooter/ShooterGame/ShooterGame.uproject",
                    "targets": ["ShooterGameEditor", "ShooterClient", "ShooterGame", "ShooterServer"]
                }
            ]
        })
        .to_string();
        let path = write_manifest(&dir, &json);
        let ingested = ProjectManifest::new(path).ingest().unwrap();
        assert_eq!(ingested.projects().len(), 2);
        assert_eq!(ingested.projects()[0].targets.len(), 3);
        assert_eq!(ingested.projects()[1].targets.len(), 4);
    }

    // ------------------------------------------------------------------
    // 6. validate() passes when all .uproject files exist on disk
    // ------------------------------------------------------------------

    #[test]
    fn test_validate_passes_when_uproject_files_exist() {
        let dir = tempfile::tempdir().unwrap();

        // Create the .uproject file inside the temp dir.
        let uproject_rel = "MyGame/MyGame.uproject";
        let uproject_full = dir.path().join(uproject_rel);
        std::fs::create_dir_all(uproject_full.parent().unwrap()).unwrap();
        std::fs::File::create(&uproject_full).unwrap();

        let json = minimal_manifest_json(uproject_rel);
        let path = write_manifest(&dir, &json);

        let result = ProjectManifest::new(path).ingest().unwrap().validate();
        assert!(result.is_ok());
    }

    // ------------------------------------------------------------------
    // 7. validate() fails when a .uproject file is missing
    // ------------------------------------------------------------------

    #[test]
    fn test_validate_fails_when_uproject_missing() {
        let dir = tempfile::tempdir().unwrap();
        let json = minimal_manifest_json("DoesNotExist/DoesNotExist.uproject");
        let path = write_manifest(&dir, &json);

        let result = ProjectManifest::new(path).ingest().unwrap().validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ManifestError::Validation(msg) => {
                assert!(msg.contains("TestGame"), "error should name the project");
            }
            other => panic!("expected Validation error, got {:?}", other),
        }
    }

    // ------------------------------------------------------------------
    // 8. validate() with multiple projects — fails on first missing one
    // ------------------------------------------------------------------

    #[test]
    fn test_validate_fails_on_first_missing_project() {
        let dir = tempfile::tempdir().unwrap();

        // Only create the second project's file.
        let second_rel = "Second/Second.uproject";
        let second_full = dir.path().join(second_rel);
        std::fs::create_dir_all(second_full.parent().unwrap()).unwrap();
        std::fs::File::create(&second_full).unwrap();

        let json = serde_json::json!({
            "projects": [
                {"name": "First",  "uproject_path": "First/First.uproject", "targets": []},
                {"name": "Second", "uproject_path": second_rel,              "targets": []}
            ]
        })
        .to_string();
        let path = write_manifest(&dir, &json);

        let result = ProjectManifest::new(path).ingest().unwrap().validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ManifestError::Validation(msg) => {
                assert!(msg.contains("First"));
            }
            other => panic!("expected Validation error, got {:?}", other),
        }
    }

    // ------------------------------------------------------------------
    // 9. project_to_triples() produces ue:name and ue:uprojectPath triples
    // ------------------------------------------------------------------

    #[test]
    fn test_project_to_triples_name_and_path() {
        let project = UeProject {
            name: "SurvivalGame".into(),
            uproject_path: "versions/SurvivalGame.uproject".into(),
            targets: vec![],
        };
        let triples = project_to_triples(&project);

        let name_pred = Term::Named("ue:name".into());
        let path_pred = Term::Named("ue:uprojectPath".into());

        let has_name = triples.iter().any(|t| {
            t.predicate == name_pred
                && t.object
                    == Term::Literal {
                        value: "SurvivalGame".into(),
                        datatype: Some("xsd:string".into()),
                        lang: None,
                    }
        });
        let has_path = triples.iter().any(|t| {
            t.predicate == path_pred
                && t.object
                    == Term::Literal {
                        value: "versions/SurvivalGame.uproject".into(),
                        datatype: Some("xsd:string".into()),
                        lang: None,
                    }
        });

        assert!(has_name, "missing ue:name triple");
        assert!(has_path, "missing ue:uprojectPath triple");
    }

    // ------------------------------------------------------------------
    // 10. project_to_triples() produces one ue:target triple per target
    // ------------------------------------------------------------------

    #[test]
    fn test_project_to_triples_targets() {
        let project = UeProject {
            name: "ShooterGame".into(),
            uproject_path: "ShooterGame.uproject".into(),
            targets: vec!["ShooterGameEditor".into(), "ShooterServer".into()],
        };
        let triples = project_to_triples(&project);
        let target_pred = Term::Named("ue:target".into());
        let target_triples: Vec<_> = triples
            .iter()
            .filter(|t| t.predicate == target_pred)
            .collect();
        assert_eq!(target_triples.len(), 2);
    }

    // ------------------------------------------------------------------
    // 11. to_triple_store() produces triples for every project
    // ------------------------------------------------------------------

    #[test]
    fn test_to_triple_store_contains_triples_for_all_projects() {
        let dir = tempfile::tempdir().unwrap();

        // Create both .uproject files.
        for name in &["Alpha", "Beta"] {
            let rel = format!("{}/{}.uproject", name, name);
            let full = dir.path().join(&rel);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::File::create(&full).unwrap();
        }

        let json = serde_json::json!({
            "projects": [
                {"name": "Alpha", "uproject_path": "Alpha/Alpha.uproject", "targets": ["AlphaEd"]},
                {"name": "Beta",  "uproject_path": "Beta/Beta.uproject",   "targets": []}
            ]
        })
        .to_string();
        let path = write_manifest(&dir, &json);

        let validated = ProjectManifest::new(path)
            .ingest()
            .unwrap()
            .validate()
            .unwrap();
        let store = validated.to_triple_store();

        // Alpha has name + path + 1 target = 3 triples; Beta has name + path = 2.
        assert_eq!(store.len(), 5);
    }

    // ------------------------------------------------------------------
    // 12. SPARQL SELECT over the triple store finds projects by name
    // ------------------------------------------------------------------

    #[test]
    fn test_sparql_select_all_triples_from_project_store() {
        use crate::sparql::{PatternExecutor, SparqlExecutor};

        let dir = tempfile::tempdir().unwrap();

        let uproject_rel = "Brm/Brm.uproject";
        let uproject_full = dir.path().join(uproject_rel);
        std::fs::create_dir_all(uproject_full.parent().unwrap()).unwrap();
        std::fs::File::create(&uproject_full).unwrap();

        let json = serde_json::json!({
            "projects": [{
                "name": "Brm",
                "uproject_path": uproject_rel,
                "targets": ["BrmServer", "BrmEditor", "Brm"]
            }]
        })
        .to_string();
        let path = write_manifest(&dir, &json);

        let validated = ProjectManifest::new(path)
            .ingest()
            .unwrap()
            .validate()
            .unwrap();
        let store = validated.to_triple_store();

        // Brm has name + path + 3 targets = 5 triples
        let executor = PatternExecutor(&store);
        let bindings = executor.select("SELECT * WHERE { ?s ?p ?o }").unwrap();
        assert_eq!(bindings.len(), 5);

        // Check that the subject IRI for Brm appears
        let subject = Term::Named("ue:project:Brm".into());
        let has_brm_subject = bindings.iter().any(|b| b.get("s") == Some(&subject));
        assert!(has_brm_subject, "expected ue:project:Brm as a subject");
    }

    // ------------------------------------------------------------------
    // 13. to_json() round-trips the project list
    // ------------------------------------------------------------------

    #[test]
    fn test_to_json_round_trips_project_data() {
        let dir = tempfile::tempdir().unwrap();
        let uproject_rel = "RealisticRendering/RealisticRendering.uproject";
        let full = dir.path().join(uproject_rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::File::create(&full).unwrap();

        let json = serde_json::json!({
            "projects": [{
                "name": "RealisticRendering",
                "uproject_path": uproject_rel,
                "targets": []
            }]
        })
        .to_string();
        let path = write_manifest(&dir, &json);

        let validated = ProjectManifest::new(path)
            .ingest()
            .unwrap()
            .validate()
            .unwrap();
        let output = validated.to_json();
        let reparsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let projects = reparsed["projects"].as_array().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0]["name"].as_str().unwrap(), "RealisticRendering");
    }

    // ------------------------------------------------------------------
    // 14. project_to_triples() subject IRI is ue:project:<NAME>
    // ------------------------------------------------------------------

    #[test]
    fn test_project_to_triples_subject_iri() {
        let project = UeProject {
            name: "FullSpectrum".into(),
            uproject_path: "FullSpectrum/FullSpectrum.uproject".into(),
            targets: vec![],
        };
        let triples = project_to_triples(&project);
        let expected_subject = Term::Named("ue:project:FullSpectrum".into());
        assert!(
            triples.iter().all(|t| t.subject == expected_subject),
            "all triples should share the same subject IRI"
        );
    }
}
