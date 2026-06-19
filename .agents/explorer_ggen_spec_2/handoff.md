# Handoff Report — ggen.toml Schema and Big Bang 80/20 Research

## 1. Observation
The local repository at `/Users/sac/ggen/` was investigated to locate the structure, schema, and processing details of the `ggen.toml` file.

### Observation 1: Reference `ggen.toml` Configuration
At `/Users/sac/ggen/ggen.toml`, the default configuration file contains the following active configuration keys:
```toml
[project]
name = "my-ggen-project"
version = "0.1.0"
...
[ontology]
source = "schema/domain.ttl"
standard_only = true
...
[inference]
rules = [
    { name = "standard-normalization", construct = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }" }
]
...
[[generation.rules]]
name = "example-rule"
query = { inline = """...""" }
template = { file = "templates/example.txt.tera" }
output_file = "ontology-summary.txt"
mode = "Overwrite"
```

### Observation 2: The "BIG BANG 80/20" Criteria
At `/Users/sac/ggen/ggen.toml` lines 8-19, the following comment block details the "BIG BANG 80/20" gate criteria:
```toml
# BIG BANG 80/20: Specification Closure First
# Before using ggen, confirm:
# 1. Do you have real user data (CSV/JSON)? Not promised—actual files.
# 2. Can you find one existing standard ontology (schema.org, FOAF, Dublin Core, SKOS)?
#    Should take 5 minutes. If it takes 3 months, you're building custom (wrong path).
# 3. Can you explain your problem in one sentence? No 100-page documents.
# 4. Has anyone (not a friend, not a co-founder) committed to this?
#    Email, contract, payment—proof, not enthusiasm.
# 5. Can you validate with 10 real users in 48 hours?
#
# If you answered NO to any of these, stop. Talk to Sean before proceeding.
```
Additionally, `/Users/sac/ggen/README.md` lines 431-439 defines the paradigm as:
```markdown
### 1. Big Bang 80/20: Specification Closure First

**What it means**: Verify that your RDF specification is 100% complete *before* generating any code. No iteration on generated artifacts -- fix the specification and regenerate.
```

### Observation 3: Core Manifest Schema types in `ggen-core`
In `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs`, the core manifest structures for `ggen.toml` are parsed into Rust structures:
- **`GgenManifest`** (Root manifest):
  ```rust
  pub struct GgenManifest {
      pub project: ProjectConfig,
      pub ontology: OntologyConfig,
      #[serde(default)]
      pub inference: InferenceConfig,
      pub generation: GenerationConfig,
      #[serde(default)]
      pub validation: ValidationConfig,
      #[serde(default)]
      pub packs: Vec<PackRef>,
      ...
  }
  ```
- **`OntologyConfig`**:
  ```rust
  pub struct OntologyConfig {
      pub source: PathBuf,
      #[serde(default)]
      pub imports: Vec<PathBuf>,
      #[serde(default)]
      pub base_iri: Option<String>,
      #[serde(default)]
      pub prefixes: BTreeMap<String, String>,
      #[serde(default)]
      pub standard_only: Option<bool>,
  }
  ```
- **`InferenceConfig`** & **`InferenceRule`**:
  ```rust
  pub struct InferenceConfig {
      #[serde(default)]
      pub rules: Vec<InferenceRule>,
      #[serde(default = "default_reasoning_timeout")]
      pub max_reasoning_timeout_ms: u64,
  }

  pub struct InferenceRule {
      pub name: String,
      #[serde(default)]
      pub description: Option<String>,
      pub construct: String,
      #[serde(default)]
      pub order: i32,
      #[serde(default)]
      pub when: Option<String>,
  }
  ```
- **`GenerationRule`**:
  ```rust
  pub struct GenerationRule {
      pub name: String,
      pub query: QuerySource,
      pub template: TemplateSource,
      pub output_file: String,
      #[serde(default)]
      pub skip_empty: bool,
      #[serde(default)]
      pub mode: GenerationMode,
      #[serde(default)]
      pub when: Option<String>,
  }
  ```
- **`QuerySource`** & **`TemplateSource`** (untagged enums mapped from TOML blocks):
  ```rust
  pub enum QuerySource {
      Pack { pack: String, output: String, file: PathBuf },
      File { file: PathBuf },
      Inline { inline: String },
  }

  pub enum TemplateSource {
      Pack { pack: String, output: String, file: PathBuf },
      File { file: PathBuf },
      Inline { inline: String },
      Git { git: String, branch: Option<String>, path: PathBuf },
      Package { package: String, version: Option<String>, path: PathBuf },
  }
  ```

### Observation 4: System/Infrastructure Schema types in `ggen-config`
In `/Users/sac/ggen/crates/ggen-config/src/config_lib/schema.rs`, the root structure is `GgenConfig` which includes daemon configs (`ai`, `templates`, `rdf`, `sparql`, `lifecycle`, `security`, `performance`, `logging`, `telemetry`, `mcp`, `a2a`).

---

## 2. Logic Chain
1. By reading `/Users/sac/rocket-craft/.agents/orchestrator_ggen_spec/PROJECT.md` (Observation 1 of scope file), the investigation was scoped to locate the specification of `ggen.toml` with `[inference]` rules using `CONSTRUCT` queries and `[[generation.rules]]` using `SELECT` queries + `.tera` templates.
2. Direct listing of the `/Users/sac/ggen/` folder revealed two crates dedicated to configuration: `ggen-config` and `ggen-core`.
3. Reading `crates/ggen-config/src/config_lib/schema.rs` (Observation 4) showed a broad infrastructure schema featuring `mcp`, `a2a`, and `ai` configs.
4. Reading `crates/ggen-core/src/manifest/types.rs` (Observation 3) showed the parsing structures for the code generation workflow blocks (`[ontology]`, `[inference]`, `[[generation.rules]]`, `[validation]`).
5. Reading `ggen.toml` (Observation 2) verified the existence of the comments defining the "BIG BANG 80/20" criteria, which were corroborated by the philosophy section in `README.md` (Observation 2).

---

## 3. Caveats
- The system configuration `GgenConfig` from `ggen-config` and `GgenManifest` from `ggen-core` are separate types on disk. Their integration relies on the parser ignoring unrelated keys. Any specification document should account for this dual-nature parsing structure.
- No code was executed or modified, in accordance with the read-only investigation constraint.

---

## 4. Conclusion
The canonical schema of `ggen.toml` has been identified. It combines runtime daemon settings (via `GgenConfig`) and target compile-time manifest parameters (via `GgenManifest`). Code generation is strictly specification-first (governed by the BIG BANG 80/20 principle), requiring complete RDF graphs, optional CONSTRUCT-based reasoning rules, and SELECT-based templates before outputs are compiled.

---

## 5. Verification Method
To verify the schemas independently:
1. Examine the struct fields in `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs` using `view_file`.
2. Inspect the comments in `/Users/sac/ggen/ggen.toml` for the "BIG BANG 80/20" list.
3. If necessary, execute cargo validation tests in `/Users/sac/ggen` using `cargo test -p ggen-core manifest::parser::tests` (requires test execution command if allowed).
