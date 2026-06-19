# Synthesis of Ggen Pack Specification Research

This document synthesizes the findings from the three parallel Explorer subagents (`explorer_ggen_spec_1`, `explorer_ggen_spec_2`, and `explorer_ggen_spec_3`) regarding the `ggen.toml` schema and validation logic.

## Consensus Schema Structure

The `ggen.toml` file functions as a dual-purpose configuration parsed by two crates:
1. `GgenManifest` (in `crates/ggen-core/src/manifest/types.rs`) for code generation tasks.
2. `GgenConfig` (in `crates/ggen-config/src/config_lib/schema.rs`) for runtime/infrastructure daemon settings (AI model, MCP, A2A, etc.).

### 1. Project Configuration (`[project]`)
- `name` (String, required): Used in generated headers.
- `version` (String, required): Project version.
- `description` (Option<String>): Project summary.
- `authors` (Option<Vec<String>>): List of authors.
- `license` (Option<String>): SPDX license string.
- `repository` (Option<String>): VCS URL.

### 2. Ontology Graph Sources (`[ontology]`)
- `source` (PathBuf, required): Path to primary Turtle (`.ttl`) file.
- `imports` (Vec<PathBuf>, optional): Paths to other RDF graph files.
- `base_iri` (Option<String>): Base URI namespace.
- `prefixes` (BTreeMap<String, String>, optional): Prefix declarations.
- `standard_only` (Option<bool>): Restricts external vocabularies to schema.org, FOAF, Dublin Core, SKOS, Big Five.

### 3. SPARQL Inference Rules (`[inference]`)
- `rules` (Vec<InferenceRule>): List of inference steps.
- `max_reasoning_timeout_ms` (u64, default `5000`): Reasoner timeout.
- `InferenceRule`:
  - `name` (String, required): Rule ID.
  - `description` (Option<String>): Human readable comment.
  - `construct` (String, required): SPARQL CONSTRUCT query text.
  - `order` (i32, default `0`): Precedence (lower values run first).
  - `when` (Option<String>): SPARQL ASK query; skips rule if false.

### 4. Code Generation Rules (`[generation]` & `[[generation.rules]]`)
- **Global generation settings**:
  - `output_dir` (PathBuf, default `.`): Root directory for generated files.
  - `max_sparql_timeout_ms` (u64, default `5000`): SPARQL timeout.
  - `require_audit_trail` (bool, default `false`): Produces cryptographic lineage receipts.
  - `determinism_salt` (Option<String>): Salt for stable ID generation.
- **`GenerationRule`**:
  - `name` (String, required): Rule ID.
  - `query` (QuerySource, required untagged enum):
    - `Inline { inline: String }`
    - `File { file: PathBuf }`
    - `Pack { pack: String, output: String, file: PathBuf }`
  - `template` (TemplateSource, required untagged enum):
    - `Inline { inline: String }`
    - `File { file: PathBuf }`
    - `Pack { pack: String, output: String, file: PathBuf }`
    - `Git { git: String, branch: Option<String>, path: PathBuf }`
    - `Package { package: String, version: Option<String>, path: PathBuf }`
  - `output_file` (String, required): Output file path (supports Tera variables like `{{label}}`).
  - `skip_empty` (bool, default `false`): Skip file generation if the SELECT query has no results.
  - `mode` (GenerationMode, default `Create`):
    - `Create`: Error out if file exists.
    - `Overwrite`: Replaces the file content entirely.
    - `Merge`: Uses markup tags to merge custom code.
  - `when` (Option<String>): SPARQL ASK query; runs generation only if true.

### 5. Validation Controls (`[validation]`)
- `strict_mode` (bool, default `false`): Elevates warnings to hard errors.

### 6. Dependency Packs (`[[packs]]`)
- `name` (String): Pack identifier.
- `path` (PathBuf): Path to dependency.

## Unified Validation Error Guards
The validator in `crates/ggen-core/src/manifest/validation.rs` enforces four primary safety checks:
1. **E0010 (VALUES Inline Guard)**: External queries (`QuerySource::File`) must not contain inline `VALUES` clauses. Such clauses must be declared inline within the manifest via `QuerySource::Inline`.
2. **E0011 (Inference Query Determinism)**: Under `strict_mode = true`, CONSTRUCT queries must include `ORDER BY` to maintain deterministic output.
3. **E0013 (Generation Query Determinism)**: Under `strict_mode = true`, SELECT queries must include `ORDER BY` to guarantee stable rendering.
4. **E0014 (Pack Dependency Guard)**: Any query/template referencing a pack must have that pack declared in the manifest's `[[packs]]` array.

## BIG BANG 80/20 Paradigm
A project health checkpoint that mandates "Specification Closure First":
1. **Real Data**: Actual CSV/JSON files must exist.
2. **Standard Ontology**: Standard schemas (FOAF, schema.org, etc.) must be identifiable within 5 minutes.
3. **One-Sentence Problem**: Problem must be describable in one sentence.
4. **Committed Users**: Non-founder commitment proof (email, contract, or payment).
5. **Real-User Validation**: Model can be validated with 10 real users in 48 hours.
