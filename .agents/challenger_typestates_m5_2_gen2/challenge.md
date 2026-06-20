# Challenge Report — Typestate Validation

## Challenge Summary

**Overall risk assessment**: MEDIUM

We have evaluated the SHACL and SPARQL validation engine (`ggen` sync with `--validate-only true`) using the standard test harness (`verify_all_rules.sh`) and custom stress test cases (`verify_extra_rules.sh`). While the validation engine correctly catches invalid configurations, we identified three significant design and operational vulnerabilities that could lead to false-negatives or pipeline failures.

---

## Challenges

### [High] Challenge 1: Non-Zero Exit Code Defect on Validation Failures

- **Assumption challenged**: The validation command `ggen sync --validate-only true` returns a non-zero exit code when custom validation rules fail.
- **Attack scenario**: A CI/CD pipeline executes `ggen sync --manifest ggen.toml --validate-only true` as a build gate, assuming that a successful exit (status 0) guarantees a clean, valid ontology. Since `ggen` returns exit code `0` even when custom validation rules fail (aborted generation), the pipeline will let the invalid build pass through unnoticed.
- **Blast radius**: High. Invalid typestates, out-of-bounds stack/heap allocations, and unoptimized shipping builds could be deployed to production.
- **Mitigation**: Update the validation engine CLI to exit with status `1` (or another non-zero code) whenever any custom rule or SHACL validation fails.

### [Medium] Challenge 2: Namespace and Import Scope in Custom SPARQL Rules

- **Assumption challenged**: Custom SPARQL ASK/SELECT validation rules defined in `ggen.toml` are executed against the fully-merged RDF graph (with all imported `.ttl` files resolved).
- **Attack scenario**: If `ggen` runs custom validation rules on the base `core.ttl` graph before resolving and merging imports, rules that check typestate compatibility referencing classes or properties defined solely in the imports (like `ue4:WasmMemoryLayout` or `ue4:CompilerOptimizationLevel` in `typestates.ttl`) would fail to match the query patterns. As a result, the `FILTER NOT EXISTS` clause evaluates to `true`, and the check silently passes (false negative).
- **Blast radius**: Medium. Custom typestate rules could be bypassed if the layout class is only defined in imported graphs and not present in the main ontology being verified.
- **Mitigation**: Ensure that custom rules in `ggen.toml` execute on the fully closed and resolved RDF graph.

### [Low] Challenge 3: Shared Temporary Backup Race Condition

- **Assumption challenged**: The backup and restore logic in the verification script is safe to run in concurrent development or CI environments.
- **Attack scenario**: The backup path `/tmp/core.ttl.bak` is hardcoded. If two concurrent test runs, different subagents, or automated CI executors execute the verification script simultaneously, they will overwrite each other's backup file. This results in restoring a modified or dirty state to `core.ttl`, causing random and non-deterministic test failures.
- **Blast radius**: Low. Non-deterministic test failures in CI.
- **Mitigation**: Use process-specific unique backup file names, for example `/tmp/core.ttl.bak.$$`.

---

## Stress Test Results

We ran the verification suite containing 22 baseline test cases and 5 custom stress test cases. All tests passed under our clean-base control.

| Scenario / Test Case | Expected Behavior | Actual Behavior | Result |
|---|---|---|---|
| **Rule 19**: WASM Initial Memory page alignment check (`50000000` bytes) | Fail with `RuleWasmMemoryLayoutPageAlignment` | Fails with expected error | **PASS** |
| **Rule 20**: WASM Fixed Heap bounds check (`AllowMemoryGrowth = false`, `Initial != Max`) | Fail with `RuleWasmMemoryBoundaries` | Fails with expected error | **PASS** |
| **Rule 21**: Static Baking missing mandated paths | Fail with `RuleStaticBakingPaths` | Fails with expected error | **PASS** |
| **Rule 22**: Static Baking VaRest Prohibition check | Fail with `RuleStaticBakingNoVaRest` | Fails with expected error | **PASS** |
| **Extra 1**: WASM Stack size larger than initial heap (`Stack = 128KB`, `Initial = 64KB`) | Fail with `RuleWasmMemoryBoundaries` | Fails with expected error | **PASS** |
| **Extra 2**: Shipping config using unoptimized build levels (`optFlag = "-O0"`) | Fail with `RuleLinkingConfigurationShipping` | Fails with expected error | **PASS** |
| **Extra 3**: Shipping config with `bOptimize = false` | Fail with `RuleBuildConfigurationConsistency` | Fails with expected error | **PASS** |

---

## Unchallenged Areas

- **C++ Generator Code Compilation** — We did not build the C++ project with the invalid output headers. We only verified that the validation gates correctly rejected the invalid inputs.
- **Performance benchmarks** — We did not measure the memory footprint or CPU overhead of running the SHACL engine during validation.
