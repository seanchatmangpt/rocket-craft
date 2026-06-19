# Subsystem Topologies Quality & Adversarial Review Report

**Date**: 2026-06-18
**Reviewer/Critic ID**: Reviewer 2 (Subsystem Topologies)

---

## Review Summary

**Verdict**: REQUEST_CHANGES

The implemented subsystem topologies schema and validation rules in `/Users/sac/.ggen/packs/ue4_ontology/` are syntactically valid, compile successfully, and execute cleanly under `validate_ontology.sh` (using `ggen sync --validate-only true`). The mapping of C++ hierarchies, materials/shaders, physics profiles, kinematics, and networking elements is highly detailed and conforms to the specified scope.

However, the adversarial review has revealed several key robustness gaps, boundary contradictions, and validation omissions (such as a missing scope check for RPC validation functions and class domain constraints) that could cause silent compile-time or runtime failures during C++ generation. Therefore, changes are requested to address these design and validation gaps.

---

## Findings

### [Major] Finding 1: Lack of Class Scope Validation for RPC Validation Functions

- **What**: There is no validation shape or query ensuring that the target `validationFunction` of a `WithValidation` RPC belongs to the same class hierarchy (or class) as the RPC.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (Line 728, `ue4:RPCValidationSignatureShape`) and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (Line 1236, `RuleServerRPCValidationMandatory`).
- **Why**: Under the current configuration, an RPC `myActor:MyRPC` can point to `someUnrelatedClass:SomeFunction` as its `validationFunction` and pass validation. At C++ code generation or compilation time, this will fail because the C++ compiler expects the `MyRPC_Validate` function to be a member of the same class (or base class) as the RPC.
- **Suggestion**: Add a constraint shape in SHACL and a corresponding rule in `ggen.toml` that checks class hierarchy alignment for validation functions:
  ```sparql
  PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
  PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
  SELECT $this ?valFunc
  WHERE {
      $this ue4:bWithValidation true ;
            ue4:validationFunction ?valFunc .
      ?rpcClass ue4:hasFunction $this .
      FILTER NOT EXISTS {
          ?rpcClass rdfs:subClassOf* ?valClass .
          ?valClass ue4:hasFunction ?valFunc .
      }
  }
  ```

### [Major] Finding 2: Class Equality Mismatch for Parameter Type Check under Inheritance

- **What**: The parameter type safety verification for material instances uses direct class equality (`?paramType = ue4:UScalarParameter`) rather than checking subclass hierarchy.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (Line 1417, `ue4:MaterialInstanceParameterValueTypeShape`) and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (Line 937, `RuleMaterialInstanceParameterValueType`).
- **Why**: If a developer extends the ontology with a custom subclass of `UScalarParameter` (e.g., `ue4:UDistanceScalarParameter`), the validation rule will fail to recognize it as a valid scalar parameter, causing a false-positive validation error.
- **Suggestion**: Replace direct equality checks with subclass path matching:
  ```sparql
  # Instead of ?paramType = ue4:UScalarParameter
  EXISTS { ?paramDef a/rdfs:subClassOf* ue4:UScalarParameter }
  ```

### [Medium] Finding 3: Kinematic Parameter Disconnect (bSimulatePhysics vs. physicsType)

- **What**: There is no constraint enforcing logical consistency between component-level physics simulation settings and rigid body kinematic types.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`.
- **Why**: A component can have `ue4:bSimulatePhysics true` while its rigid body is configured with `ue4:physicsType ue4:PhysType_Kinematic`. In UE4, kinematic bodies are driven by keyframes and do not simulate forces, creating a logical contradiction that will lead to unpredictable physics behavior.
- **Suggestion**: Add a SHACL validation rule ensuring that if `bSimulatePhysics` is true, the associated rigid body's `physicsType` is `PhysType_Simulated` or `PhysType_Default`.

### [Medium] Finding 4: Domain Limitation for Collision Channel Overrides

- **What**: The property `ue4:hasChannelResponse` has its domain restricted solely to `ue4:UCollisionProfile`.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` (Line 474).
- **Why**: In UE4, primitive components can directly override channel responses on a component level without using a profile. Defining these direct overrides in RDF using `ue4:hasChannelResponse` will trigger a domain violation under strict OWL 2 DL validation.
- **Suggestion**: Expand the domain of `ue4:hasChannelResponse` to the union of component and profile:
  ```turtle
  ue4:hasChannelResponse a owl:ObjectProperty ;
      rdfs:domain [
          a owl:Class ;
          owl:unionOf ( ue4:UPrimitiveComponent ue4:UCollisionProfile )
      ] ;
      rdfs:range ue4:CollisionChannelResponse .
  ```

### [Minor] Finding 5: Enum Subclassing Inconsistency

- **What**: Some C++ enum equivalents subclass `ue4:UEnum` (such as `ENetRole` and `ELifetimeCondition`), whereas others (like `EShaderFrequency`, `ERenderAPI`, `ECollisionResponse`, `ECollisionEnabled`, `ECollisionChannel`, `EPhysicsType`, `EDOFMode`) do not.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`.
- **Why**: This introduces semantic inconsistency across the reflection layer.
- **Suggestion**: Ensure that all classes representing C++ enums subclass `ue4:UEnum`.

---

## Verified Claims

- **Claim 1**: The ontology is syntactically correct and compiles under the GGen validation tool.
  - *Method*: Executed `/Users/sac/rocket-craft/validate_ontology.sh`.
  - *Result*: **PASS**. Output indicates all quality gates (Manifest Schema, SPARQL Validation, Rule Validation) passed.
- **Claim 2**: Material instance parent chains cannot be cyclic and must resolve to a base `UMaterial`.
  - *Method*: Verified SHACL shapes `ue4:MaterialInstanceParentShape` and `ue4:MaterialInstanceAcyclicityShape` in `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and verified query logic.
  - *Result*: **PASS**.
- **Claim 3**: Server RPCs are strictly validated (require validation function and boolean signature return).
  - *Method*: Inspected `ue4:ServerRPCValidationMandatoryShape` and `ue4:RPCValidationSignatureShape`.
  - *Result*: **PASS**.

---

## Coverage Gaps

- **UGameInstance Subsystem WebGL Check**: If the `URenderingSubsystem` is associated with the `UGameInstance` rather than the `UWorld`, `ue4:WasmSubsystemWebGLFallbackShape` fails to verify the WebGL fallback because the ontology lacks a relation between `UWorld` and `UGameInstance`.
  - *Risk Level*: Medium.
  - *Recommendation*: Introduce a property such as `ue4:worldGameInstance` relating `UWorld` to `UGameInstance` to bridge this gap.

---

## Unverified Items

- **WASM Memory Page Alignment Rule Runtime Behavior**: We verified the syntax of `RuleWasmMemoryLayoutPageAlignment` (which checks page alignment of 65536 bytes). However, we did not verify this shape against an active invalid binary memory config because the ontology pack does not contain test instance fixtures.
  - *Reason not verified*: No invalid memory instance fixtures are present in the current pack.

---

# Adversarial Challenge Report

**Overall risk assessment**: MEDIUM

## Challenges

### [High] Challenge 1: Invalid Verification Target Class
- **Assumption challenged**: That the validation function is correctly scope-bound to the RPC's class.
- **Attack scenario**: A malicious or faulty configuration links an RPC in `AMyPlayerController` to a validation function inside an unrelated actor `AMyNPC`.
- **Blast radius**: SHACL validation passes, but the C++ code generator emits code that references `AMyNPC::SomeFunc_Validate` within the scope of `AMyPlayerController`, leading to compiler errors.
- **Mitigation**: Implement the class scope validation check described in Finding 1.

### [Medium] Challenge 2: False Positive Type Safety Violations
- **Assumption challenged**: That all parameters will directly use base parameter classes (`UScalarParameter`, etc.) without extension.
- **Attack scenario**: The system is extended with specialized subclass parameter types.
- **Blast radius**: The validation engine rejects valid parameters because it evaluates direct class equivalence rather than subClassOf relationships.
- **Mitigation**: Update SHACL shapes to traverse subclasses (`a/rdfs:subClassOf*`).

## Stress Test Results

- **Validation Script Execution**: Checked behavior of `validate_ontology.sh` on the existing codebase.
  - *Expected*: Zero errors, clean validation.
  - *Actual*: **PASS** (Zero errors).

## Unchallenged Areas

- **Unreal HTML5 Packaging Pipeline**: Verification of actual compiled WASM artifact sizes and memory allocation at runtime in the browser was not performed since it requires the UE4 editor/packaging engine and browser harness, which is outside the scope of this RDF/ontology-only review.
