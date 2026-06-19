# Verification Plan — Milestone 4 (Subsystem Topologies)

## Objective
Launch the full Explorer -> Worker -> Reviewer -> Challenger -> Auditor cycle to analyze, design, implement, and verify the Subsystem Topologies (`subsystems.ttl`) schema. This ensures compliance with OWL 2 DL, GGen semantic authority, and the Projection Law constraints while mapping:
1. **Rendering Pipeline**: Materials (UMaterial, UMaterialInstance, parameters), Shaders (UShaderClass, frequency, parameters), and WebGL/RHI fallbacks.
2. **Physics Layers**: Collision Volumes (UCollisionProfile, channels, responses) and Kinematics (URigidBody, mass, velocity constraints, DOF mode).
3. **Networking Domain**: Replication (ELifetimeCondition, replicated properties) and RPCs (Server, Client, Multicast).

## Phase 1: Exploration
- **Explorer 1 (Role: Rendering Subsystem Explorer)**:
  - Analyze how to represent UE4 materials, shaders, parameter overrides, and WebGL 2.0 / OpenGL ES3 fallback pathways in `subsystems.ttl`.
  - Suggest SHACL shape validations to ensure no cycles in material instances and that valid fallbacks are declared.
- **Explorer 2 (Role: Physics Subsystem Explorer)**:
  - Analyze how to model collision channels, responses, profiles, and rigid body kinematic configurations (mass, damping, max velocities).
  - Suggest validation rules to verify that simulated gravity bodies must have valid collision profiles.
- **Explorer 3 (Role: Networking Subsystem Explorer)**:
  - Analyze replication conditions (ELifetimeCondition) and RPC structures (Server, Client, Multicast) with reliable/validation flags.
  - Suggest validation rules to enforce that Server RPCs on client-owned actors have appropriate validation functions.

## Phase 2: Implementation & Compilation
- **Worker (Role: Subsystem Topologies Worker)**:
  - Read recommendations from the 3 Explorers.
  - Implement full schema in `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`.
  - Add SHACL shapes to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and custom SPARQL ASK rules to `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.
  - Run validation command `/Users/sac/rocket-craft/validate_ontology.sh`.

## Phase 3: Review & Challenge
- **Reviewer 1 & 2 (Role: Subsystem Topologies Reviewer)**:
  - Review the schema and validate against engine design.
  - Verify that OWL 2 DL compliance is strictly maintained.
- **Challenger 1 & 2 (Role: Subsystem Topologies Challenger)**:
  - Run `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` to ensure all 16 negative test cases fail as expected.
  - Verify that invalid topologies (e.g. cyclic material parent links, simulated bodies without collision, server RPCs without validation) are successfully caught.

## Phase 4: Forensic Audit
- **Forensic Auditor (Role: Subsystem Topologies Forensic Auditor)**:
  - Audit changes for authenticity, check for hardcoded test result cheating, and verify that the validation runner produces clean output.

## Gate Criteria
- Baseline compiles and validates successfully via `validate_ontology.sh`.
- All reviewers approve.
- All challengers confirm empirical test coverage.
- Auditor returns CLEAN verdict.
