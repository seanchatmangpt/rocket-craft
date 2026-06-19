# Quality & Adversarial Review Report: Ggen Pack Specification

This document presents a comprehensive quality and adversarial review of the Ggen Pack Specification (`/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md`).

---

# PART 1: QUALITY REVIEW

## Review Summary

**Verdict**: APPROVE

The canonical specification is highly complete, structurally sound, and accurately reflects the structure and constraints of the `ggen` ontology pack and configuration system. All requested configuration blocks, error codes, and the 5 "BIG BANG 80/20" criteria are properly documented. Verification of the quick-start boilerplate examples indicates that the configurations, schemas, queries, and templates are syntactically and logically correct.

---

## Findings

### [Major] Finding 1: Diagnostic Code Collision on `E0011`
- **What**: The error code `E0011` is dual-mapped in the codebase.
- **Where**: 
  - `crates/ggen-core/src/manifest/validation.rs:97` -> `error[E0011]: Inference rule '{}' CONSTRUCT query lacks ORDER BY` (Inference query determinism).
  - `crates/ggen-core/src/codegen/pipeline.rs:987` -> `error[E0011]: Output file already exists in 'Create' mode` (File creation safety).
- **Why**: This is a collision in the diagnostic system. The specification in `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` only documents `E0011` as "Inference Query Determinism" and does not account for the file existence collision.
- **Suggestion**: The codebase should refactor the output file existence error to a separate error code (e.g., `E0016` or `E0017`), and the specification should be updated to reflect this distinction.

### [Minor] Finding 2: Boilerplate TOML Structure Style
- **What**: The boilerplate `ggen.toml` uses inline table arrays for rules (e.g., `rules = [ { name = "...", ... } ]`) under `[inference]` and `[generation]`.
- **Where**: `ggen.toml` example in Section 6.1.
- **Why**: While parsed identically by `serde` in Rust, inline arrays are harder to read and format across multiple lines compared to standard TOML table arrays (`[[generation.rules]]` or `[[inference.rules]]`).
- **Suggestion**: Update the boilerplate `ggen.toml` to use standard table array blocks for improved readability.

### [Minor] Finding 3: Omission of Validation Code `E0012`
- **What**: Error code `E0012` (Unsafe block validation) is omitted from the specification.
- **Where**: Section 4: Validation Error Guards.
- **Why**: The codebase implements `E0012` in `crates/ggen-core/src/codegen/pipeline.rs:1473` to enforce the `no_unsafe = true` check. Documenting this would complete the code coverage.
- **Suggestion**: Add a section detailing `E0012` under the Validation Error Guards list.

---

## Verified Claims

- **Claim**: The specification includes all core schema blocks: `[project]`, `[ontology]`, `[inference]`, `[[generation.rules]]`, `[validation]`, and `[[packs]]`.
  - *Verified via*: Direct inspection of `GGEN_PACK_SPEC.md` (Sections 2.1 - 2.7) -> **PASS**
- **Claim**: The difference between inference and generation is explained correctly.
  - *Verified via*: Direct inspection of Section 3, noting that SPARQL CONSTRUCT is for graph enrichment/in-memory, and SELECT + Tera is for file generation -> **PASS**
- **Claim**: The validation error codes `E0010`, `E0011`, `E0013`, and `E0014` are detailed.
  - *Verified via*: Direct inspection of Section 4 -> **PASS**
- **Claim**: The 5 "BIG BANG 80/20" criteria are listed.
  - *Verified via*: Direct inspection of Section 5 -> **PASS**
- **Claim**: The boilerplate example syntax (TOML, Turtle, SPARQL, and Tera) is valid.
  - *Verified via*: Syntax tracing and cross-referencing with parser constraints -> **PASS**
- **Claim**: The ggen workspace compiles cleanly without errors.
  - *Verified via*: Executed `just check` in `/Users/sac/ggen`, which compiled all 15 crates successfully in 1m 55s -> **PASS**

---

## Coverage Gaps

- **Crate configuration fields (`[sync]`, `[rdf]`, etc.)** — risk level: Low — The optional daemon configuration blocks are mentioned in Section 2.8 but not fully specified. This is acceptable since they are ignored by the generator, but future revisions could detail them for the runtime configuration daemon.
- **Error code sequence gaps** — risk level: Low — Gaps in documented error codes (like `E0012`) exist. Recommendation: Accept risk for now and document them incrementally.

---

## Unverified Items

- **Actual parsing behavior of all ignored configuration fields** — reason not verified: These are passed directly to `toml::Value` and handled dynamically by the config crate, which was out of scope.

---
---

# PART 2: ADVERSARIAL REVIEW

## Challenge Summary

**Overall risk assessment**: LOW

The specification provides a deterministic and structured ontology packaging format that successfully prevents common code-generation pitfalls. The primary stress point is the integration of non-standard SPARQL CONSTRUCT query syntax and handling duplicate error definitions in the compiler.

---

## Challenges

### [Medium] Challenge 1: Non-standard `ORDER BY` in SPARQL `CONSTRUCT`
- **Assumption challenged**: The specification assumes that SPARQL CONSTRUCT queries support and enforce `ORDER BY` deterministically.
- **Attack scenario**: Standard SPARQL 1.1 engines do not guarantee triple ordering in CONSTRUCT results. An external tool parsing `domain.ttl` or executing the CONSTRUCT query might ignore the `ORDER BY` clause, causing out-of-order assertions when exported.
- **Blast radius**: Low. Internal `ggen` compilation is deterministic, but interoperability with standard external SPARQL triple-stores may exhibit slight ordering differences.
- **Mitigation**: Add a note in Section 3.1 clarifying that `ORDER BY` in `CONSTRUCT` is a `ggen`-specific extension implemented to enforce deterministic graph serialization and git diff stability.

### [Low] Challenge 2: Tera Unbound Variables and Empty Queries
- **Assumption challenged**: The SELECT query always yields rows matching the expected bindings in the Tera template.
- **Attack scenario**: If the SPARQL query returns 0 rows (e.g. if the ontology contains no `rdfs:Class` triples), the Tera loop over `results` executes 0 times. If `skip_empty` is `false`, an empty output file is generated.
- **Blast radius**: Low. Can result in blank files being committed.
- **Mitigation**: Ensure that templates gracefully handle empty collections or default values, and enforce `skip_empty = true` for generation rules targeting structures that cannot be empty.

---

## Stress Test Results

- **Empty Ontology Input** -> `domain.ttl` is empty -> The parser handles it successfully but the SELECT query returns empty results. If `skip_empty = false`, it creates an empty `output_structs.txt`. If `skip_empty = true`, it skips file generation -> **PASS**
- **Strict Mode Enforcement** -> Run without `ORDER BY` in SELECT/CONSTRUCT -> Triggers E0013/E0011 compile error, successfully preventing non-deterministic builds -> **PASS**
- **Undeclared Pack Dependency** -> Reference pack not in `[[packs]]` -> Triggers E0014 error, preventing execution -> **PASS**

---

## Unchallenged Areas

- **AI/LLM-based generation flags** — reason not challenged: The AI providers/models are specified at a configuration level and handled by external client drivers, meaning code-level stress testing requires active network/API keys.
