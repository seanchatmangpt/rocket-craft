/// Errors that can occur while parsing a manifest.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// A required field was missing from the manifest.
    #[error("Manifest missing required field: {0}")]
    MissingField(String),
    /// An error occurred during TOML parsing.
    #[error("TOML parse error: {0}")]
    Parse(String),
}

/// Configuration for a single language generator.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneratorConfig {
    pub lang: String,
    pub template: Option<String>,
    pub out_dir: String,
}

/// Top-level manifest equivalent to ggen's `ggen.toml`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    /// List of ontology IRIs to load.
    pub ontologies: Vec<String>,
    pub generators: Vec<GeneratorConfig>,
    pub output: String,
}

impl Manifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml(toml: &str) -> Result<Self, ManifestError> {
        // Minimal hand-rolled TOML parser for the manifest fields.
        let mut name = String::new();
        let mut version = String::new();
        let mut ontologies: Vec<String> = Vec::new();
        let mut generators: Vec<GeneratorConfig> = Vec::new();
        let mut output = String::new();

        let mut current_generator: Option<GeneratorConfig> = None;
        let mut in_ontologies_array = false;

        for line in toml.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // --- Table / array-of-tables headers ---
            if line == "[[generators]]" {
                if let Some(gen) = current_generator.take() {
                    generators.push(gen);
                }
                current_generator = Some(GeneratorConfig {
                    lang: String::new(),
                    template: None,
                    out_dir: String::new(),
                });
                in_ontologies_array = false;
                continue;
            }
            // Standalone [ontologies] section header (no '=' on this line)
            if line == "[ontologies]" {
                in_ontologies_array = true;
                continue;
            }
            // Any other [section] header
            if line.starts_with('[') {
                in_ontologies_array = false;
                continue;
            }

            // --- Inline array: ontologies = ["...", "..."] ---
            // Check this BEFORE the generic key=value handler so we don't treat
            // "ontologies" as a top-level scalar field.
            if line.starts_with("ontologies") && line.contains('=') {
                let val = after_equals(line);
                // Strip surrounding [ ]
                let inner = val.trim_matches(|c| c == '[' || c == ']');
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"');
                    if !s.is_empty() {
                        ontologies.push(s.to_string());
                    }
                }
                in_ontologies_array = false;
                continue;
            }

            // --- Array element inside [ontologies] section ---
            if in_ontologies_array && line.starts_with('"') {
                let s = line.trim_matches(|c| c == '"' || c == ',');
                if !s.is_empty() {
                    ontologies.push(s.to_string());
                }
                continue;
            }

            // --- Generic key = value ---
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim().trim_matches('"');
                if let Some(gen) = current_generator.as_mut() {
                    match key {
                        "lang" => gen.lang = val.to_string(),
                        "template" => gen.template = Some(val.to_string()),
                        "out_dir" => gen.out_dir = val.to_string(),
                        _ => {
                            let _ignored = key;
                        }
                    }
                } else {
                    match key {
                        "name" => name = val.to_string(),
                        "version" => version = val.to_string(),
                        "output" => output = val.to_string(),
                        _ => {
                            let _ignored = key;
                        }
                    }
                }
            }
        }

        // Flush the last pending generator
        if let Some(gen) = current_generator {
            generators.push(gen);
        }

        if name.is_empty() {
            return Err(ManifestError::MissingField("name".into()));
        }
        Ok(Manifest {
            name,
            version,
            ontologies,
            generators,
            output,
        })
    }

    /// Serialize to a TOML string.
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("name = \"{}\"\n", self.name));
        out.push_str(&format!("version = \"{}\"\n", self.version));
        out.push_str(&format!("output = \"{}\"\n", self.output));
        // Ontologies as an inline TOML array
        let onto_list = self
            .ontologies
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("ontologies = [{}]\n", onto_list));
        // Generator tables
        for gen in &self.generators {
            out.push_str("\n[[generators]]\n");
            out.push_str(&format!("lang = \"{}\"\n", gen.lang));
            if let Some(tmpl) = &gen.template {
                out.push_str(&format!("template = \"{}\"\n", tmpl));
            }
            out.push_str(&format!("out_dir = \"{}\"\n", gen.out_dir));
        }
        out
    }

    /// Create a default manifest for the given name.
    pub fn default_for(name: &str) -> Self {
        Manifest {
            name: name.to_string(),
            version: "0.1.0".into(),
            ontologies: Vec::new(),
            generators: vec![GeneratorConfig {
                lang: "rust".into(),
                template: None,
                out_dir: "generated".into(),
            }],
            output: "out".into(),
        }
    }
}

fn after_equals(line: &str) -> &str {
    if let Some(pos) = line.find('=') {
        line[pos + 1..].trim()
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default_for_creates_valid_manifest() {
        let m = Manifest::default_for("my-ontology");
        assert_eq!(m.name, "my-ontology");
        assert_eq!(m.version, "0.1.0");
        assert!(!m.generators.is_empty());
        assert_eq!(m.generators[0].lang, "rust");
    }

    #[test]
    fn test_manifest_to_toml_from_toml_roundtrip() {
        let original = Manifest {
            name: "test-manifest".into(),
            version: "1.2.3".into(),
            ontologies: vec![
                "http://example.org/ont1".into(),
                "http://example.org/ont2".into(),
            ],
            generators: vec![
                GeneratorConfig {
                    lang: "rust".into(),
                    template: Some("templates/rust.hbs".into()),
                    out_dir: "src/generated".into(),
                },
                GeneratorConfig {
                    lang: "typescript".into(),
                    template: None,
                    out_dir: "ts/generated".into(),
                },
            ],
            output: "dist".into(),
        };

        let toml_str = original.to_toml();
        let parsed = Manifest::from_toml(&toml_str).expect("round-trip should succeed");

        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.output, original.output);
        assert_eq!(parsed.ontologies.len(), original.ontologies.len());
        assert_eq!(parsed.generators.len(), original.generators.len());
        assert_eq!(parsed.generators[0].lang, "rust");
        assert_eq!(
            parsed.generators[0].template.as_deref(),
            Some("templates/rust.hbs")
        );
        assert_eq!(parsed.generators[1].lang, "typescript");
        assert!(parsed.generators[1].template.is_none());
    }

    #[test]
    fn test_manifest_from_toml_missing_name_returns_error() {
        let toml = "version = \"1.0\"\noutput = \"out\"\n";
        let result = Manifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn default_for_sets_generator_out_dir() {
        let m = Manifest::default_for("my-crate");
        assert!(!m.generators[0].out_dir.is_empty(), "out_dir must not be empty");
    }

    #[test]
    fn default_for_output_is_non_empty() {
        let m = Manifest::default_for("my-crate");
        assert!(!m.output.is_empty(), "output dir must not be empty");
    }

    #[test]
    fn from_toml_invalid_syntax_returns_error() {
        let result = Manifest::from_toml("<<< this is not valid toml >>>");
        assert!(result.is_err(), "invalid TOML must return an error");
    }

    #[test]
    fn to_toml_includes_name_and_version() {
        let m = Manifest::default_for("rocket-rdf");
        let toml = m.to_toml();
        assert!(toml.contains("rocket-rdf"), "name must appear in serialized TOML");
        assert!(toml.contains("0.1.0"), "version must appear in serialized TOML");
    }
}
