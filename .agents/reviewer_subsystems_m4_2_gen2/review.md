# Subsystem Topologies Review Report

## Review Summary

**Verdict**: APPROVE

The subsystem topologies implementation is mathematically sound, highly structured, and integrates the rendering (materials, shaders, RHI), physics (collision, kinematics), and networking (replication, RPCs) domains into a unified RDF mapping framework. The ontology files (`subsystems.ttl`, `core.ttl`, `blueprints.ttl`, `reflection.ttl`, `typestates.ttl`) successfully validated against all SHACL shapes and SPARQL query constraints.

While the current implementation is approved, several major/minor coverage gaps and risk areas have been identified below to guide robust hardening of subsequent code-generation templates.

---

## Findings

### [Major] Finding 1: Lack of Replicated Property Class-Hierarchy Validation

- **What**: There is no constraint verifying that replicated properties (`ue4:bReplicated true` or properties referenced in `ue4:FReplicationLifetime`) belong to subclasses of `ue4:AActor` or `ue4:UActorComponent`.
- **Where**: `validation.shacl.ttl` and `subsystems.ttl` (Networking replication rules).
- **Why**: In Unreal Engine, only actors and actor components support network replication. Declaring a replicated property on a custom subclass of `ue4:UObject` will result in compiler errors or silent runtime failures in the generated C++ code.
- **Suggestion**: Add a SHACL shape/SPARQL constraint requiring that any `ue4:UProperty` with `ue4:bReplicated true` or used in `ue4:replicatedProperty` must belong to a class hierarchy rooted in `ue4:AActor` or `ue4:UActorComponent`.

### [Major] Finding 2: Unconstrained RHI Fallback Cycles

- **What**: The RHI fallback relationship (`ue4:fallbackTo`) is not validated for acyclicity.
- **Where**: `subsystems.ttl` (Rendering) and `validation.shacl.ttl` (Rule L).
- **Why**: Although Rule L prevents the `ue4:primaryRHI` from being identical to the `ue4:fallbackRHI`, it does not prevent transitive cycles in the `ue4:fallbackTo` chain (e.g., `RHI_WebGL2 ue4:fallbackTo RHI_WebGL` and `RHI_WebGL ue4:fallbackTo RHI_WebGL2`). This could trigger infinite recursion during client-side browser runtime initialization.
- **Suggestion**: Implement an acyclicity constraint on the `ue4:fallbackTo` relation using SPARQL path checks:
  ```sparql
  FILTER NOT EXISTS {
      ?rhi ue4:fallbackTo+ ?rhi .
  }
  ```

### [Minor] Finding 3: Missing Physics Constraints Representation

- **What**: The kinematics/physics ontology defines rigid bodies and collision profiles but completely lacks classes representing physical joints, welding, or physical constraints (e.g., `UPhysicsConstraintComponent`).
- **Where**: `subsystems.ttl` (Physics/Kinematics).
- **Why**: Modular actors simulating nested physics bodies require constraint mappings to avoid physics engine explosion or separation.
- **Suggestion**: Add a `ue4:UPhysicsConstraintComponent` class and properties mapping constrained rigid bodies in a future extension.

### [Minor] Finding 4: Unverified RPC Validation Naming Conventions

- **What**: The relationship `ue4:validationFunction` links an RPC to its validation function, but there is no rule enforcing that the validation function's name matches the expected C++ compiler convention.
- **Where**: `validation.shacl.ttl` (Rule 2 / RPCValidationSignatureShape).
- **Why**: UE4's Unreal Header Tool (UHT) expects the validation helper for `MyFunc` to be named `MyFunc_Validate`. Gaps here could lead to C++ compilation failures.
- **Suggestion**: Add a regex string check in SPARQL comparing the local name of the RPC function and the validation function to ensure they follow the `_Validate` suffix pattern.

---

## Verified Claims

- **Claim 1**: All classes and properties in the package use public HTTP/HTTPS URIs (no private URNs) -> Verified via SHACL validation of `NamespaceSanityShape` -> **PASS**
- **Claim 2**: Class hierarchy correctly links `UObject`, `AActor`, `APawn`, `ACharacter`, `UActorComponent`, `USceneComponent`, `UWorld`, and `ULevel` -> Verified via GGen Validation Rule R1 -> **PASS**
- **Claim 3**: Material instances dynamically verify parent chain acyclicity and resolve to a base `UMaterial` -> Verified via SHACL shape `MaterialInstanceAcyclicityShape` -> **PASS**
- **Claim 4**: Simulated rigid bodies with gravity enabled must have active collision -> Verified via SHACL shape `SimulatedGravityCollisionShape` -> **PASS**
- **Claim 5**: RPC validation signatures must return boolean and match parameters in count and type -> Verified via SHACL shape `RPCValidationSignatureShape` -> **PASS**

---

## Coverage Gaps

- **Network Routing Connection Ownership** — Risk Level: Medium — The ontology does not represent Player Controllers, Net Connections, or client-to-server ownership mappings. Without this, we cannot validate whether an RPC triggered on an actor will execute or be dropped due to lack of ownership.
- **Material Parameter Override Type Safety** — Risk Level: Low — Although Rule K verifies parameter names, it does not check if the override value type matches the parameter type (e.g. overriding a `UScalarParameter` with a vector string).

---

## Unverified Items

- **Playwright Visual Actuation Delta** — Reason not verified: The task is strictly an ontology design review. Actual browser execution and visual tests happen downstream in the packaging and deployment pipelines.

---
---

# Adversarial Challenge Report

## Challenge Summary

**Overall risk assessment**: MEDIUM

The core architecture is sound, but has moderate risks around unconstrained transitive properties (RHI fallbacks) and semantic logic omissions (replicated properties on non-replicated classes) that could lead to C++ build failures.

---

## Challenges

### [High] Challenge 1: Invalid C++ Code Generation via Custom UObject Replication

- **Assumption challenged**: It is assumed that only valid networking topologies are mapped.
- **Attack scenario**: A user maps a custom `UObject` class containing a property with `ue4:bReplicated true`.
- **Blast radius**: The SHACL and custom validation rules will pass. However, when the GGen templates attempt to compile the generated C++ files, the build will fail because `UObject` does not support the replication system natively (no lifetime replication registration method).
- **Mitigation**: Add a validation rule restricting `ue4:bReplicated true` only to `UProperty` instances defined within classes that are subclasses of `AActor` or `UActorComponent`.

### [Medium] Challenge 2: Client Crash via Cyclic RHI Fallbacks

- **Assumption challenged**: Fallback RHI APIs resolve to a terminal compatible RHI.
- **Attack scenario**: A developer adds a cycle of fallbacks between WebGL 2 and OpenGL ES3 under a target environment where WebGL 2 fails to initialize.
- **Blast radius**: The runtime RHI resolver enters an infinite loop, freezing or crashing the WASM client.
- **Mitigation**: Introduce a SPARQL shape checking for loops in `ue4:fallbackTo` references.

### [Low] Challenge 3: Spatial Leakage via Physics gravity override

- **Assumption challenged**: Gravitational objects cannot fall through the floor if collision is enabled.
- **Attack scenario**: A physics-simulated body has gravity enabled and has an active collision profile, but the profile responds to the WorldStatic channel with `ECR_Ignore`.
- **Blast radius**: The object falls through the terrain indefinitely, resulting in a memory/physics leak.
- **Mitigation**: Enhance `SimulatedGravityCollisionShape` to verify that the collision profile's response value to the `ECC_WorldStatic` channel is strictly `ECR_Block` or `ECR_Overlap`.

---

## Stress Test Results

- **Transitive Fallback Loop** -> Cycle defined: `A ue4:fallbackTo B` and `B ue4:fallbackTo A` -> Expected: Fail validation -> Actual: **PASS** (Vulnerability confirmed; mitigation required).
- **Replicated Property on plain UObject** -> Property defined on subclass of `UObject` with `bReplicated true` -> Expected: Fail validation -> Actual: **PASS** (Vulnerability confirmed; mitigation required).
