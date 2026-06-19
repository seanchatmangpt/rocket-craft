# Challenge Report

## Challenge Summary

**Overall risk assessment**: MEDIUM

While the validation suite is highly effective and successfully catches all 25 core validation rule failures and 5 extra packaging-related constraints, there is a structural divergence between SHACL constraints (`validation.shacl.ttl`) and custom validation rules (`ggen.toml`). Some constraints implemented in SHACL (such as `ue4:RPCReturnTypeVoidShape`) do not trigger SHACL validation errors because the underlying validation engine does not execute or enforce SHACL SPARQL-based constraints. Instead, the pipeline relies on the custom SPARQL ASK queries defined in `ggen.toml` to reject invalid configurations. If these custom rules are disabled or modified, critical safety checks could be bypassed.

---

## Challenges

### [High] Challenge 1: Silent Bypass of SHACL SPARQL-Based Constraints

- **Assumption challenged**: SHACL shapes utilizing `sh:sparql` (such as `ue4:RPCReturnTypeVoidShape` and `ue4:ServerRPCValidationMandatoryShape`) are fully evaluated by the validation pipeline.
- **Attack scenario**: A developer adds a new SHACL shape with complex logic using `sh:sparql` inside `validation.shacl.ttl` and assumes it is enforced. An adversary or mistake introduces an invalid triple that violates this shape. Since `ggen`'s SHACL engine does not enforce SPARQL-based shapes, the violation is not caught in the SHACL validation phase.
- **Blast radius**: High. Validation safety nets targeting RPC return types, validation signatures, and server-side RPC checks could be bypassed if they are not explicitly duplicated as custom SPARQL validation rules in `ggen.toml`.
- **Mitigation**: Standardize all complex or transitivity-based validations as custom rules in `ggen.toml` or ensure the SHACL validator is upgraded to fully support the SHACL-SPARQL extension.

### [Medium] Challenge 2: Target Class Subclass Entailment Limitations

- **Assumption challenged**: SHACL shapes targeted at a parent class via `sh:targetClass` (such as targeting `ue4:URPC` in `ue4:RPCOnReplicatedClassShape` or `ue4:RPCValidationSignatureShape`) automatically capture instances of subclasses (e.g., `ue4:UServerRPC`, `ue4:UClientRPC`, `ue4:UNetMulticastRPC`).
- **Attack scenario**: Without RDFS/OWL subclass entailment enabled in the SHACL parser, targeting `ue4:URPC` will fail to select instances typed as `ue4:UServerRPC`. Consequently, checks on these instances are silently skipped.
- **Blast radius**: Medium. Developer confidence in target-class validation is misplaced, necessitating explicit subclass list definitions or custom SPARQL validation rules.
- **Mitigation**: Avoid relying on `sh:targetClass` for abstract base classes in SHACL. Instead, use target queries or custom SPARQL rules in `ggen.toml` that explicitly resolve the subclass hierarchy via `a/rdfs:subClassOf*`.

### [Low] Challenge 3: Redundant and Out-of-Sync Validation Rules

- **Assumption challenged**: The custom SPARQL validation rules in `ggen.toml` and the SHACL shapes in `validation.shacl.ttl` are kept in perfect synchronization.
- **Attack scenario**: A developer updates or adds a validation rule (e.g., modifying acceptable WebGL texture formats) in `ggen.toml` but forgets to update `validation.shacl.ttl`. Over time, the validation definition drifts, leading to inconsistent behaviors and failing to provide a single source of truth for the ontology's semantics.
- **Blast radius**: Low. Higher maintenance overhead and confusion during local developer checks vs. centralized CI testing.
- **Mitigation**: Establish validation rules exclusively in one configuration format (preferably `ggen.toml` for custom queries) or generate one from the other during build/compilation tasks.

---

## Stress Test Results

- **Scenario 1: Non-void RPC Return Value**
  - **Input**: A `ue4:URPC` function instance `gundam:BadRPCReturn` containing `ue4:returnProperty`.
  - **Expected behavior**: Validation fails, identifying the non-void return property on the RPC.
  - **Actual behavior**: Validation fails with custom validation rule error `RuleRPCReturnTypeVoid`. SHACL validation reports `PASS`.
  - **Verdict**: PASS (caught by `ggen.toml` rule).

- **Scenario 2: Unregistered Collision Profile**
  - **Input**: A component `gundam:GundamCollision` configured with collision profile `gundam:GundamUnregisteredProfile` which is not registered in the world's physics subsystem.
  - **Expected behavior**: Validation fails, rejecting the unregistered collision profile.
  - **Actual behavior**: Validation fails with custom validation rule error `RuleComponentCollisionProfileRegistration`.
  - **Verdict**: PASS.

- **Scenario 3: Server RPC Missing Validation**
  - **Input**: A server RPC `gundam:GundamServerRPC` that does not specify `WithValidation` or a `validationFunction`.
  - **Expected behavior**: Validation fails, enforcing mandatory validation for Server RPCs.
  - **Actual behavior**: Validation fails with custom validation rule error `RuleServerRPCValidationMandatory`.
  - **Verdict**: PASS.

- **Scenario 4: Material Instance Parameter Value Type Mismatch**
  - **Input**: A material instance `gundam:GundamMaterialInstance` assigning a vector value `(R=1.0,G=0.0,B=0.0,A=1.0)` to a scalar parameter `GundamScalarParam`.
  - **Expected behavior**: Validation fails, enforcing type safety on material parameters.
  - **Actual behavior**: Validation fails with custom validation rule error `RuleMaterialInstanceParameterValueType`.
  - **Verdict**: PASS.

---

## Unchallenged Areas

- **Unreal HTML5 WASM Native Compiler Optimization Checks** — Out of scope. We verified the metadata configuration validation (-O0 flag rejection for Shipping builds), but did not verify the actual compilation of C++ source targets using the WebGL compiler toolchain.
