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
