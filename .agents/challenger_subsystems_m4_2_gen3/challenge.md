# Challenge Report: Subsystem Topologies (m4_2_gen3)

## Challenge Summary

**Overall risk assessment**: HIGH

While the core rules successfully target the intended constraints, we identified multiple severe gaps in the test harness and the tool's validation lifecycle integration.

---

## Challenges

### [Critical] Challenge 1: Non-Zero Exit Code Masking in `ggen sync`
- **Assumption challenged**: `ggen sync --validate-only true` is assumed to propagate non-zero exit codes to the shell upon validation failure.
- **Attack scenario**: A CI/CD build script executes validation, expects failure when rules are violated, but the process exits with `0` despite reporting `status: "error"` and printing `Custom validation rules: FAIL`.
- **Blast radius**: Invalid configurations (such as security policy violations or invalid parameters) will bypass standard gatekeepers (which rely on exit status), leading to corrupted, unbuildable, or non-compliant artifacts progressing to packaging.
- **Mitigation**: Update `ggen` execution logic to return exit status `1` (or a specific validation error code) whenever quality gates fail.

### [High] Challenge 2: Fragile Test Cleanup State Pollutes Baseline
- **Assumption challenged**: The testing scripts (`verify_all_rules.sh`) assume a clean slate before each run, but lack functional exit-state handlers.
- **Attack scenario**: A test case fails or the script is terminated early. Since the trap (`trap cleanup EXIT`) was commented out, the modified `core.ttl` is left on disk. The next run backs up this contaminated file as the "clean baseline" (`/tmp/core.ttl.bak`), propagating false positives to later steps (e.g., `RuleF` or `RuleRPCReturnTypeVoid` failing in unrelated tests).
- **Blast radius**: Developer test runs become non-deterministic, reporting incorrect errors and forcing manual git/file recoveries.
- **Mitigation**: Uncomment `trap cleanup EXIT` in `verify_all_rules.sh` and ensure that `restore` is executed robustly on exit.

### [Medium] Challenge 3: Unbounded Scope in VaRest Static Baking Prohibition
- **Assumption challenged**: The rule `RuleStaticBakingNoVaRest` (and corresponding SHACL shape) is assumed to only check graphs related to the statically baked world.
- **Attack scenario**: The SPARQL query does not trace the relationship from the target `?world` down to the graph `?node`. If a totally unrelated graph in the workspace contains a VaRest function call, the validation fails for the static target.
- **Blast radius**: Large multi-world ontologies cannot co-exist in the same repository if any of them use dynamic REST services (e.g. VaRest), causing validation blockages for valid worlds.
- **Mitigation**: Restrict the `?node` search in the query to graphs that are linked to the world:
  ```sparql
  ?world ue4:hasLevel/ue4:hasActor/ue4:hasGraph/ue4:hasNode ?node .
  ```

---

## Stress Test Results

### 1. Exit Status Verification
- **Scenario**: Append invalid pin connection (RuleA violation) and execute `ggen sync`.
- **Expected behavior**: Returns non-zero exit code.
- **Actual behavior**: Exited with code `0`.
- **Result**: **FAIL**

### 2. Validation Rejection: Material Parameter Mismatch (Rule 23)
- **Scenario**: Material instance defines a `scalarValue` parameter but base defines it as `UVectorParameter`.
- **Expected behavior**: Rejection with `RuleMaterialInstanceParameterValueType`.
- **Actual behavior**: Successfully rejected with `RuleMaterialInstanceParameterValueType`.
- **Result**: **PASS**

### 3. Validation Rejection: Unregistered Collision Profile (Rule 24)
- **Scenario**: Primitive component references a collision profile that the world's physics subsystem does not register.
- **Expected behavior**: Rejection with `RuleComponentCollisionProfileRegistration`.
- **Actual behavior**: Successfully rejected with `RuleComponentCollisionProfileRegistration`.
- **Result**: **PASS**

### 4. Validation Rejection: Server RPC missing validation (Rule 25)
- **Scenario**: UServerRPC has no validation function.
- **Expected behavior**: Rejection with `RuleServerRPCValidationMandatory`.
- **Actual behavior**: Successfully rejected with `RuleServerRPCValidationMandatory`.
- **Result**: **PASS**

### 5. Validation Rejection: RPC Return Type Void
- **Scenario**: URPC specifies a non-void returnProperty.
- **Expected behavior**: Rejection with `RuleRPCReturnTypeVoid`.
- **Actual behavior**: Successfully rejected with `RuleRPCReturnTypeVoid`.
- **Result**: **PASS**

---

## Unchallenged Areas

- **C++ Compiler Validation** — Out of scope for current test harness, as we focus exclusively on RDF-level and SHACL-level validation rules.
