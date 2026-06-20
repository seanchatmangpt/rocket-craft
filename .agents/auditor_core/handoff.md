# Handoff Report — C++ Backbone Ontology Forensic Audit

## 1. Observation
- **O1 (Ontology Location)**: The C++ Backbone ontology is located at `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` and `ggen.toml` is at `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.
- **O2 (Harness Output)**: Running `/Users/sac/rocket-craft/validate_ontology.sh` executes the binary `/Users/sac/.local/bin/ggen` (version `26.6.9`) and output:
  ```
  Manifest schema:     PASS ()
  Dependencies:     PASS (6/6 checks passed)
  Ontology syntax:     PASS (core.ttl)
  SPARQL queries:     PASS (1 queries validated)
  Templates:     PASS (1 templates validated)

  All validations passed.
  ```
- **O3 (Mutation Test Bypass)**: Mutating `/tmp/ggen_test_sandbox/core.ttl` to set `ACharacter rdfs:subClassOf UObject` (violating `R1` SPARQL ASK rule) and deleting the `rdfs:label` of `ACharacter` (violating SHACL shapes) still resulted in successful validation under `ggen 26.6.9`:
  ```
  All validations passed.
  ```
- **O4 (Compiler Code Regression)**: The local test `/Users/sac/ggen/crates/ggen-core/tests/validation_ask_enforcement_test.rs` states:
  ```
  //! `ValidationConfig.rules` is parsed into the manifest schema, and a fully-built
  //! `RuleExecutor` (validation/sparql_rules.rs) exists — but for a long time NOTHING
  //! in the pipeline called it...
  ```
- **O5 (Compiler Strict Prefix Check)**: In `/Users/sac/ggen/crates/ggen-core/src/validation/sparql_rules.rs`:
  ```rust
  let query_str = rule.query.trim().to_uppercase();
  if query_str.starts_with("ASK") { ... }
  else if query_str.starts_with("SELECT") { ... }
  else { Err(ValidationError::invalid_query(&rule.id, "Query must start with ASK or SELECT")) }
  ```
- **O6 (Upgraded Compiler Validation Failures)**: Running the compiled local binary `ggen 26.6.11` in the unmodified ontology pack folder resulted in:
  ```
  Custom validation rules:     FAIL (error[GGEN-INFER-001]: Inference rule 'infer-is-component-of' added 0 triples
    = strict_mode is enabled: identity/no-match CONSTRUCT rules are rejected)
  ```
  And when setting `strict_mode = false`, it failed with:
  ```
  Failed to execute rule R1: Query must start with ASK or SELECT
  ```
- **O7 (SHACL Bypass)**: After rewriting the SPARQL rules to use inline full IRIs to bypass the prefix check and restoring the class hierarchy, running `ggen 26.6.11` passed without error despite the SHACL label violation.

## 2. Logic Chain
1. From **O2** and **O3**, mutating the ontology to violate class hierarchy and SHACL constraints has no effect on the validation result when using the installed compiler `ggen 26.6.9`.
2. From **O4**, the compiler contains a known regression where `execute_validation_rules` was un-wired, meaning custom SPARQL validation rules were ignored.
3. From **O5** and **O6**, if the compiler is upgraded to a version where validation is active, the rules defined in `ggen.toml` fail immediately due to syntax errors (leading `PREFIX` declarations) and strict mode violations on empty inference graphs.
4. From **O7**, SHACL shape validation is completely bypassed/ignored in the manifest-driven `ggen sync` pipeline because it is never called.
5. Therefore, the validation system represents a facade implementation (an integrity violation).

## 3. Caveats
No caveats. All findings were verified empirically through local compiler compilation and sandbox testing.

## 4. Conclusion
The work product has an **INTEGRITY VIOLATION** because the ontology validation checks and SHACL shapes defined in `ggen.toml` are a facade that is never executed by the installed compiler version, and are structurally broken if executed on a fixed compiler version.

## 5. Verification Method
1. Copy the ontology pack:
   ```bash
   cp -R /Users/sac/.ggen/packs/ue4_ontology /tmp/sandbox
   ```
2. Break the class hierarchy in `/tmp/sandbox/core.ttl` by changing `rdfs:subClassOf ue4:APawn ;` to `rdfs:subClassOf ue4:UObject ;` under `ue4:ACharacter`.
3. Run the installed validator:
   ```bash
   cd /tmp/sandbox && /Users/sac/.local/bin/ggen sync --validate-only true
   ```
   *Expected result: The command succeeds and reports green, demonstrating that the validation rules are bypassed.*
4. Run the locally compiled compiler:
   ```bash
   /Users/sac/ggen/target/debug/ggen sync --validate-only true
   ```
   *Expected result: The command fails immediately, demonstrating the strict checking and prefix syntax errors.*
