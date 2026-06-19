# Challenge Report — Subsystem Topologies Validation Rules

## Challenge Summary

**Overall risk assessment**: MEDIUM

While the validation rules themselves are mathematically precise and successfully reject invalid schemas in the 32 automated test cases, the validation test harness contains critical design vulnerabilities regarding state isolation and loose assertion checks.

---

## Challenges

### [High] Challenge 1: Lack of Robust Harness Cleanup (State Contamination)

- **Assumption challenged**: The test script assumes that all tests execute cleanly or that manual restoration isn't required after a script exits early.
- **Attack scenario**: The scripts use `set -eo pipefail` to immediately exit on failure. However, they lack shell `trap` handlers on `EXIT` or `ERR`. If any rule fails (e.g., during development or regression), the script aborts immediately *before* calling the `cleanup` or `restore` functions. On the next execution, the `setup` function runs, backing up the already-dirty `core.ttl` file. This contaminates `/tmp/core.ttl.bak` and `/tmp/core_extra.ttl.bak`.
- **Blast radius**: High. Corrupts the local workspace `core.ttl`, causes unrelated subsequent tests to fail (e.g., Test 4 failing with RuleC instead of RuleD), and prevents developers from obtaining clear error signals without manual recovery.
- **Mitigation**: Implement shell traps in both `verify_all_rules.sh` and `verify_extra_rules.sh` to guarantee cleanup:
  ```bash
  trap cleanup EXIT
  ```

### [Medium] Challenge 2: Loose Test Case Assertions (Grep False Positives)

- **Assumption challenged**: `run_test_case` assumes that a test is successful if the output contains the expected error string (`grep -q "$expected_error"`).
- **Attack scenario**: If a test case is contaminated by previous failures and emits multiple validation errors (e.g., both `RuleC` and `RuleD`), the test runner will still report `PASS` because the grep matches the expected error. It fails to detect that unintended rules are also failing, masking side-effects and dirty states.
- **Blast radius**: Medium. Masked bugs and side-effects can slip through the test suites undetected.
- **Mitigation**: Assert that the output contains *only* the expected validation failure or precisely match the structured output of the `ggen` JSON validation report.

### [Low] Challenge 3: Kinematic Simulation Disconnect Edge Case

- **Assumption challenged**: `RuleKinematicSimulationDisconnect` assumes that components simulating physics always declare a rigid body.
- **Attack scenario**: If a component has `ue4:bSimulatePhysics true` but lacks the `ue4:hasRigidBody` relationship entirely, the SPARQL rule does not trigger. In Unreal Engine, simulating physics without a valid rigid body setup is a structural defect (physics won't actuate or the engine will warn).
- **Blast radius**: Low-to-Medium. Dampened physics simulation setups could pass validation silently.
- **Mitigation**: Update or add a companion rule to verify that components with `ue4:bSimulatePhysics true` must define a valid `ue4:hasRigidBody` relation.

---

## Stress Test Results

| Test Scenario / Command | Expected Behavior | Actual Behavior | Pass/Fail |
|---|---|---|---|
| Run `verify_all_rules.sh` (Baseline) | All 27 tests pass successfully | All 27 tests passed | **PASS** |
| Run `verify_extra_rules.sh` (Baseline) | All 5 tests pass successfully | All 5 tests passed | **PASS** |
| State Contamination Test: Exit `verify_extra_rules.sh` mid-way | `core.ttl` left modified | `core.ttl` left with appended blocks; subsequent runs fail until manually restored | **FAIL** (Harness Vulnerability) |
| Test 26 (RPC Validation Class Scope Mismatch) | Correctly rejects RPC validation function defined outside class hierarchy | Rejects with `RuleRPCValidationClassScope` | **PASS** |
| Test 27 (Kinematic Simulation Disconnect) | Correctly rejects simulating components mapped to kinematic bodies | Rejects with `RuleKinematicSimulationDisconnect` | **PASS** |

---

## Unchallenged Areas

- **C++ Class Generation**: We focused strictly on the validation rules and did not challenge the subsequent C++ header generation output because the pipeline aborted on validation failures as designed by the Agent Jidoka rule.
