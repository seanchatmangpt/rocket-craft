# analysis.md — ggen.toml Schema & Configuration Analysis

## Executive Summary
This analysis documents the complete schema, parsing pathways, and design doctrines governing the `ggen.toml` configuration file in the `ggen` repository. The investigation reveals a dual-schema architecture: `GgenManifest` (compiled by `ggen-core` for deterministic code generation) and `GgenConfig` (compiled by `ggen-config` for runtime infrastructure, AI providers, MCP, and agent communication).

---

## 1. Dual-Schema Architecture of `ggen.toml`
The `ggen.toml` file functions as a unified project configuration, but it is parsed by two different crates depending on the operational context:

### A. Core Generator Manifest (`GgenManifest`)
- **Crate**: `ggen-core` (Source: `crates/ggen-core/src/manifest/types.rs`)
- **Purpose**: Defines the inputs (ontologies), transformation rules (SPARQL inference and generation), and validation gates (SHACL and SPARQL validation) that produce reproducible code.
- **Strict Validation**: The parser enforces strict schema boundaries via `#[serde(deny_unknown_fields)]` on sub-structures, though the root manifest ignores runtime configuration blocks (`logging`, `performance`, `telemetry`, etc.) by capturing them into unparsed TOML values.

### B. System/Infrastructure Configuration (`GgenConfig`)
- **Crate**: `ggen-config` (Source: `crates/ggen-config/src/config_lib/schema.rs`)
- **Purpose**: Governs runtime environment, AI models, Model Context Protocol (MCP) tool hosting, Agent-to-Agent (A2A) network communication, telemetry endpoints, and development overrides.

---

## 2. Structure of the Core Manifest Blocks (`GgenManifest`)

### 2.1 The `[project]` Block
Defines metadata about the target code package.
- **Fields**:
  - `name` (String, required): Project identifier used in generated code headers.
  - `version` (String, required): Semantic version of the code project.
  - `description` (Option<String>): Brief summary of the generated codebase.
  - `authors` (Option<Vec<String>>): List of project developers.
  - `license` (Option<String>): SPDX license identifier.
  - `repository` (Option<String>): Project repository URL.

### 2.2 The `[ontology]` Block
Configures how the RDF graph structure is resolved and verified.
- **Fields**:
  - `source` (PathBuf, required): Relative path to the primary Turtle (`.ttl`) ontology file.
  - `imports` (Vec<PathBuf>, optional): Array of paths to additional RDF files to merge into the graph database.
  - `base_iri` (Option<String>): Default namespace prefix for relative subject/predicate IRIs.
  - `prefixes` (BTreeMap<String, String>, optional): Delineates prefix mappings used in query scopes (e.g., `schema = "https://schema.org/"`).
  - `standard_only` (Option<bool>): Enforcement flag that allows only standard, established ontologies (e.g., schema.org, FOAF, Dublin Core, SKOS, Big Five) to pass build gates.

### 2.3 The `[inference]` Block & Rules
Defines an ordered chain of SPARQL CONSTRUCT queries executed before code generation. These are crucial for enriching the graph database with derived facts (e.g., transitively mapping inheritance or applying standard normalizations).
- **Block Fields**:
  - `rules` (Vec<InferenceRule>): List of inference rules executed sequentially.
  - `max_reasoning_timeout_ms` (u64, default = `5000`): Hard deadline for inference operations.
- **`InferenceRule` Fields**:
  - `name` (String, required): Unique identifier for the audit trail.
  - `construct` (String, required): SPARQL CONSTRUCT query representing the rule logic.
  - `description` (Option<String>): Description of the reasoning hypothesis.
  - `order` (i32, default = `0`): Lower numbers run earlier.
  - `when` (Option<String>): SPARQL ASK query condition; the rule is skipped if the condition evaluates to false.

### 2.4 The `[generation]` and `[[generation.rules]]` Block
Delineates the core code generation rules, mapping queries onto Tera template engines.
- **Block Fields**:
  - `rules` (Vec<GenerationRule>, required): Array of rules (represented in TOML as `[[generation.rules]]`).
  - `output_dir` (PathBuf, default = `.`): Root directory for generated file placements.
  - `max_sparql_timeout_ms` (u64, default = `5000`): Maximum time allowed per SELECT query.
  - `require_audit_trail` (bool, default = `false`): Generates an `audit.json` mapping each generated file back to the query results and cryptographic seeds that created it.
  - `determinism_salt` (Option<String>): Salt applied to generate stable UUIDs/IRIs.
  - `enable_llm`, `llm_provider`, `llm_model`: AI generation features.
- **`GenerationRule` Fields**:
  - `name` (String, required): Identifier for the rule.
  - `query` (QuerySource, required): Mapped enum determining how to retrieve the SELECT query (see sources below).
  - `template` (TemplateSource, required): Mapped enum determining how to retrieve the Tera template (see sources below).
  - `output_file` (String, required): Path pattern for the generated file, supporting variable injection (e.g., `src/models/{{label}}.rs`).
  - `skip_empty` (bool, default = `false`): If the query yields no rows, skip creating/overwriting the file.
  - `mode` (GenerationMode, default = `Create`): Handles pre-existing files:
    - `Create`: Generates only on the first pass; fails on subsequent passes if the file is present (safe scaffolding).
    - `Overwrite`: Replaces the file content entirely.
    - `Merge`: Combines generated contents with manual code using markers.
  - `when` (Option<String>): SPARQL ASK query; if it fails, skips file generation.

---

## 3. Enumerated Sources for Queries and Templates

### 3.1 QuerySource Mappings
The system supports three ways to fetch a SPARQL query, parsed as an untagged TOML enum:
1. **File Source**: `{ file = "queries/structs.sparql" }` - Reads from local file.
2. **Inline Source**: `{ inline = "SELECT ?class ... WHERE { ... }" }` - Embeds query directly in `ggen.toml`.
3. **Pack Source**: `{ pack = "wasm4pm-compat", output = "queries", file = "pm-rust-bridge.rq" }` - Retrieves the query from a declared pack dependency.

### 3.2 TemplateSource Mappings
Templates are rendered via the Tera engine and fetched using five options:
1. **File Source**: `{ file = "templates/rust-struct.tera" }` - Local filesystem path.
2. **Inline Source**: `{ inline = "struct {{name}} { ... }" }` - Inline Tera text.
3. **Pack Source**: `{ pack = "wasm4pm-compat", output = "templates", file = "rust-struct.tera" }` - References a pack's named output directory.
4. **Git Source**: `{ git = "https://github.com/org/repo", branch = "main", path = "template.tera" }` - Git-based delivery.
5. **Package Source**: `{ package = "core-templates", version = "^1.0.0", path = "rust.tera" }` - Fetches from the ggen marketplace registry.

---

## 4. BIG BANG 80/20 Paradigm and Reference Criteria

The "BIG BANG 80/20" doctrine is a non-negotiable development paradigm. It mandates **Specification Closure First**—verifying that the RDF specification is 100% complete and valid *before* generating code, avoiding subjective narrative feedback and code-level iterations.

### The 5 Reference Gate Criteria
Before running `ggen sync`, developers must verify five conditions:
1. **Real Data**: Do you have real user data (CSV/JSON)? Not promised—actual files.
2. **Standard Ontology**: Can you find one existing standard ontology (schema.org, FOAF, Dublin Core, SKOS)? Should take 5 minutes. If it takes 3 months, you're building custom (wrong path).
3. **One-Sentence Problem**: Can you explain your problem in one sentence? No 100-page documents.
4. **Committed Users**: Has anyone (not a friend, not a co-founder) committed to this? Email, contract, payment—proof, not enthusiasm.
5. **Validation Potential**: Can you validate with 10 real users in 48 hours?

*If any response is negative, the instruction is to stop and escalate/re-architect before proceeding.*
