# Subsystem Topologies Integration Change Log

This document records the changes made to the UE4 Universal RDF Mapping subsystems topology modeling, SHACL shape validation, and project-scoped rules.

## Summary of Modifications

### 1. Schema Extensions (`subsystems.ttl`)
Modified `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` and `/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl` to introduce:
- **Primitive Component Hierarchy:** Declared `UPrimitiveComponent` inheriting from `USceneComponent`, and subclasses `UMeshComponent`, `UStaticMeshComponent`, `USkeletalMeshComponent`, `UShapeComponent`, `UBoxComponent`, `USphereComponent`, and `UCapsuleComponent` to provide complete alignment with the native Unreal C++ layout.
- **Network Roles & Controllers:** Integrated networking roles (`ENetRole` with individuals `ROLE_None`, `ROLE_SimulatedProxy`, `ROLE_AutonomousProxy`, and `ROLE_Authority`), actor network role properties (`role`, `remoteRole`), non-physical actors controllers (`AController` and `APlayerController`), and pawn possession relationships (`possesses`, `possessedBy`).
- **Domain & Range Corrections:**
  - Expanded the domain of `parameterName` to `ue4:UMaterialParameterValue` and `ue4:UMaterialParameter` using a union to avoid class punning and semantic mismatch when queried on parameter definitions.
  - Restricted the domain of `hasCollisionProfile` and `hasRigidBody` from `USceneComponent` to `UPrimitiveComponent`.
  - Configured union domains for `collisionEnabled` and `collisionObjectType` to allow direct component overrides or profile-level overrides without RDFS classification punning.
- **Kinematics Extensions:** Introduced `bSimulatePhysics` on `UPrimitiveComponent` and `bOverrideMass` on `URigidBody`.
- **Replication Registration Links:** Introduced `hasReplicationLifetime` to relate `UClass` to `FReplicationLifetime` and `actorOwner` to relate an actor to its network routing owner.

### 2. SHACL Validation Shapes (`validation.shacl.ttl`)
Appended new validation rules to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and `/Users/sac/rocket-craft/ggen-validation-tests/shacl/validation.shacl.ttl`:
- **Rendering Parameter Type Safety (`MaterialInstanceParameterValueTypeShape`):** Validates that overrides in material instances match the exact value type (scalar, vector, or texture) defined in the base material.
- **Value Type Exclusivity (`UMaterialParameterValueExclusionShape`):** Enforces that a parameter value contains exactly one type of value.
- **Shader Integrity shapes (`UShaderClassShape`, `UShaderParameterShape`, `UMaterialCompiledShadersShape`):** Verifies shader execution stages, parameter structures, and that a base material compiles to at least one Vertex and one Pixel shader.
- **RHI Fallback Acyclicity and Reachability (`ERenderAPIFallbackAcyclicityShape`, `URenderingSubsystemFallbackReachabilityShape`):** Detects loops and verifies that the configured fallback RHI is reachable.
- **Packaging RHI Alignment (`PackagingTargetRhiSupportShape`):** Validates profile API support by the world's rendering subsystem.
- **Physics Shapes (`UCollisionProfileShape`, `CollisionProfileUniqueChannelsShape`, `PhysicsSubsystemUniqueProfileNamesShape`, `ComponentCollisionProfileRegistrationShape`, `RigidBodyPhysicalLimitsShape`, `SymmetricCollisionResponseWarningShape`):** Ensures collision channels and profile names are unique, damping values are non-negative, and used profiles are registered.
- **Replication Consistency (`RPCReturnTypeVoidShape`, `ClientRPCValidationProhibitedShape`, `ServerRPCValidationMandatoryShape`, `StrengthenedComponentReplicationOwnerShape`, `ReplicationConditionConsistencyShape`, `RepNotifyRequiresReplicationShape`, `CompleteReplicatedPropertySpecificationShape`, `RPCParameterObjectTypeSafetyShape`):** Catch orphaned replicated components, void return constraints, and anti-cheat validation compliance.

### 3. GGen Configuration Rules (`ggen.toml`)
Appended corresponding SPARQL ASK validation rules to `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` and `/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml` to enforce validation constraints at the GGen sync level.

### 4. Test Suite Enhancements (`verify_all_rules.sh`)
Enhanced `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` to include test cases verifying the new validations:
- **Test 23 (Material Instance Parameter Value Type Safety check):** Verifies vector overrides of scalar parameters fail.
- **Test 24 (SHACL Unregistered Collision Profile Usage check):** Verifies components using unregistered collision profiles fail.
- **Test 25 (SHACL Server RPC missing validation check):** Verifies Server RPCs without validation functions fail.

## Verification Results
- **Pack Validation (`validate_ontology.sh`):** Exited with code `0` (Success, 61 custom validation rules and SHACL shapes validated).
- **Test Runner (`verify_all_rules.sh`):** All `25 / 25` test cases passed successfully.
- **Extra Verification (`verify_extra_rules.sh`):** All `5 / 5` test cases passed successfully.
