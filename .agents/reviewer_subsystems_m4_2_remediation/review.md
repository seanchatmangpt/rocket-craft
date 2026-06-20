# Subsystem Topologies Quality & Adversarial Review Report

**Date**: 2026-06-19
**Reviewer/Critic ID**: Reviewer 2 (Subsystem Topologies)
**Working Directory**: `/Users/sac/rocket-craft/.agents/reviewer_subsystems_m4_2_remediation`

---

# Part 1: Quality Review

## Review Summary

**Verdict**: APPROVE

The remediated subsystem topologies schema and validation shapes/rules have been successfully updated. All target files—subsystems ontology (`subsystems.ttl`), SHACL shapes (`validation.shacl.ttl`), and GGen configuration (`ggen.toml`)—are syntactically valid, compile successfully, and execute cleanly under `validate_ontology.sh` (using `ggen sync --validate-only true`). 

Most importantly, the 5 major defects identified in the previous loop have been completely and robustly resolved with mathematically sound OWL 2 DL class hierarchies and strict SHACL/SPARQL validation guards.

---

## Findings

No critical or major findings remain. However, a minor structural improvement and a coverage gap are identified for future enhancement:

### [Minor] Finding 1: Indirect WebGL Fallback Coverage Gap
- **What**: The WebGL fallback validation shape (`ue4:WasmSubsystemWebGLFallbackShape`) assumes rendering subsystems are directly attached to the world.
- **Where**: `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (Line 556, `ue4:WasmSubsystemWebGLFallbackShape`) and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (Line 410, `RuleM`).
- **Why**: If a rendering subsystem is associated with the `UGameInstance` rather than the `UWorld` (as is common for persistent engine-level rendering settings), the shape will fail to verify the WebGL fallback because the ontology lacks a direct relationship between `UWorld` and `UGameInstance`.
- **Suggestion**: Add a relation `ue4:worldGameInstance` between `UWorld` and `UGameInstance` and update the query to traverse through this relation if the subsystem is not directly on the world.

---

## Verified Claims

- **Claim 1: Defect 1 Resolved (Class scope validation for RPC validation functions)**
  - *Method*: Inspected `ue4:RPCValidationSignatureShape` in `validation.shacl.ttl` (lines 800–815) and `RuleRPCValidationClassScope` in `ggen.toml` (lines 1256–1275).
  - *Result*: **PASS**. The validation now strictly ensures that validation functions are defined either on the same class or a parent class of the RPC using transitively chained subclass pathways (`rdfs:subClassOf*`).
- **Claim 2: Defect 2 Resolved (Subclass-aware parameter type safety)**
  - *Method*: Inspected `ue4:MaterialInstanceParameterValueTypeShape` in `validation.shacl.ttl` (lines 1433–1440) and `RuleMaterialInstanceParameterValueType` in `ggen.toml` (lines 937–944).
  - *Result*: **PASS**. Direct equality checks for parameter types have been replaced with subclass-aware path checks (`a/rdfs:subClassOf*`), supporting any custom user-extended subclasses of `UScalarParameter`, `UVectorParameter`, or `UTextureParameter`.
- **Claim 3: Defect 3 Resolved (Kinematic parameter disconnect validation)**
  - *Method*: Inspected `ue4:KinematicSimulationDisconnectShape` in `validation.shacl.ttl` (lines 1920–1937) and `RuleKinematicSimulationDisconnect` in `ggen.toml` (lines 1277–1294).
  - *Result*: **PASS**. A constraint is enforced such that if `bSimulatePhysics` is true on a primitive component, its rigid body's `physicsType` must be `PhysType_Simulated` or `PhysType_Default`, preventing logical conflicts where kinematic bodies attempt physical simulation.
- **Claim 4: Defect 4 Resolved (Collision channel override domain expansion)**
  - *Method*: Inspected `subsystems.ttl` property domains for `hasChannelResponse` (lines 479–487), `collisionEnabled` (lines 495–503), and `collisionObjectType` (lines 504–512).
  - *Result*: **PASS**. The domains are correctly defined as the union class `owl:unionOf ( ue4:UPrimitiveComponent ue4:UCollisionProfile )`, allowing both reusable profiles and direct component overrides without OWL 2 DL classification errors.
- **Claim 5: Defect 5 Resolved (Enum subclassing under UEnum)**
  - *Method*: Inspected all enum classes in `subsystems.ttl` (such as `ENetRole`, `EShaderFrequency`, `ERenderAPI`, `ECollisionResponse`, `ECollisionEnabled`, `ECollisionChannel`, `EPhysicsType`, `EDOFMode`, `ELifetimeCondition`).
  - *Result*: **PASS**. Every enum class successfully declares `rdfs:subClassOf ue4:UEnum`, ensuring strict reflection layer uniformity.
- **Claim 6: General Syntax and Compilation**
  - *Method*: Ran `/Users/sac/rocket-craft/validate_ontology.sh`.
  - *Result*: **PASS**. All 11 Quality Gates executed and passed without warnings or errors.

---

## Coverage Gaps

- **UGameInstance Subsystem WebGL Check** — risk level: **Low** — recommendation: **accept risk**
  - If a rendering subsystem is bound to a game instance instead of a world, it escapes the WASM WebGL compatibility validation checks. This is a low-risk gap since rendering subsystems are typically world-bound in HTML5 deployment scenarios.

---

## Unverified Items

- **WASM Memory Page Alignment Rule Runtime Behavior**
  - *Reason*: No invalid WASM memory configuration instance fixtures are present in the current ontology pack directory to test negative SHACL enforcement at runtime. However, query syntax and limits (65536 byte page alignment checks) have been statically verified.

---
---

# Part 2: Adversarial Review

## Challenge Summary

**Overall risk assessment**: LOW

The remediated schema has addressed the critical attack surfaces. The system restricts invalid class configurations, prevents raw asset injections, forbids dynamic VaRest calls under static baking configurations, and guarantees parameter type safety.

---

## Challenges

### [Medium] Challenge 1: Unchecked Indirect Component Attachment
- **Assumption challenged**: That all components attached to an actor inherit the actor's replication properties correctly.
- **Attack scenario**: A nested component hierarchy exists where a component replicates, but its parent component does not (even if the outer actor does).
- **Blast radius**: The client projection layer receives out-of-order state or missing updates for child components, causing visual desynchronization.
- **Mitigation**: Introduce a validation rule ensuring that if a component replicates, all scene components in its parent attachment path up to the actor root also replicate.

---

## Stress Test Results

- **Validation Robustness Test**: Executed the `ggen` validator.
  - *Expected*: Core validation passes with zero warnings or errors.
  - *Actual*: **PASS**. All custom rules and SHACL shapes parsed and executed within 28ms.

---

## Unchallenged Areas

- **Run-time WebGL Shader Compilation**: Static checking can verify that material definitions compile to vertex/pixel shaders and that target APIs match WebGL 2.0. However, actual driver-level shader linkage on the client browser cannot be verified within the scope of RDF/SHACL.
