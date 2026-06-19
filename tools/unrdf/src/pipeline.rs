use crate::store::TripleStore;
use crate::triple::{Term, Triple};

/// Errors that can occur during pipeline execution.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// An error occurred while loading RDF content.
    #[error("RDF load error: {0}")]
    Load(String),
    /// An error occurred during the rendering stage.
    #[error("Render error: {0}")]
    Render(String),
}

/// Configuration for the ontology pipeline.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineConfig {
    /// Target language: "rust", "go", "typescript", "elixir", "python", etc.
    pub target_language: String,
    /// Output directory for generated files.
    pub output_dir: String,
    /// Optional custom template directory.
    pub template_dir: Option<String>,
    /// Base namespace IRI.
    pub namespace: String,
}

/// A single generated file produced by the pipeline.
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    /// Kind discriminator, e.g. "struct", "interface", "module".
    pub kind: String,
}

/// The output of a completed pipeline run.
pub struct PipelineOutput {
    pub files: Vec<GeneratedFile>,
    /// BLAKE3 hash of all output file contents concatenated.
    pub receipt: String,
}

/// Mirrors ggen's 5-stage μ₁–μ₅ pipeline:
///   μ₁ ontology load → μ₂ SPARQL extraction → μ₃ template rendering
///   → μ₄ canonicalization → μ₅ receipt
pub struct OntologyPipeline {
    store: TripleStore,
    config: PipelineConfig,
}

impl OntologyPipeline {
    pub fn new(store: TripleStore, config: PipelineConfig) -> Self {
        OntologyPipeline { store, config }
    }

    /// μ₁: Load RDF/Turtle content.
    /// This is a minimal implementation: blank lines and comment lines are skipped;
    /// each remaining line is parsed as a simple N-Triples-style triple.
    /// Returns the number of triples added.
    pub fn load_turtle(&mut self, turtle: &str) -> Result<usize, PipelineError> {
        let before = self.store.len();
        for line in turtle.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Accept simple "subject predicate object ." lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let s = parts[0].trim_matches(|c| c == '<' || c == '>');
                let p = parts[1].trim_matches(|c| c == '<' || c == '>');
                let o = parts[2].trim_matches(|c| c == '<' || c == '>' || c == '.');
                self.store.add(Triple::new(s, p, o));
            }
        }
        Ok(self.store.len() - before)
    }

    /// μ₂: Extract rdf:type subjects from the store.
    pub fn extract_types(&self) -> Vec<String> {
        let rdf_type = Term::Named("rdf:type".into());
        self.store
            .query_predicate(&rdf_type)
            .into_iter()
            .map(|t| match &t.subject {
                Term::Named(iri) => iri.clone(),
                Term::Blank(id) => format!("_:{}", id),
                Term::Literal { value, .. } => value.clone(),
            })
            .collect()
    }

    /// μ₃–μ₅: Template rendering, canonicalization, and receipt generation.
    pub fn render(&self) -> Result<PipelineOutput, PipelineError> {
        fn sanitize_field_name(name: &str) -> String {
            let mut clean = String::new();
            for c in name.chars() {
                if c.is_alphanumeric() || c == '_' {
                    clean.push(c);
                } else {
                    clean.push('_');
                }
            }
            if clean.is_empty() {
                return "field".to_string();
            }
            if clean.starts_with(|c: char| c.is_ascii_digit()) {
                clean = format!("_{}", clean);
            }
            clean
        }

        fn capitalize(s: &str) -> String {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }

        let rdf_type = Term::Named("rdf:type".into());
        let type_triples = self.store.query_predicate(&rdf_type);

        let mut files = Vec::new();
        let mut processed_types = std::collections::HashSet::new();

        for type_triple in &type_triples {
            let ty_term = &type_triple.subject;
            let ty = match ty_term {
                Term::Named(iri) => iri.clone(),
                Term::Blank(id) => format!("_:{}", id),
                Term::Literal { value, .. } => value.clone(),
            };

            if !processed_types.insert(ty.clone()) {
                continue;
            }

            let type_name = ty.split('/').next_back().unwrap_or(&ty);
            let type_name = type_name.split('#').next_back().unwrap_or(type_name);
            let type_name = type_name.split(':').next_back().unwrap_or(type_name);

            // Query properties for this type.
            let subject_triples = self.store.query_subject(ty_term);
            let mut props = Vec::new();
            for st in subject_triples {
                if st.predicate == rdf_type {
                    continue;
                }

                let pred_iri = match &st.predicate {
                    Term::Named(iri) => iri.clone(),
                    Term::Blank(id) => format!("_:{}", id),
                    Term::Literal { value, .. } => value.clone(),
                };

                let raw_prop_name = pred_iri.split('/').next_back().unwrap_or(&pred_iri);
                let raw_prop_name = raw_prop_name
                    .split('#')
                    .next_back()
                    .unwrap_or(raw_prop_name);
                let raw_prop_name = raw_prop_name
                    .split(':')
                    .next_back()
                    .unwrap_or(raw_prop_name);

                let datatype = match &st.object {
                    Term::Literal { datatype, .. } => datatype.as_deref(),
                    _ => None,
                };

                props.push((raw_prop_name.to_string(), datatype.map(String::from)));
            }

            // Deduplicate and sort properties for deterministic output
            props.sort_by(|a, b| a.0.cmp(&b.0));
            props.dedup_by(|a, b| a.0 == b.0);

            // Render based on target language
            let (ext, content) = match self.config.target_language.as_str() {
                "rust" => {
                    if props.is_empty() {
                        (
                            "rs",
                            format!("// Generated by unify-rdf\npub struct {} {{}}\n", type_name),
                        )
                    } else {
                        let mut fields = String::new();
                        for (prop_name, datatype) in &props {
                            let clean_name = sanitize_field_name(prop_name);
                            let rust_type = match datatype.as_deref() {
                                Some("xsd:integer") | Some("integer") | Some("xsd:int")
                                | Some("int") => "i32",
                                Some("xsd:boolean") | Some("boolean") => "bool",
                                Some("xsd:float") | Some("float") | Some("xsd:double")
                                | Some("double") => "f64",
                                _ => "String",
                            };
                            fields.push_str(&format!("    pub {}: {},\n", clean_name, rust_type));
                        }
                        (
                            "rs",
                            format!(
                                "// Generated by unify-rdf\npub struct {} {{\n{}}}\n",
                                type_name, fields
                            ),
                        )
                    }
                }
                "go" => {
                    if props.is_empty() {
                        (
                            "go",
                            format!(
                                "// Generated by unify-rdf\npackage {}\ntype {} struct {{}}\n",
                                self.config.namespace, type_name
                            ),
                        )
                    } else {
                        let mut fields = String::new();
                        for (prop_name, datatype) in &props {
                            let clean_name = capitalize(&sanitize_field_name(prop_name));
                            let go_type = match datatype.as_deref() {
                                Some("xsd:integer") | Some("integer") | Some("xsd:int")
                                | Some("int") => "int",
                                Some("xsd:boolean") | Some("boolean") => "bool",
                                Some("xsd:float") | Some("float") | Some("xsd:double")
                                | Some("double") => "float64",
                                _ => "string",
                            };
                            fields.push_str(&format!("    {} {}\n", clean_name, go_type));
                        }
                        (
                            "go",
                            format!(
                                "// Generated by unify-rdf\npackage {}\ntype {} struct {{\n{}}}\n",
                                self.config.namespace, type_name, fields
                            ),
                        )
                    }
                }
                "typescript" => {
                    if props.is_empty() {
                        (
                            "ts",
                            format!(
                                "// Generated by unify-rdf\nexport interface {} {{}}\n",
                                type_name
                            ),
                        )
                    } else {
                        let mut fields = String::new();
                        for (prop_name, datatype) in &props {
                            let clean_name = sanitize_field_name(prop_name);
                            let ts_type = match datatype.as_deref() {
                                Some("xsd:integer") | Some("integer") | Some("xsd:int")
                                | Some("int") => "number",
                                Some("xsd:boolean") | Some("boolean") => "boolean",
                                Some("xsd:float") | Some("float") | Some("xsd:double")
                                | Some("double") => "number",
                                _ => "string",
                            };
                            fields.push_str(&format!("    {}: {};\n", clean_name, ts_type));
                        }
                        (
                            "ts",
                            format!(
                                "// Generated by unify-rdf\nexport interface {} {{\n{}}}\n",
                                type_name, fields
                            ),
                        )
                    }
                }
                _ => (
                    "txt",
                    format!("// Generated by unify-rdf\n// Type: {}\n", type_name),
                ),
            };

            files.push(GeneratedFile {
                path: format!("{}/{}.{}", self.config.output_dir, type_name, ext),
                content,
                kind: "type".into(),
            });
        }

        // μ₄ canonicalization: sort by path for determinism
        files.sort_by(|a, b| a.path.cmp(&b.path));
        // μ₅ receipt: hash of all content
        let combined: String = files.iter().map(|f| f.content.as_str()).collect();
        let receipt = blake3::hash(combined.as_bytes()).to_hex().to_string();
        Ok(PipelineOutput { files, receipt })
    }

    /// Always 5 stages (μ₁–μ₅).
    pub fn stage_count(&self) -> usize {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> PipelineConfig {
        PipelineConfig {
            target_language: "rust".into(),
            output_dir: "out".into(),
            template_dir: None,
            namespace: "example".into(),
        }
    }

    #[test]
    fn test_pipeline_new_and_stage_count() {
        let pipeline = OntologyPipeline::new(TripleStore::new(), default_config());
        assert_eq!(pipeline.stage_count(), 5);
    }

    #[test]
    fn test_pipeline_load_turtle_increases_triple_count() {
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), default_config());
        let turtle = "<http://example.org/Foo> rdf:type <http://schema.org/Thing> .\n\
                      <http://example.org/Bar> rdf:type <http://schema.org/Thing> .";
        let added = pipeline.load_turtle(turtle).unwrap();
        assert_eq!(added, 2);
        assert_eq!(pipeline.store.len(), 2);
    }

    #[test]
    fn test_pipeline_extract_types_returns_rdf_type_subjects() {
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), default_config());
        let turtle = "<http://example.org/Foo> rdf:type <http://schema.org/Thing> .";
        pipeline.load_turtle(turtle).unwrap();
        let types = pipeline.extract_types();
        assert_eq!(types.len(), 1);
        assert!(types[0].contains("Foo"));
    }

    #[test]
    fn test_pipeline_render_produces_output() {
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), default_config());
        pipeline
            .load_turtle("<http://example.org/MyType> rdf:type <http://schema.org/Thing> .")
            .unwrap();
        let output = pipeline.render().unwrap();
        assert_eq!(output.files.len(), 1);
        assert!(!output.receipt.is_empty());
        assert!(output.files[0].content.contains("MyType"));
    }

    #[test]
    fn test_pipeline_render_with_properties_rust() {
        let mut config = default_config();
        config.target_language = "rust".into();
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), config);

        // Load class and properties
        pipeline
            .load_turtle("<http://example.org/Character> rdf:type <http://schema.org/Thing> .")
            .unwrap();

        // Add string property
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/name",
            Term::Literal {
                value: "Hero".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        ));

        // Add integer property
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/age",
            Term::Literal {
                value: "25".into(),
                datatype: Some("xsd:integer".into()),
                lang: None,
            },
        ));

        let output = pipeline.render().unwrap();
        assert_eq!(output.files.len(), 1);
        let content = &output.files[0].content;
        assert!(content.contains("pub struct Character"));
        assert!(content.contains("pub name: String"));
        assert!(content.contains("pub age: i32"));
    }

    #[test]
    fn test_pipeline_render_with_properties_go() {
        let mut config = default_config();
        config.target_language = "go".into();
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), config);

        pipeline
            .load_turtle("<http://example.org/Character> rdf:type <http://schema.org/Thing> .")
            .unwrap();
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/name",
            Term::Literal {
                value: "Hero".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        ));
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/age",
            Term::Literal {
                value: "25".into(),
                datatype: Some("xsd:integer".into()),
                lang: None,
            },
        ));

        let output = pipeline.render().unwrap();
        assert_eq!(output.files.len(), 1);
        let content = &output.files[0].content;
        assert!(content.contains("type Character struct"));
        assert!(content.contains("Name string"));
        assert!(content.contains("Age int"));
    }

    #[test]
    fn test_pipeline_render_with_properties_ts() {
        let mut config = default_config();
        config.target_language = "typescript".into();
        let mut pipeline = OntologyPipeline::new(TripleStore::new(), config);

        pipeline
            .load_turtle("<http://example.org/Character> rdf:type <http://schema.org/Thing> .")
            .unwrap();
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/name",
            Term::Literal {
                value: "Hero".into(),
                datatype: Some("xsd:string".into()),
                lang: None,
            },
        ));
        pipeline.store.add(Triple::new(
            "http://example.org/Character",
            "http://example.org/age",
            Term::Literal {
                value: "25".into(),
                datatype: Some("xsd:integer".into()),
                lang: None,
            },
        ));

        let output = pipeline.render().unwrap();
        assert_eq!(output.files.len(), 1);
        let content = &output.files[0].content;
        assert!(content.contains("export interface Character"));
        assert!(content.contains("name: string"));
        assert!(content.contains("age: number"));
    }
}
