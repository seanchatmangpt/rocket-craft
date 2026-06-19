# Handoff Report: Ggen Pack Specification Schema Discovery

## 1. Observation
I directly investigated the schema definitions, parsing code, and manifest validation implementation within the `/Users/sac/ggen/` repository.

### Exact File Paths and Contents:
1.  **`ggen.toml` Comments (BIG BANG 80/20 criteria):**
    *   **Path:** `/Users/sac/ggen/ggen.toml` (lines 8-18)
    *   **Verbatim Quote:**
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

2.  **`GgenManifest` Root Structure:**
    *   **Path:** `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs` (lines 94-146)
    *   **Verbatim Code:**
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
            // ... ignored configuration fields
        }
        ```

3.  **Validation Error Guards:**
    *   **Path:** `/Users/sac/ggen/crates/ggen-core/src/manifest/validation.rs`
    *   **Verbatim Error E0010 (VALUES Inline Enforcement):**
        ```rust
        if query_contains_values(&content) {
            return Err(Error::new(&format!(
                "error[E0010]: VALUES data must be inline in ggen.toml\n  --> rule: '{}'\n  --> file: {}\n  |\n  = VALUES clauses belong in ggen.toml as `query = {{ inline = \"SELECT ... WHERE {{ VALUES ... }}\" }}`\n  = External .rq files are for queries against real RDF triples only\n  = help: Move the VALUES block into ggen.toml and delete the .rq file",
                rule.name,
                query_path.display()
            )));
        }
        ```
    *   **Verbatim Error E0011 (Inference Query Determinism):**
        ```rust
        if !construct_upper.contains("ORDER BY") {
            if self.manifest.validation.strict_mode {
                return Err(Error::new(&format!(
                    "error[E0011]: Inference rule '{}' CONSTRUCT query lacks ORDER BY\n  |\n  = strict_mode is enabled: non-deterministic triple ordering is rejected\n  = help: Add ORDER BY to your CONSTRUCT query to guarantee deterministic output\n  = help: Or set `strict_mode = false` in [validation] to downgrade to a warning",
                    rule.name
                )));
            }
        }
        ```
    *   **Verbatim Error E0013 (Generation Query Determinism):**
        ```rust
        if !query_has_order_by(query_text) {
            if self.manifest.validation.strict_mode {
                return Err(Error::new(&format!(
                    "error[E0013]: Generation rule '{}' SELECT query lacks ORDER BY\n  |\n  = strict_mode is enabled: non-deterministic row ordering is rejected\n  = help: Add ORDER BY to your SELECT query to guarantee deterministic template rendering\n  = help: Or set `strict_mode = false` in [validation] to downgrade to a warning",
                    rule.name
                )));
            }
        }
        ```
    *   **Verbatim Error E0014 (Pack Dependency Declaration):**
        ```rust
        if let Some(pack_name) = rule_pack_name {
            if !self.manifest.packs.iter().any(|p| p.name == pack_name) {
                return Err(Error::new(&format!(
                    "error[E0014]: Pack '{}' used in rule '{}' is not declared in [[packs]]",
                    pack_name, rule.name
                )));
            }
        }
        ```

---

## 2. Logic Chain
1.  **Deserialization Model:** I analyzed `crates/ggen-core/src/manifest/types.rs` and mapped how TOML configuration sections bind to Rust structures via serde `Deserialize` attributes (e.g. `GgenManifest` mapping `project`, `ontology`, `inference`, `generation`, `validation`, and `packs` blocks).
2.  **Semantic Quality Gates:** I inspected `crates/ggen-core/src/manifest/validation.rs` to understand post-parse sanity checks.
3.  **Strict Mode Controls:** The validator enforces determinism warnings (missing `ORDER BY` in SPARQL statements) as hard compile errors (`E0011` and `E0013`) when `strict_mode` is enabled under the `[validation]` block.
4.  **Static Data Separation:** `E0010` protects the codebase by ensuring mock values or hardcoded data mapping lists are defined inline within the manifest, rather than polluting external `.rq` graph query files.
5.  **Dependency Guarantees:** `E0014` ensures that any generation rule referencing a template or query inside a ggen pack has that pack declared in the `[[packs]]` array of the manifest.
6.  **"BIG BANG 80/20" Project Health Gate:** In the reference `ggen.toml` file, a detailed comment section outlines five non-technical criteria to decide whether ontology-driven generation is ready for start or should be escalated to the project owner (Sean).

---

## 3. Caveats
- I did not execute the actual query parsing or template rendering loop since this was a read-only investigation.
- I assumed the codebase targets `strict_mode` defaults as documented in `validation.rs`, where lack of `ORDER BY` is a warning unless `strict_mode` is explicitly `true` in `[validation]`.

---

## 4. Conclusion
The `ggen.toml` manifest is strongly-typed and validated under the `ggen` framework. The deserialization mapping and semantic quality checks are fully implemented. We have a solid blueprint of the schema (detailed in `analysis.md`) and the error cases (`E0010`, `E0011`, `E0013`, `E0014`) to proceed with downstream pack specification authoring.

---

## 5. Verification Method
1.  **Inspecting Files:**
    - Examine my detailed findings report: `/Users/sac/rocket-craft/.agents/explorer_ggen_spec_1/analysis.md`
    - Verify structural definitions directly in the code: `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs`
2.  **Running Tests:**
    - Run the negative validation unit tests under `/Users/sac/ggen` using:
      ```bash
      cargo test --package ggen-core --test manifest_schema_validation_test
      ```
    - Check for successful test results confirming that the validator behaves as specified.
