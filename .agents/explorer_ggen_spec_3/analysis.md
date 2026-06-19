# Ggen Manifest Schema Analysis (ggen.toml)

This document provides a comprehensive analysis of the `ggen.toml` manifest schema, its structural representation in the codebase, and the validation gates that govern its usage (including the "BIG BANG 80/20" criteria).

---

## 1. Schema Overview

The core manifest is parsed into a strongly-typed `GgenManifest` struct (defined in `crates/ggen-core/src/manifest/types.rs`), which is distinct from the more complex `GgenConfig` configuration struct in `crates/ggen-config`. The manifest uses `BTreeMap` and standard vectors to maintain a deterministic serialization order.

### [project] Block
The project block contains metadata about the project.
* **Rust Representation (`ProjectConfig`)**:
  ```rust
  pub struct ProjectConfig {
      pub name: String,
      pub version: String,
      pub description: Option<String>,
      pub authors: Option<Vec<String>>,
      pub license: Option<String>,
      pub repository: Option<String>,
  }
  ```
* **Key Fields & Validation**:
  - `name` (String, Required): Must not be empty. Used in generated code headers.
  - `version` (String, Required): Must not be empty. Must comply with basic semantic versioning format (`x.y.z`).
  - `description`, `authors`, `license`, `repository` (Optional).

---

### [ontology] Block (Graph Sources)
The ontology block defines where RDF graphs and prefixes are located.
* **Rust Representation (`OntologyConfig`)**:
  ```rust
  pub struct OntologyConfig {
      pub source: PathBuf,
      pub imports: Vec<PathBuf>,
      pub base_iri: Option<String>,
      pub prefixes: BTreeMap<String, String>,
      pub standard_only: Option<bool>,
  }
  ```
* **Key Fields & Validation**:
  - `source` (PathBuf, Required): Path to the primary ontology `.ttl` file. Resolved relative to `ggen.toml`. The parser validates that the file exists.
  - `imports` (Vec<PathBuf>, Optional): Additional ontology file paths. Validator verifies that every import file exists.
  - `base_iri` (String, Optional): Base IRI for relative URIs.
  - `prefixes` (BTreeMap<String, String>, Optional): Prefix mappings for SPARQL queries.
  - `standard_only` (bool, Optional): When set to `true`, enforces that only approved standard ontologies are referenced (Dublin Core, schema.org, FOAF, SKOS, Big Five).

---

### [inference] Block (SPARQL CONSTRUCT Rules)
Inference rules specify a list of SPARQL CONSTRUCT queries executed in a specific order to expand or transform the RDF graph before code generation.
* **Rust Representation (`InferenceConfig` and `InferenceRule`)**:
  ```rust
  pub struct InferenceConfig {
      pub rules: Vec<InferenceRule>,
      pub max_reasoning_timeout_ms: u64, // Default 5000 ms
  }

  pub struct InferenceRule {
      pub name: String,
      pub description: Option<String>,
      pub construct: String,
      pub order: i32, // Default 0
      pub when: Option<String>,
  }
  ```
* **Key Fields & Validation**:
  - `name` (String, Required): Must not be empty. Used in audit trails.
  - `construct` (String, Required): The SPARQL CONSTRUCT query text. Must not be empty.
  - `order` (i32, Optional, Default 0): Determines execution precedence (lower values executed first).
  - `when` (String, Optional): A SPARQL ASK query. The inference rule is skipped if this query evaluates to false.
  - **Determinism Guard (E0011)**: In `strict_mode`, all `CONSTRUCT` queries must contain an `ORDER BY` clause. If missing, it results in a hard validation error. If `strict_mode` is false, it logs a warning.

---

### [[generation.rules]] Block (SELECT Queries and Templates)
An array of code generation rules that extract graph values using SPARQL SELECT queries and render them to output files using Tera templates.
* **Rust Representation (`GenerationConfig` and `GenerationRule`)**:
  ```rust
  pub struct GenerationConfig {
      pub rules: Vec<GenerationRule>,
      pub max_sparql_timeout_ms: u64, // Default 5000 ms
      pub require_audit_trail: bool,
      pub determinism_salt: Option<String>,
      pub output_dir: PathBuf, // Default "."
      pub enable_llm: bool,
      pub llm_provider: Option<String>,
      pub llm_model: Option<String>,
  }

  pub struct GenerationRule {
      pub name: String,
      pub query: QuerySource,
      pub template: TemplateSource,
      pub output_file: String,
      pub skip_empty: bool,
      pub mode: GenerationMode,
      pub when: Option<String>,
  }
  ```
* **QuerySource Options (Untagged Enum)**:
  - `Inline { inline: String }`: Raw SPARQL SELECT query text.
  - `File { file: PathBuf }`: Path to a `.sparql` or `.rq` file.
  - `Pack { pack: String, output: String, file: PathBuf }`: Query loaded from a dependency pack.
* **TemplateSource Options (Untagged Enum)**:
  - `Inline { inline: String }`: Raw Tera template string.
  - `File { file: PathBuf }`: Path to a `.tera` file.
  - `Pack { pack: String, output: String, file: PathBuf }`: Template loaded from a dependency pack.
  - `Git { git: String, branch: Option<String>, path: PathBuf }`: Git repository template source.
  - `Package { package: String, version: Option<String>, path: PathBuf }`: Package manager template source.
* **GenerationMode Enum**:
  - `Create` (Default): Fails if the output file already exists (protects custom code/scaffolding).
  - `Overwrite`: Overwrites the file on every generation run.
  - `Merge`: Combines generated code with handwritten code using marker tags.
* **Key Validations**:
  - **Values Inline Guard (E0010)**: SPARQL query files (`QuerySource::File`) must **never** contain inline `VALUES` clauses. All `VALUES` data must remain inline inside `ggen.toml` (e.g. using `QuerySource::Inline`). External `.rq` files are restricted to queries against real RDF triples on disk.
  - **Determinism Guard (E0013)**: In `strict_mode`, all SELECT queries (except pack-derived ones) must contain an `ORDER BY` clause to guarantee deterministic rendering.
  - **Pack Declared Guard (E0014)**: Any pack referenced in `QuerySource::Pack` or `TemplateSource::Pack` must be declared in the root `[[packs]]` array.
  - `output_file` must not be empty. Can contain Tera variables (e.g. `"src/models/{{className}}.rs"`).

---

## 2. The "BIG BANG 80/20" Criteria

The "BIG BANG 80/20" is ggen's core philosophical constraint: **Specification Closure First**.
Before code generation is allowed to proceed, the RDF specification must be 100% complete. There is no iteration on generated code; developers fix issues by modifying the `.ttl` source graph and regenerating.

As defined in the reference `ggen.toml`, the following 5 criteria must be met before proceeding with ggen-based automation:

1. **Real User Data**: Do you have real user data (CSV/JSON)? Not promised—actual files.
2. **Standard Ontology**: Can you find one existing standard ontology (schema.org, FOAF, Dublin Core, SKOS, Big Five)? If it takes 3 months to construct a custom schema, it is the wrong path. It should be identifiable in 5 minutes.
3. **One-Sentence Problem**: Can you explain your problem in a single sentence? Long, rambling documents are rejected.
4. **Committed Commitment**: Has someone (other than a friend or co-founder) committed to the project with an email, contract, or payment? (Proof, not enthusiasm).
5. **Real-User Validation**: Can you validate the model with 10 real users in 48 hours?

### Implementation Details:
- **`standard_only = true`**: When enabled in `[ontology]`, the system strictly validates that all imported schemas or namespaces are standard, standardizing naming patterns, namespaces, and preventing custom, non-interoperable Ontologies from cluttering code generation.
- **`ggen sync --validate-only`**: Used to verify the graph's structural closure (via SHACL and custom SPARQL ASK checks in `[validation]`) and produce cryptographic receipts before executing code generation.
