# Forensic Audit Report

**Work Product**: C++ Backbone Ontology (`core.ttl`), `ggen.toml`, and Validation Harness (`validate_ontology.sh`)
**Profile**: General Project (Development Mode / Demo Mode)
**Verdict**: INTEGRITY VIOLATION

---

## 1. Summary of Findings
Through systematic forensic analysis, compiler instrumentation, and sandbox mutation testing, we verified the following facts:
1. **Validation Engine Facade (Integrity Violation)**: The custom validation rules (SPARQL ASK rules `R1`-`R4`) and SHACL shape configuration (`validation.shacl.ttl`) declared in `ggen.toml` are **never executed** by the installed compiler binary (`ggen 26.6.9`) during pre-flight checks. The validation harness `validate_ontology.sh` reports success (`All validations passed`) even when the ontology is mutated to violate both SHACL constraints and class hierarchy constraints.
2. **Broken Validation Rules**: If the compiler is updated to a version that actually executes validation rules (e.g., `ggen 26.6.11` built from local source), the validation rules `R1`-`R4` fail immediately. They are syntactically invalid because they begin with `PREFIX` declarations, which trips the compiler's prefix check (expecting queries to start directly with `ASK` or `SELECT`).
3. **Broken Inference Rules**: Under the updated compiler, the inference rules `infer-is-component-of` and `infer-is-level-of` fail immediately under `strict_mode = true` because they match 0 triples (no instance data exists in the ontology). 
4. **SHACL Validator Bypass**: SHACL shape validation is completely bypassed/ignored in the compiler's manifest-driven sync pipeline. The `shacl` array in `ggen.toml` is only checked for path existence but is never evaluated against the ontology.
5. **Class Hierarchy Definitions**: The class hierarchies and namespace definitions in `core.ttl` are genuinely and fully defined, but due to the compiler and validation rule defects, they are not being verified by the validation harness.

---

## 2. Phase Results

### Phase 1: Source Code Analysis
- **Hardcoded output detection**: PASS. No hardcoded test outputs or bypass strings were found in `core.ttl` or `ggen.toml`.
- **Facade detection**: FAIL.
  - The custom validation rules `R1`-`R4` in `ggen.toml` represent a facade: they appear to validate class hierarchies, subsystems, blueprints, and WASM typestates, but are un-wired/ignored in `ggen 26.6.9`.
  - The SHACL shape configuration `validation.shacl.ttl` is a facade: it is never evaluated against the ontology by the `ggen sync` pipeline.
- **Pre-populated artifact detection**: PASS. No pre-populated logs or validation receipts predated the run.

### Phase 2: Behavioral Verification
- **Build and run validation harness**: PASS. The script `/Users/sac/rocket-craft/validate_ontology.sh` executes successfully.
- **Sandbox Mutation Testing (Verification)**: FAIL.
  - **Mutation 1 (SPARQL ASK Violation)**: Changed `ue4:ACharacter rdfs:subClassOf ue4:APawn` to `ue4:ACharacter rdfs:subClassOf ue4:UObject` in a sandboxed `core.ttl`. The installed binary `ggen 26.6.9` reported success (`All validations passed`).
  - **Mutation 2 (SHACL Shape Violation)**: Removed `rdfs:label "ACharacter"` from `core.ttl`, violating the SHACL minimum count constraint. `ggen 26.6.9` reported success.
- **Local Compiler Verification**:
  - Compiled `ggen 26.6.11` from `/Users/sac/ggen` and executed it on the original ontology. It failed immediately with:
    `error[GGEN-INFER-001]: Inference rule 'infer-is-component-of' added 0 triples`
  - Disabled `strict_mode` to test the SPARQL ASK rules. It failed immediately with:
    `Failed to execute rule R1: Query must start with ASK or SELECT` (caused by leading `PREFIX` in `ggen.toml` rules).
  - Fixed the rules to use inline full IRIs and ran the validator. It successfully failed on the class hierarchy violation (Mutation 1), but completely ignored the SHACL violation (Mutation 2), confirming that the SHACL pipeline is bypassed.

---

## 3. Evidence

### A. Installed Compiler Bypass (facade behavior)
Command:
```bash
/Users/sac/.local/bin/ggen sync --validate-only true
```
Output:
```
Manifest schema:     PASS ()
Dependencies:     PASS (6/6 checks passed)
Ontology syntax:     PASS (core.ttl)
SPARQL queries:     PASS (1 queries validated)
Templates:     PASS (1 templates validated)

All validations passed.
```
*(Notice that "Custom validation rules" and SHACL validation checks are completely missing from the PASS/FAIL log, even though `ggen.toml` specifies them.)*

### B. Upgraded Compiler Failure on Unmodified Ontology
Command:
```bash
/Users/sac/ggen/target/debug/ggen sync --validate-only true
```
Output:
```
Custom validation rules:     FAIL (error[GGEN-INFER-001]: Inference rule 'infer-is-component-of' added 0 triples
  = strict_mode is enabled: identity/no-match CONSTRUCT rules are rejected
  = help: Verify the query pattern against the loaded ontology
  = help: If intentionally conditional, add a `when` clause)
```

### C. Upgraded Compiler Prefix Error
*(After setting `strict_mode = false` in `ggen.toml` to bypass GGEN-INFER-001)*
Command:
```bash
/Users/sac/ggen/target/debug/ggen sync --validate-only true
```
Output:
```
Custom validation rules:     FAIL (error[GGEN-VALIDATION]: 1 custom validation rule(s) failed (Error severity):
  - query-execution-error: Failed to execute rule R1: SPARQL query execution failed: Invalid query for rule R1: Query must start with ASK or SELECT
  = generation aborted before writing files)
```

### D. Upgraded Compiler Successful Capture of Mutation
*(After rewriting SPARQL rules using inline full IRIs instead of prefixes)*
Command:
```bash
/Users/sac/ggen/target/debug/ggen sync --validate-only true
```
Output:
```
Custom validation rules:     FAIL (error[GGEN-VALIDATION]: 1 custom validation rule(s) failed (Error severity):
  - R1: Verify class hierarchy (UObject, AActor, APawn, ACharacter, UActorComponent, UWorld, ULevel existence and subClassOf connections)
  = generation aborted before writing files)
```

---

## 4. Logic Chain
1. If the validation rules in `ggen.toml` were genuinely executed, mutating the ontology to violate class hierarchy relationships (`ACharacter` subclass of `UObject`) would cause the validation to fail.
2. When mutating the ontology and running `/Users/sac/.local/bin/ggen sync --validate-only true`, the validation passed without error.
3. This proves that the installed compiler version (`26.6.9`) does not execute custom SPARQL validation rules.
4. Inspecting the compiler codebase at `/Users/sac/ggen/crates/ggen-core/tests/validation_ask_enforcement_test.rs` confirms this is a known regression ("membership != identity" trap) where the validation configs were parsed but never invoked in the pipeline.
5. In addition, the rules declared in `ggen.toml` are structurally broken because they begin with `PREFIX`, which is rejected by the compiler's query executor.
6. Mutating the ontology to violate SHACL shapes while satisfying class hierarchy rules results in a successful validation run on the fixed compiler, proving that SHACL shapes are completely ignored during manifest synchronization.
7. Therefore, the validation configuration and the harness represent a facade implementation of quality gates.
