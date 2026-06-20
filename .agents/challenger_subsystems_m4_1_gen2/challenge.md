# Adversarial Challenge Report: Subsystem Topologies Validation

**Overall risk assessment**: CRITICAL

## Challenge Summary

The validation framework for the UE4 Universal RDF Mapping project contains a critical gap: **SPARQL-based SHACL constraints (`sh:sparql`) are silently ignored by the `ggen` validation runner.**

While the test suite in `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` reports 16/16 tests passing, this is a false positive. Several core validation checks (such as input pin connection limits and node parentage constraints) only appear to pass because they are duplicated as custom SPARQL rules in `ggen.toml`. 

For constraints implemented *solely* as SHACL `sh:sparql` rules (specifically all physics kinematics and collision safety rules), the validation pipeline fails silently. Invalid schemas (e.g., simulated bodies with zero mass or gravity-enabled bodies without collision) are admitted without warning, allowing corrupt topologies to proceed to HTML5/WASM packaging and causing runtime instability (NaN values or falling through the floor).

---

## Confirmed Vulnerabilities (Silent Failures)

### 1. Silent Admission of Zero-Mass Simulated Bodies (SHACL `ue4:SimulatedBodyMassShape`)
- **Assumption challenged**: The validation framework rejects simulated rigid bodies (`PhysType_Simulated`) that lack a declared mass or have a mass of `0.0 kg`.
- **Attack Scenario**: We appended a simulated rigid body with `massKg 0.0` to the ontology.
- **Blast Radius**: `ggen sync` passed validation without any errors. At runtime, a rigid body with zero mass causes PhysX/simulation engines to divide by zero, producing `NaN` velocities and crashing the physics thread.
- **Mitigation**: Port `ue4:SimulatedBodyMassShape` from SHACL into a custom SPARQL validation rule in `ggen.toml`.

### 2. Silent Admission of Simulated Bodies without Gravity-Collision Safety (SHACL `ue4:SimulatedGravityCollisionShape`)
- **Assumption challenged**: The validation framework ensures that simulated bodies with gravity enabled must have active collision (not `NoCollision`) to prevent them from falling through the floor.
- **Attack Scenario**: We appended a simulated rigid body with gravity enabled, but with `collisionEnabled ue4:NoCollision` on its parent component.
- **Blast Radius**: `ggen sync` passed validation without any errors. At runtime, the actor immediately falls through the static level geometry indefinitely, causing out-of-bounds performance degradation.
- **Mitigation**: Port `ue4:SimulatedGravityCollisionShape` from SHACL into a custom SPARQL validation rule in `ggen.toml`.

### 3. Test Harness False Positives due to Exit Code Masking
- **Assumption challenged**: The test runner `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` correctly asserts that the baseline validation passes.
- **Attack Scenario**: The script runs `ggen sync --validate-only true` and checks if it returns a non-zero exit code.
- **Blast Radius**: `ggen sync` returns exit code `0` even when custom validation rules fail. As a result, the baseline validation in the test runner falsely reports success, masking pre-existing validation errors (e.g., `RuleNetWorldSubsystemTopology` which failed on the baseline).
- **Mitigation**: Update the test runner and validation scripts to check for the presence of `"FAIL"` or `"status": "error"` in the JSON output, rather than relying on the process exit code.

---

## Stress Test Results

| Scenario | Expected Behavior | Actual Behavior | Result |
|---|---|---|---|
| Material Instance Loop | Rejected with RuleJ / Acyclicity | Rejected (RuleJ SPARQL error) | **PASS** |
| Negative Parameter Index | Rejected with RuleParameterIndex | Rejected (RuleParameterIndex & SHACL) | **PASS** |
| Zero-Mass Simulated Body | Rejected with SimulatedBodyMassShape | Accepted silently (No errors) | **FAIL** |
| Missing Gravity-Collision Safety | Rejected with SimulatedGravityCollisionShape | Accepted silently (No errors) | **FAIL** |
| Baseline Validation check | Detect errors | Reports success despite errors | **FAIL** |

---

## Attack Surface

### Hypotheses Tested
- **SPARQL target matching in SHACL**: Confirmed that `ggen`'s SHACL engine ignores `sh:sparql` constraints.
- **Comparison operator evaluation**: Confirmed that `xsd:decimal` comparisons like `?mass > 0.0` evaluate correctly when executed as raw SPARQL, proving the bug is in the SHACL engine rather than the SPARQL parser.

### Untested Angles
- **Multiple nested material instances**: Loops of depth > 2 are not tested by the test runner.
- **Concurrent sync races**: Behavior of `ggen sync` when multiple processes write to the same target directory has not been fully stress-tested.
