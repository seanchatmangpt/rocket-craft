# Empirical Verification Report (Handoff)

## 1. Observation

### Baseline Run
Running the baseline validation script `/Users/sac/rocket-craft/validate_ontology.sh` executes successfully:
```
All Gates: ✅ PASSED → Proceeding to generation phase
...
SHACL validation:     PASS (1 SHACL shape files)
All validations passed.
SUCCESS: Ontology validation passed.
```

### SHACL Validator Bypasses and Ignored Rules
Looking at `ShapeLoader::load` in `/Users/sac/ggen/crates/ggen-core/src/validation/shacl.rs`:
```rust
        // Step 1: Find all NodeShapes with targetClass
        let find_shapes_query = r"
            PREFIX sh: <http://www.w3.org/ns/shacl#>
            SELECT ?shape ?targetClass WHERE {
                ?shape a sh:NodeShape .
                ?shape sh:targetClass ?targetClass .
            }
        ";
        ...
        for row in &shape_rows {
            ...
            shape_set.shapes.insert(shape_iri, shape);
        }
```
And `validate_shape` in `/Users/sac/ggen/crates/ggen-core/src/validation/validator.rs`:
```rust
    fn validate_shape(&self, graph: &Graph, shape: &ShaclShape, violations: &mut Vec<Violation>) {
        for (path, constraint) in &shape.properties {
            let effective_severity = constraint.severity;
            // sh:minCount, sh:maxCount, sh:datatype, sh:in, sh:pattern, sh:minLength, sh:maxLength checks...
        }
    }
```

When executing `/tmp/run_empirical_tests.py`, which copies the ontology pack, injects invalid RDF data (e.g. connections between incompatible pins, cross-graph wires, private namespaces, and class hierarchy/typestate errors), and executes `/Users/sac/.local/bin/ggen sync --validate-only true`, all test cases return exit code `0` (Success):
```
Running test case: incompatible_pins...
Result code: 0
Running test case: cross_graph_wires...
Result code: 0
Running test case: invalid_pin_mapping...
Result code: 0
Running test case: exec_vs_data_pins...
Result code: 0
Running test case: namespace_sanity...
Result code: 0
Running test case: missing_class_label...
Result code: 0
Running test case: invalid_cooking_state...
Result code: 0
```

### Custom SPARQL Detections
When running `/tmp/run_empirical_sparql_tests.py`, which executes targeted SPARQL validation queries against the same test cases using `ggen graph query`, all violations are successfully caught:
```
=== TESTING CASE: valid_gundam_pc ===
  [VERIFICATION] Gundam PC is fully connected: found 2 execution flows

=== TESTING CASE: incompatible_pins ===
  [VIOLATION] Caught by Rule A: Pin Connection Direction Check: found 2 matches

=== TESTING CASE: cross_graph_wires ===
  [VIOLATION] Caught by Rule B: Graph Isolation Check: found 2 matches

=== TESTING CASE: invalid_pin_mapping ===
  [VIOLATION] Caught by Rule C: Function Call Pin Mapping Target Integrity: found 1 matches

=== TESTING CASE: pin_param_dir_mismatch ===
  [VIOLATION] Caught by Rule D: Pin Parameter Direction Match: found 1 matches

=== TESTING CASE: exec_vs_data_pins ===
  [VIOLATION] Caught by Rule E: Exec vs. Data Pin Separation: found 2 matches

=== TESTING CASE: namespace_sanity ===
  [VIOLATION] Caught by Rule F: Namespace Sanity: found 1 matches

=== TESTING CASE: missing_class_label ===
  [VIOLATION] Caught by Rule G: Class Label Check: found 1 matches

=== TESTING CASE: invalid_cooking_state ===
  [VIOLATION] Caught by Rule H: Typestate Range Check: found 1 matches
```

---

## 2. Logic Chain

1. **Rule A, B, C, D, E Bypass**: In `validation.shacl.ttl`, the Blueprint validation rules are declared as SPARQL constraints (`sh:sparql`) on shapes. Since `ShapeLoader` only processes constraints declared inside `sh:property` blank nodes and `validator.rs` does not implement `sh:sparql` evaluation, all Blueprint graph compatibility constraints are ignored by `ggen`'s internal validation engine.
2. **BTreeMap Overwrite Bug**: A single `sh:NodeShape` containing multiple `sh:targetClass` declarations (like `ClassLabelShape` targeting `rdfs:Class , owl:Class`) gets overwritten in the `BTreeMap` shapes registry. Only the class processed last (`rdfs:Class`) is preserved. Consequently, classes defined as `owl:Class` (which includes `UObject` and all core C++ classes) are completely bypassed and never validated for labels or comments.
3. **Shape-Level Constraint Bypass**: Naming conventions/checks (like `NamespaceSanityShape`) that place `sh:nodeKind` and `sh:pattern` directly on the `NodeShape` itself instead of nested under a `sh:property` path are not loaded by the compiler's `ShapeLoader`, meaning private URN namespaces (e.g. `urn:private:`) pass validation without errors.
4. **Typestates Unchecked Range**: The typestates in `typestates.ttl` have no property range constraints in the compiler shape rules. Any state property can map to any object type, allowing invalid states (e.g. `ue4:hasCookingState ue4:WasmReady`) to pass.
5. **SPARQL Validation Robustness**: Executing direct SPARQL queries against the ontology graph using `ggen graph query` completely bypasses all compiler SHACL validator shortcomings. It accurately captures all invalid pin direction connections, cross-graph wiring, namespace sanity issues, missing labels, and mismatched typestate values.

---

## 3. Caveats

- We did not compile or modify the `ggen` binary ourselves, adhering to the "Review-only" mandate.
- All testing is performed in `/tmp` to maintain Layout Compliance and prevent metadata pollution of `.agents/`.
- We assume that the `ggen graph query` execution path parses turtle imports correctly.

---

## 4. Conclusion & Adversarial Review

### Challenge Summary
**Overall risk assessment**: HIGH

While the UE4 Reflection and Blueprint Graph Ontology is logically coherent, complete, and correct, the acceptance test harness (`validate_ontology.sh`) suffers from significant false positives. Because the compiler validator ignores `sh:sparql`, shape-level constraints, and multiple target classes, it fails to actively catch any structural or typestate defects in the Blueprint network.

### Challenges

#### [Critical] Challenge 1: Bypassed Blueprint Rules
- **Assumption challenged**: Running `ggen sync --validate-only` verifies Blueprint connection direction, graph isolation, and exec pin separation.
- **Attack scenario**: Connecting an exec pin to a float pin or wiring pins across different graphs compiles and validates with exit code `0`.
- **Blast radius**: Allows malformed blueprint visual logic to proceed to WASM packaging, triggering engine compilation/linking panics or browser crashes.
- **Mitigation**: Execute the direct SPARQL queries (Rules A-E) as an independent pre-flight build gate in the pipeline.

#### [High] Challenge 2: Overwritten Target Classes
- **Assumption challenged**: SHACL shapes like `ClassLabelShape` enforce metadata requirements across both `rdfs:Class` and `owl:Class`.
- **Attack scenario**: The compiler shape loader overwrites `owl:Class` with `rdfs:Class` in the shape registry, allowing core classes (declared as `owl:Class`) to completely bypass label verification.
- **Blast radius**: Missing labels on C++ backbone elements lead to generated headers with blank names or syntax errors.
- **Mitigation**: Split multi-target NodeShapes in `validation.shacl.ttl` into separate shape blocks (one per target class).

#### [Medium] Challenge 3: Ineffective Namespace Sanity
- **Assumption challenged**: Private URNs (`urn:private:`) are rejected.
- **Attack scenario**: Defining a class or property subject with a private URN passes SHACL validation.
- **Blast radius**: Hardcoded paths and opaque URIs pollute the global RDF ontology mapping.
- **Mitigation**: Wrap the `sh:nodeKind` and `sh:pattern` check in a nested `sh:property` on a wildcard path or use our custom SPARQL sanity check (Rule F).

#### [Medium] Challenge 4: Mismatched Typestates
- **Assumption challenged**: Injecting an invalid cooking or packaging status breaks validation.
- **Attack scenario**: Mapping `ue4:hasCookingState` to a `WasmPackagingTypestate` instance passes validation.
- **Blast radius**: Misrouted builds and deployment failures due to packaging ready states linked to uncooked maps.
- **Mitigation**: Implement explicit range checking via SPARQL (Rule H).

---

## 5. Verification Method

### How to Reproduce
1. Run the custom SPARQL validation harness:
   ```bash
   python3 /tmp/run_empirical_sparql_tests.py
   ```
2. Inspect the test outcomes printed to `stdout` and logged to `/tmp/sparql_test_results.json`.
3. Invalidation condition: The test fails if `valid_gundam_pc` triggers any of the rules A-H, or if any of the invalid cases fail to be detected by their respective rules.
