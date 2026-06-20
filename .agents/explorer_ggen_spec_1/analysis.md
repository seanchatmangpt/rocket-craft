# Analysis of ggen.toml Schema and Validation Rules

## Executive Summary
This document provides a detailed investigation into the schema, parsing, and validation behavior of the `ggen.toml` manifest, as implemented in the `/Users/sac/ggen/` repository. 

In `ggen`, software artifacts are treated as deterministic projections of knowledge graphs. The code generation process is orchestrated using a `ggen.toml` file, which specifies:
1. **Metadata:** Project details (`[project]`).
2. **Ontology Input:** Location of Turtle/RDF files (`[ontology]`).
3. **Inference Rules:** SPARQL CONSTRUCT queries to expand the graph (`[inference]`).
4. **Generation Rules:** SPARQL SELECT queries paired with Tera templates (`[[generation.rules]]`).
5. **Validation:** SHACL shape validation and custom SPARQL ASK quality checks (`[validation]`).
6. **Packs:** Dependency packs from registries or local directories (`[[packs]]`).

---

## 1. Root Manifest Structure (`GgenManifest`)
The parsing layer is implemented in Rust using `serde` to deserialize the TOML manifest into the `GgenManifest` struct defined in `crates/ggen-core/src/manifest/types.rs`. 

The core fields are:

| Field Name | Type | TOML Section | Description |
|---|---|---|---|
| `project` | `ProjectConfig` | `[project]` | Required metadata. |
| `ontology` | `OntologyConfig` | `[ontology]` | Required ontology sources. |
| `inference` | `InferenceConfig` | `[inference]` | Optional graph inference rules (runs before code generation). |
| `generation` | `GenerationConfig` | `[generation]` | Required generation config and `rules` array. |
| `validation` | `ValidationConfig` | `[validation]` | Optional quality validation rules (SHACL/ASK). |
| `packs` | `Vec<PackRef>` | `[[packs]]` | Optional pack references. |

### Handling of Ignored Sections
To maintain compatibility with downstream tool configurations (e.g., `ggen sync` configs handled by `ggen-config`), the parser defines several fields as `Option<toml::Value>` with `#[serde(default)]`. This prevents parsing failures when sections handled by other subsystems are present in the same file. These ignored sections include: `sync`, `rdf`, `templates`, `output`, `ai`, `sparql`, `lifecycle`, `security`, `performance`, `logging`, `telemetry`, `features`, and `env`.

---

## 2. Block-by-Block Schema Definition

### 2.1 `[project]` Block
Defines metadata utilized in output files or generated headers.
*   **`name`** (`String`, Required): Project identifier.
*   **`version`** (`String`, Required): Semantic version.
*   **`description`** (`Option<String>`): Short summary.
*   **`authors`** (`Option<Vec<String>>`): List of authors.
*   **`license`** (`Option<String>`): E.g., `"MIT"`.
*   **`repository`** (`Option<String>`): Repository URL.

### 2.2 `[ontology]` Block
Declares the base graph assets.
*   **`source`** (`PathBuf`, Required): Path to primary ontology Turtle file.
*   **`imports`** (`Vec<PathBuf>`, Default empty): Extra ontology file paths to merge.
*   **`base_iri`** (`Option<String>`): Namespace base URI.
*   **`prefixes`** (`BTreeMap<String, String>`, Default empty): Namespace mapping prefix keys to URI values.
*   **`standard_only`** (`Option<bool>`): Restricts the project to approved standard ontologies if `true`.

### 2.3 `[inference]` Block
Coordinates graph-expansion passes using standard SPARQL CONSTRUCT queries.
*   **`max_reasoning_timeout_ms`** (`u64`, Default: `5000`): Timeout for reasoning.
*   **`rules`** (`Vec<InferenceRule>`, Default empty): An array of inference rules. In TOML, this is represented as `[[inference.rules]]`.
    *   **`name`** (`String`, Required): Unique rule name.
    *   **`description`** (`Option<String>`): Rule description.
    *   **`construct`** (`String`, Required): SPARQL CONSTRUCT query.
    *   **`order`** (`i32`, Default: `0`): Lower values run earlier.
    *   **`when`** (`Option<String>`): Optional SPARQL ASK guard. Execution skips if the ASK query returns false.

### 2.4 `[generation]` & `[[generation.rules]]`
Directs code generation rules.
*   **`max_sparql_timeout_ms`** (`u64`, Default: `5000`): Limit for SPARQL select queries.
*   **`require_audit_trail`** (`bool`, Default: `false`): Produces a detailed `audit.json` trace.
*   **`determinism_salt`** (`Option<String>`): Deterministic IRI generation salt.
*   **`output_dir`** (`PathBuf`, Default: `.`): Code generation destination root.
*   **`enable_llm`** (`bool`, Default: `false`): Enables LLM-driven generation.
*   **`llm_provider`** (`Option<String>`): LLM provider name.
*   **`llm_model`** (`Option<String>`): Model name.
*   **`rules`** (`Vec<GenerationRule>`, Required): In TOML, represented as `[[generation.rules]]`.
    *   **`name`** (`String`, Required): Unique generation rule name.
    *   **`query`** (`QuerySource`, Required): SPARQL SELECT query (untagged enum).
    *   **`template`** (`TemplateSource`, Required): Tera template structure (untagged enum).
    *   **`output_file`** (`String`, Required): Destination template path (supports variables).
    *   **`skip_empty`** (`bool`, Default: `false`): Skips template rendering if query returns zero rows.
    *   **`mode`** (`GenerationMode`, Default: `Create`): Defines file emission protocol.
    *   **`when`** (`Option<String>`): Optional SPARQL ASK guard.

#### `QuerySource` (Untagged Serialization Union)
Can be declared in one of three ways:
1.  **Pack Output:** `{ pack = "<pack-name>", output = "<output-key>", file = "<relative-path>" }`
2.  **File:** `{ file = "<path-to-rq>" }`
3.  **Inline String:** `{ inline = "SELECT ... " }`

#### `TemplateSource` (Untagged Serialization Union)
Can be declared in one of five ways:
1.  **Pack Output:** `{ pack = "<pack-name>", output = "<output-key>", file = "<relative-path>" }`
2.  **File:** `{ file = "<path-to-tera>" }`
3.  **Inline String:** `{ inline = "template contents" }`
4.  **Git Repo:** `{ git = "<git-url>", branch = "<branch>", path = "<relative-path>" }`
5.  **Package Registry:** `{ package = "<package-name>", version = "<constraint>", path = "<relative-path>" }`

#### `GenerationMode` Enum
*   **`Create`** (Default): Emits file only on first run. Fails subsequent syncs if the file already exists (protects custom domain logic).
*   **`Overwrite`**: Always rewrites target output files.
*   **`Merge`**: Merges code sections using markers.

### 2.5 `[validation]` Block
Enforces quality metrics and syntax guards.
*   **`shacl`** (`Vec<PathBuf>`, Default empty): Paths to SHACL shapes files for graph verification.
*   **`validate_syntax`** (`bool`, Default: `false`): Runs compilation checks on generated code.
*   **`no_unsafe`** (`bool`, Default: `false`): Rejects generated Rust containing `unsafe`.
*   **`strict_mode`** (`bool`, Default: `false`): Promotes warnings to hard errors.
*   **`rules`** (`Vec<ValidationRule>`, Default empty): SPARQL-based custom validations (`[[validation.rules]]`).
    *   **`name`** (`String`): Identifier.
    *   **`description`** (`String`): Error message shown on fail.
    *   **`ask`** (`String`): SPARQL ASK query. The rule is valid only if this query evaluates to `true`.
    *   **`severity`** (`ValidationSeverity`, Default: `Error`): E.g., `Error` or `Warning`.

---

## 3. Strict Validation Errors (Poka-Yoke Gates)
The validation layer (`crates/ggen-core/src/manifest/validation.rs`) performs semantic analysis at load time. Four critical errors are defined:

1.  **`error[E0010]` (VALUES Inline Enforcement):** 
    VALUES clauses in SPARQL queries contain static mock or literal dataset structures. `ggen` requires that static VALUES configurations be declared inline inside `ggen.toml` (e.g. `query = { inline = "SELECT ... WHERE { VALUES ... }" }`) rather than stored in external `.rq` files. External files are reserved for queries running against actual graph triples. If an external `.rq` query contains a `VALUES` block (outside comment boundaries), `E0010` is thrown.
2.  **`error[E0011]` (Inference Query Determinism):**
    Under `strict_mode = true`, if an inference rule's SPARQL CONSTRUCT query lacks an `ORDER BY` clause, `E0011` is triggered. Since triple ordering can influence downstream code generator templates, deterministic graph sorting is enforced.
3.  **`error[E0013]` (Generation Query Determinism):**
    Similarly, under `strict_mode = true`, if a generation rule's SELECT query lacks an `ORDER BY` clause, `E0013` is thrown. This guarantees deterministic template iteration and output generation.
4.  **`error[E0014]` (Pack Dependency Declaration):**
    If a generation rule uses a query or template pointing to a pack (e.g., `query = { pack = "x", ... }`), the pack `x` MUST be registered in the manifest's top-level `[[packs]]` array. If not, `E0014` is thrown.

---

## 4. The "BIG BANG 80/20" Criteria
As documented in the comments of the reference `ggen.toml` file, the **"BIG BANG 80/20"** rule enforces *Specification Closure First*. It serves as a checklist to evaluate whether a project is ready for ontology-driven code generation, or if the requirements are still poorly formed.

### Verbatim Criteria:
> **BIG BANG 80/20: Specification Closure First**
> Before using ggen, confirm:
> 1. Do you have real user data (CSV/JSON)? Not promised—actual files.
> 2. Can you find one existing standard ontology (schema.org, FOAF, Dublin Core, SKOS)?
>    Should take 5 minutes. If it takes 3 months, you're building custom (wrong path).
> 3. Can you explain your problem in one sentence? No 100-page documents.
> 4. Has anyone (not a friend, not a co-founder) committed to this?
>    Email, contract, payment—proof, not enthusiasm.
> 5. Can you validate with 10 real users in 48 hours?
>
> If you answered NO to any of these, stop. Talk to Sean before proceeding.

### Meaning and Rationale:
*   **Real Data Gate:** Avoids speculative modeling by demanding actual files immediately.
*   **Ontology Leverage:** Prevents the anti-pattern of constructing domain taxonomies from scratch. Projects should adopt well-understood industry standards (e.g., FOAF for people, schema.org for items) to maintain immediate semantic compatibility.
*   **Simplicity Constraint:** Forces clarity of scope. If the system architecture requires massive text documentation to describe the base problem, the ontology boundaries will become bloated.
*   **External Validation (Commitment & Users):** Ensures that the generated product is bound to real-world utility and verified loops.
*   **The Escalate Mandate:** If any gate fails, developers are blocked from commencing and directed to coordinate directly with the project owner (Sean).
