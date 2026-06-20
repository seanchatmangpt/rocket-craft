# Quality and Adversarial Review of Ggen Pack Specification

This document contains the independent quality review and adversarial challenge for the canonical `GGEN_PACK_SPEC.md` specification.

---

# Part 1: Quality Review Report

## Review Summary

**Verdict**: REQUEST_CHANGES

The specification is exceptionally complete and detailed, covering all required schema blocks, the core pipeline architecture (Inference vs. Generation), the four critical validation error codes, and the 5 "BIG BANG 80/20" criteria. However, there is a major correctness issue in the quick-start boilerplate example where the `output_file` target violates the project's own LSP (static analysis) rules, causing it to fail validation upon first use.

---

## Findings

### [Major] Finding 1: Boilerplate `output_file` violates `GGEN-YIELD-001` (Layer Boundary Violation)

- **What**: The boilerplate `ggen.toml` specifies `output_file = "output_structs.txt"`.
- **Where**: `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` (Line 226)
- **Why**: Under the `ggen-lsp` static analysis rules (defined in `unify-rs/anti-llm-cheat-lsp/src/rules/ggen.rs` and `unify-rs/anti-llm-cheat-lsp/src/parsers/ggen_toml.rs`), `output_file` targets that do not contain a directory separator `/` trigger a layer boundary violation (`GGEN-YIELD-001` / `ggen_layer_violation`), since they target the pack root instead of a consumer path. This is a blocking diagnostic that aborts execution.
- **Suggestion**: Change `output_file = "output_structs.txt"` to a path containing a directory prefix, e.g., `output_file = "src/output_structs.txt"`.

### [Major] Finding 2: Missing Documentation of `output_file` Directory and Path Restrictions

- **What**: The specification text in Section 2.5 (`[[generation.rules]]` block description) does not document the restrictions on `output_file` paths.
- **Where**: `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` (Section 2.5)
- **Why**: Standard users copying/writing manifests need to know that:
  1. `output_file` must contain a directory component (to prevent layer violations).
  2. `output_file` must not contain `/generated/`, `/output/`, or `/gen/` segments (to prevent second-class source warnings).
  If these constraints are not documented in the specification, users will experience unexpected build blocks.
- **Suggestion**: Document these two path-based constraints clearly under Section 2.5.

---

## Verified Claims

- **Coverage of all schema blocks** &rarr; verified via visual inspection of `/Users/sac/.ggen/specs/GGEN_PACK_SPEC.md` sections 2.1 through 2.7 &rarr; **PASS**
- **Explanation of Inference vs. Generation** &rarr; verified via visual inspection of Section 3 (diagram and text matching pipeline behavior) &rarr; **PASS**
- **Detailing of error codes (E0010, E0011, E0013, E0014)** &rarr; verified via inspection of Section 4 (explanations, rationale, and exact error blocks present) &rarr; **PASS**
- **Listing of 5 "BIG BANG 80/20" criteria** &rarr; verified via inspection of Section 5 &rarr; **PASS**
- **Syntactic validity of RDF/Turtle boilerplate** &rarr; verified syntax of `schema/domain.ttl` &rarr; **PASS**
- **Syntactic validity of SPARQL SELECT/CONSTRUCT queries** &rarr; verified syntax against SPARQL 1.1 query specification &rarr; **PASS**
- **Execution of LSP code tests** &rarr; verified by running `cargo test` in `unify-rs/anti-llm-cheat-lsp` &rarr; **PASS** (all 22 tests pass)

---

## Coverage Gaps

- **Integration with actual ggen execution runtime** — Risk Level: **Medium** — Recommendation: Accept risk. The specification defines the schema block settings correctly, and runtime testing of the `ggen` binary itself is handled by the compilation/integration pipelines of those repositories.

---

## Unverified Items

- None. All major claims in the spec were cross-referenced with the codebase.

---

# Part 2: Adversarial Review Report

## Challenge Summary

**Overall risk assessment**: MEDIUM

The specification's conceptual integrity is sound, but we identified edge-case gaps where the simplified LSP analyzer rules deviate from the specification or could be circumvented.

---

## Challenges

### [Medium] Challenge 1: LSP Cheat Parser Variable Mismatch Bypass

- **Assumption challenged**: The LSP analyzer checks that all variables consumed by a template are projected by the query (`GGEN-TPL-001`).
- **Attack scenario**: The cheat LSP analyzer (`unify-rs/anti-llm-cheat-lsp/src/parsers/tera_template.rs`) resolves sibling query files by replacing the `.tera` extension with `.rq` in the same directory. If a developer declares queries and templates in different directories (e.g. `queries/` and `templates/` as in the boilerplate), the cheat-LSP fails to locate the query file and skips variable validation entirely.
- **Blast radius**: Template variable mismatches will bypass pre-flight LSP checks and only fail during real `ggen sync` runs.
- **Mitigation**: Update the cheat LSP parser to read the `ggen.toml` manifest to map rules to their queries, rather than relying on directory naming heuristics.

### [Low] Challenge 2: Non-standard TOML Array of Tables representation

- **Assumption challenged**: The spec states in 2.5 that `[[generation.rules]]` is an array of tables.
- **Attack scenario**: The boilerplate `ggen.toml` defines:
  ```toml
  [generation]
  rules = [
      { name = "generate-structs", ... }
  ]
  ```
  While parsed identically by `serde` in Rust, this array-of-inline-tables notation can become extremely long and unreadable in standard TOML files.
- **Blast radius**: Low. Standard TOML parsing will still succeed, but it degrades developer ergonomics.
- **Mitigation**: Write the boilerplate rule using the standard `[[generation.rules]]` array of tables header.

---

## Stress Test Results

- **Empty SPARQL Projection** &rarr; A SELECT query with no projected variables (`SELECT *`) &rarr; Cheat LSP skips validation to prevent false positives &rarr; **PASS**
- **Whitespace / Carriage Return in TTL** &rarr; Turtle parser parsing Windows-style CRLF line endings &rarr; Standard Turtle parsers ignore whitespace, so it is robust &rarr; **PASS**
- **Missing Sibling Query File** &rarr; Template without a matching `.rq` file &rarr; LSP safely falls back without crashing &rarr; **PASS**

---

## Unchallenged Areas

- **SHACL engine performance under strict mode** — Reason: Out of scope for this spec review.
