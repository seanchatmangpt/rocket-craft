#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceConfig {
    pub root: String,
    pub members: Vec<String>,
    pub default_target: String,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodegenConfig {
    pub ontology_dir: String,
    pub output_dir: String,
    pub targets: Vec<String>,
    pub template_dir: Option<String>,
    pub receipt_dir: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspConfig {
    pub servers: Vec<LspServerConfig>,
    pub conformance_threshold: f64,
    pub andon_enabled: bool,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CliConfig {
    pub default_namespace: String,
    pub json_output: bool,
    pub introspect_enabled: bool,
    pub chain_enabled: bool,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestConfig {
    pub style: String,
    pub coverage_threshold: f64,
    pub snapshot_dir: String,
    pub golden_dir: String,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct OtelConfig {
    pub endpoint: Option<String>,
    pub service_name: String,
    pub enabled: bool,
    pub ocel_export: bool,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct RdfConfig {
    pub default_namespace: String,
    pub ontologies: Vec<String>,
    pub sparql_endpoint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn workspace_default_has_empty_members() {
        let w = WorkspaceConfig::default();
        assert!(w.members.is_empty());
        assert!(w.root.is_empty());
    }

    #[test]
    fn codegen_default_has_no_optional_dirs() {
        let c = CodegenConfig::default();
        assert!(c.template_dir.is_none());
        assert!(c.receipt_dir.is_none());
    }

    #[test]
    fn lsp_default_conformance_threshold_is_zero() {
        let l = LspConfig::default();
        assert_eq!(l.conformance_threshold, 0.0);
        assert!(!l.andon_enabled);
    }

    #[test]
    fn lsp_server_config_default_empty_strings() {
        let s = LspServerConfig::default();
        assert!(s.name.is_empty());
        assert!(s.command.is_empty());
        assert!(s.args.is_empty());
    }

    #[test]
    fn cli_config_default_json_output_false() {
        let c = CliConfig::default();
        assert!(!c.json_output);
        assert!(!c.introspect_enabled);
        assert!(!c.chain_enabled);
    }

    #[test]
    fn otel_config_default_disabled() {
        let o = OtelConfig::default();
        assert!(!o.enabled);
        assert!(!o.ocel_export);
        assert!(o.endpoint.is_none());
    }

    #[test]
    fn rdf_config_serializes_cleanly() {
        let r = RdfConfig {
            default_namespace: "http://example.com/".into(),
            ontologies: vec!["core.ttl".into()],
            sparql_endpoint: Some("http://localhost:3030".into()),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("core.ttl"));
        assert!(json.contains("localhost:3030"));
    }

    #[test]
    fn test_config_default_coverage_zero() {
        let t = TestConfig::default();
        assert_eq!(t.coverage_threshold, 0.0);
        assert!(t.snapshot_dir.is_empty());
    }
}
