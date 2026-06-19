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
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub out_dir: String,
}

/// Top-level manifest equivalent to ggen's `ggen.toml`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    /// List of ontology IRIs to load.
    #[serde(default)]
    pub ontologies: Vec<String>,
    #[serde(default)]
    pub generators: Vec<GeneratorConfig>,
    #[serde(default)]
    pub output: String,
}

impl Manifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml(toml: &str) -> Result<Self, ManifestError> {
        let manifest: Self =
            toml::from_str(toml).map_err(|e| ManifestError::Parse(e.to_string()))?;
        if manifest.name.is_empty() {
            return Err(ManifestError::MissingField("name".into()));
        }
        Ok(manifest)
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
    fn default_for_name_is_preserved() {
        let m = Manifest::default_for("my-crate");
        assert_eq!(m.name, "my-crate");
    }

    #[test]
    fn to_toml_round_trips_name() {
        let m = Manifest::default_for("rocket-rdf");
        let toml = m.to_toml();
        let parsed = Manifest::from_toml(&toml).unwrap();
        assert_eq!(parsed.name, "rocket-rdf");
    }

    #[test]
    fn from_toml_invalid_syntax_returns_error() {
        let result = Manifest::from_toml("<<< invalid >>>");
        assert!(result.is_err());
    }
}
