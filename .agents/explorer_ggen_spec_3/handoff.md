# Handoff Report — Explorer Ggen Schema 3

This handoff report summarizes the research and findings regarding the `ggen.toml` schema specification and the "BIG BANG 80/20" criteria.

---

## 1. Observation

The following files were inspected in the `/Users/sac/ggen/` workspace:

### A. Reference `ggen.toml` at `/Users/sac/ggen/ggen.toml`:
* Lines 8-18 detail the BIG BANG 80/20 criteria:
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
* Line 24-26 defines:
  ```toml
  source = "schema/domain.ttl"
  # Use standard ontologies only (BIG BANG 80/20 gate)
  standard_only = true
  ```

### B. Core Manifest Struct Definitions at `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs`:
* Line 97 defines `pub struct GgenManifest` representing the parsed ggen.toml layout:
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
      // ...
  }
  ```
* Line 151 defines `ProjectConfig` with `name`, `version`, `description`, `authors`, `license`, `repository`.
* Line 178 defines `OntologyConfig` with `source`, `imports`, `base_iri`, `prefixes`, `standard_only`.
* Line 200 defines `InferenceConfig` and `InferenceRule` with `construct` (String query), `order` (i32), `when` (Option<String> query).
* Line 238 defines `GenerationConfig` with `rules`, `max_sparql_timeout_ms`, `require_audit_trail`, `determinism_salt`, `output_dir`, `enable_llm`.
* Line 274 defines `GenerationRule` with `name`, `query` (`QuerySource`), `template` (`TemplateSource`), `output_file`, `skip_empty`, `mode` (`GenerationMode`), `when`.
* Line 303 defines `QuerySource` (Pack, File, or Inline), and Line 333 defines `TemplateSource` (Pack, File, Inline, Git, or Package).
* Line 397 defines `ValidationConfig` and `ValidationRule` with `shacl` (Vec<PathBuf>), `validate_syntax`, `no_unsafe`, `strict_mode`, `rules`.

### C. Validation Rules at `/Users/sac/ggen/crates/ggen-core/src/manifest/validation.rs`:
* Lines 94-100 check for `ORDER BY` in `CONSTRUCT` queries for inference rules under `strict_mode`:
  ```rust
  if !construct_upper.contains("ORDER BY") {
      if self.manifest.validation.strict_mode {
          return Err(Error::new(&format!(
              "error[E0011]: Inference rule '{}' CONSTRUCT query lacks ORDER BY...",
              rule.name
          )));
      }
  }
  ```
* Lines 145-151 check that `VALUES` statements remain inline in `ggen.toml`:
  ```rust
  if query_contains_values(&content) {
      return Err(Error::new(&format!(
          "error[E0010]: VALUES data must be inline in ggen.toml\n  --> rule: '{}'...",
          rule.name
      )));
  }
  ```
* Lines 175-181 validate E0014:
  ```rust
  if !self.manifest.packs.iter().any(|p| p.name == pack_name) {
      return Err(Error::new(&format!(
          "error[E0014]: Pack '{}' used in rule '{}' is not declared in [[packs]]",
          pack_name, rule.name
      )));
  }
  ```
* Lines 196-202 check E0013:
  ```rust
  if !query_has_order_by(query_text) {
      if self.manifest.validation.strict_mode {
          return Err(Error::new(&format!(
              "error[E0013]: Generation rule '{}' SELECT query lacks ORDER BY...",
              rule.name
          )));
      }
  }
  ```

### D. Philosophy / Concept Documentation at `/Users/sac/ggen/README.md`:
* Lines 431-439 describe the "Big Bang 80/20: Specification Closure First":
  ```markdown
  ### 1. Big Bang 80/20: Specification Closure First

  **What it means**: Verify that your RDF specification is 100% complete *before* generating any code. No iteration on generated artifacts -- fix the specification and regenerate.
  ```

---

## 2. Logic Chain

1. From **Observation A** and **Observation D**, the **BIG BANG 80/20** paradigm represents a strict policy requiring a 100% complete RDF specification and standard ontology validation (`standard_only = true`) before code generation runs.
2. From **Observation B**, the schema structure of `ggen.toml` parsed by the code-generator engine is modeled inside `crates/ggen-core/src/manifest/types.rs` through `GgenManifest` and its inner blocks (`[project]`, `[ontology]`, `[inference]`, `[[generation.rules]]`, `[validation]`).
3. From **Observation C**, several critical validation errors are defined to enforce this schema structure and determinism:
   - **E0010**: Rejects external `.rq` files containing `VALUES` clauses (which must reside inline in `ggen.toml`).
   - **E0011**: Rejects `CONSTRUCT` queries lacking `ORDER BY` when `strict_mode = true`.
   - **E0013**: Rejects `SELECT` queries lacking `ORDER BY` when `strict_mode = true`.
   - **E0014**: Rejects rules referencing packs not declared in `[[packs]]`.

---

## 3. Caveats

- We assumed that `crates/ggen-core/src/manifest/` is the active parsing logic used during code generation (`ggen sync`). While `crates/ggen-config/src/config_lib/schema.rs` also defines a `GgenConfig` structure, `crates/ggen-core/src/lib.rs` exports both, and the compiler/linter error codes (E0010, E0011, E0013, E0014) match the schema constraints of `GgenManifest`.
- We did not investigate the third-party ontology registries that standard_only interacts with to fetch standard schema components since it is out of scope for a read-only investigation.

---

## 4. Conclusion

The schema structure of `ggen.toml` has been successfully identified, including the exact fields and validation gates (E0010, E0011, E0013, E0014) that govern its configuration blocks (`[project]`, `[ontology]`, `[inference]`, and `[[generation.rules]]`). The five BIG BANG 80/20 criteria are documented, demonstrating that ggen enforces a "Specification-First" design policy.

---

## 5. Verification Method

To independently verify the configuration structures and parsing logic:
1. Open and view `/Users/sac/ggen/crates/ggen-core/src/manifest/types.rs` to review struct fields.
2. Open and view `/Users/sac/ggen/crates/ggen-core/src/manifest/validation.rs` to review validation checks.
3. Run the unit tests within the `ggen-core` crate using the following command inside `/Users/sac/ggen/`:
   ```bash
   cargo test -p ggen-core --manifest-path /Users/sac/ggen/Cargo.toml manifest::tests
   ```
   This will run `test_parse_minimal_manifest` and `test_parse_full_manifest`, confirming that the TOML matches the described parser schemas.
