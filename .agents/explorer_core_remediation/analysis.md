# Analysis and Remediation Strategy: Milestone 2 Integrity Violation Fixes

## 1. Executive Summary
This report analyzes the Forensic Auditor's verdict of `INTEGRITY VIOLATION` on Milestone 2 and provides a complete remediation plan. The audit revealed that:
1. Custom SPARQL validation rules and SHACL validation were completely bypassed/ignored in the sync pipeline.
2. The validation rules had syntax errors (leading `PREFIX` declarations) rejected by the compiler's query executor routing parser.
3. The inference rules fail under `strict_mode = true` due to matching 0 triples when no instance data exists.
4. SHACL validation was completely un-wired from both the pre-flight check and code generation pipelines.

To resolve these quality defects, we have analyzed the `ggen` compiler codebase and formulated a concrete fix strategy.

---

## 2. Root Cause Analysis in Codebase

### A. Custom Validation Rules (SPARQL ASK/SELECT) Bypass and Prefix Error
- **The Bypass**: In `ggen 26.6.9`, `execute_validation_rules` was never invoked in the main sync pipeline. In the newer local compiler codebase (`26.6.11`), the wiring exists inside `crates/ggen-core/src/codegen/pipeline.rs::run()`, but is completely missing from the `--validate-only` command path in `crates/ggen-core/src/codegen/executor.rs`.
- **The Prefix Syntax Error**: The rule executor in `crates/ggen-core/src/validation/sparql_rules.rs` uses a simple check to determine if a query is `ASK` or `SELECT`:
  ```rust
  let query_str = rule.query.trim().to_uppercase();
  if query_str.starts_with("ASK") { ... }
  ```
  If a SPARQL query starts with `PREFIX` or `BASE`, this check fails and returns `Query must start with ASK or SELECT`, despite the query being syntactically valid in SPARQL.

### B. Inference Rules Failure under `strict_mode = true` (GGEN-INFER-001)
- In `crates/ggen-core/src/codegen/pipeline.rs::execute_inference_rule()`, if a rule adds 0 triples, it is rejected under `strict_mode = true` with error `GGEN-INFER-001`.
- Because the ontology pack `/Users/sac/.ggen/packs/ue4_ontology` contains only class and property declarations (schema-level ontology) without instance data, the patterns `?actor ue4:hasComponent ?component` and `?world ue4:hasLevel ?level` match 0 triples, triggering the failure.

### C. SHACL Validation Bypass
- In the manifest validator (`crates/ggen-core/src/manifest/validation.rs`), the `shacl` array in `ggen.toml` is only checked for the existence of files on disk.
- There is no code in `crates/ggen-core/src/codegen/pipeline.rs` or `crates/ggen-core/src/codegen/executor.rs` that loads the SHACL files and invokes `SparqlValidator::validate()` against the loaded ontology. It is completely un-wired.

---

## 3. Detailed Remediation Strategy

### Step 1: Fix the SPARQL Query Executor Check (Prefix Handling)
We modify `crates/ggen-core/src/validation/sparql_rules.rs` to dynamically detect the query type by executing it directly via `output.query(&rule.query)` and matching on the returned `QueryResults` variant, instead of doing string prefix parsing.
- If it returns `QueryResults::Boolean`, it is processed as an `ASK` rule.
- If it returns `QueryResults::Solutions`, it is processed as a `SELECT` rule.
- If it returns `QueryResults::Graph`, a compilation error is returned (unsupported query type).

This completely resolves the prefix constraint and allows standard SPARQL rules with `PREFIX` declarations to execute natively.

*See patch: `sparql_rules.rs.patch`*

### Step 2: Wire SHACL Shape Validation into the Sync Pipeline
We implement a new function `pub fn execute_shacl_validation(&mut self) -> Result<()>` in `crates/ggen-core/src/codegen/pipeline.rs` that:
1. Loads all SHACL shape files listed under `manifest.validation.shacl` into a `shapes_graph`.
2. Validates the `ontology_graph` (post-inference) against `shapes_graph` using `SparqlValidator`.
3. Records any warnings and returns a hard error on any `Severity::Violation` errors to abort the generation process before writing files.

We add calls to this validation in:
- `GenerationPipeline::run` (in `pipeline.rs`) to block code rendering on SHACL failures.
- `execute_validate_only` (in `executor.rs`) to report SHACL validation results alongside other quality gates.

*See patches: `pipeline.rs.patch` and `executor.rs.patch`*

### Step 3: Prevent Strict-Mode Failures on Inference Rules (Conditional Execution)
To allow the ontology schemas to load and validate without triggering `GGEN-INFER-001` when no instances exist, we use the compiler's conditional execution feature (`when` clause).
We add `when` clauses to the inference rules in `ggen.toml` to check for the presence of the relevant triples before executing the construct query:
- `infer-is-component-of` runs only `when` there are `hasComponent` triples.
- `infer-is-level-of` runs only `when` there are `hasLevel` triples.

*See proposed manifest: `proposed_ggen.toml`*

---

## 4. Verification Methodology

After the patches and configuration changes are applied:
1. **Compilation Check**: Run `cargo build` in `/Users/sac/ggen/` to ensure the updated compiler compiles cleanly.
2. **Unit Tests**: Run `cargo test --package ggen-core` to verify that existing test suites pass.
3. **Execution Verification**:
   - Install the new binary: `cargo install --path crates/ggen-cli --locked --root /Users/sac/.local/`
   - Run the validation harness: `/Users/sac/rocket-craft/validate_ontology.sh`.
   - It should output `PASS` for all checks (including SHACL validation and Custom validation rules).
4. **Sandbox Mutation Testing (Defect Capture)**:
   - **Mutation 1 (SPARQL ASK Violation)**: Change `ue4:ACharacter rdfs:subClassOf ue4:APawn` to `ue4:ACharacter rdfs:subClassOf ue4:UObject` in `core.ttl`. Run `validate_ontology.sh`. The harness must fail.
   - **Mutation 2 (SHACL Shape Violation)**: Remove `rdfs:label "ACharacter"` from `core.ttl`. Run `validate_ontology.sh`. The harness must fail due to SHACL minimum count constraint violation.
