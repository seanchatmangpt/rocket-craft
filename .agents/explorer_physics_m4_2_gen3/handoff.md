# Handoff Report: Physics Subsystem Exploration

## 1. Observation
- **Observation A:** File `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` targets class definitions that do not exist in the Turtle ontologies:
  - Line 386: `sh:targetClass ue4:USkeletalMeshComponent ;`
  - Line 411: `sh:targetClass ue4:UBoxComponent ;`
  - Grep search in `/Users/sac/.ggen/packs/ue4_ontology` for `UBoxComponent`, `USkeletalMeshComponent`, and `UPrimitiveComponent` confirmed they are not defined as OWL classes or subclasses in `core.ttl` or `subsystems.ttl`.
- **Observation B:** File `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` defines domain limits on collision properties:
  - Line 367: `ue4:collisionEnabled a owl:ObjectProperty ; ... rdfs:domain ue4:USceneComponent`
  - Line 373: `ue4:collisionObjectType a owl:ObjectProperty ; ... rdfs:domain ue4:USceneComponent`
- **Observation C:** File `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` utilizes `collisionEnabled` on a profile instance at lines 647-648:
  - `?comp ue4:hasCollisionProfile ?profile . ?profile ue4:collisionEnabled ue4:NoCollision .`
- **Observation D:** Missing properties in `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl`:
  - Grep search for `bSimulatePhysics`, `bOverrideMass`, and `PhysicsConstraint` returned no results in any of the Turtle files.
- **Observation E:** Command `ggen sync --validate-only true` executed in `/Users/sac/.ggen/packs/ue4_ontology` succeeds with exit code 0 under the current schema configuration.

---

## 2. Logic Chain
- **Step 1 (Inconsistent Targets):** Based on **Observation A**, `USkeletalMeshComponent` and `UBoxComponent` are targeted by SHACL but do not exist in the ontology. This creates an architectural gap where generated/baked assets instantiated with these classes have no formal subclass hierarchy back to `USceneComponent` or `UObject`.
- **Step 2 (Classification Conflict):** Based on **Observation B**, the domain of `collisionEnabled` is strictly `USceneComponent`. Based on **Observation C**, the validation rules use `collisionEnabled` on `UCollisionProfile`. In RDFS reasoning, this forces any `UCollisionProfile` to be inferred as an instance of `USceneComponent`, which violates class classification boundaries.
- **Step 3 (Kinematics & Mass Limits):** Based on **Observation D**, a lack of `bSimulatePhysics` and `bOverrideMass` means the semantic authority cannot tell the compilation engine whether to activate physics simulation on the component or override calculated mass with explicit kilogram values. The absence of Joint/Constraint classes prevents modeling multi-body physical joints.
- **Step 4 (Validation Gaps):** Since `ggen` sync validation passes (**Observation E**), the current SHACL shape list is structurally valid but incomplete. It does not verify that collision profiles have unique channels, that profile names are unique in a subsystem, that components use registered profiles, or that simulated bodies are registered in the physics subsystem.

---

## 3. Caveats
- **Assumption:** We assume that the compiler/generator utilizes the standard class naming scheme of Unreal Engine 4 (e.g. `UBoxComponent` is preferred over `FBoxComponent` or similar custom structures).
- **Residual Risks:** Only metadata is analyzed; the physical WASM client behavior under Playwright has not been replayed by this explorer because this is a read-only investigation.

---

## 4. Conclusion
The UE4 physics layers have a robust foundation but suffer from dangling target classes, a semantic RDFS domain mismatch on collision properties, and a lack of support for advanced kinematics (physics constraints and explicit mass overrides). To resolve this, we recommend:
1. Defining the `UPrimitiveComponent` class tree in `subsystems.ttl` to model primitive components and shape volumes.
2. Expanding the domains of `collisionEnabled` and `collisionObjectType` to a union of `UPrimitiveComponent` and `UCollisionProfile`.
3. Adding five new SHACL shape rules to validate unique profile names, channel uniqueness, unregistered profile usage, and physical parameter limits.

Detailed code proposals and diffs are documented in `analysis.md` in the working directory.

---

## 5. Verification Method
- **Ontology Sanity Verification:** Run `ggen sync --validate-only true` in `/Users/sac/.ggen/packs/ue4_ontology` to ensure syntax remains valid after class and domain updates.
- **SHACL Rule Verification:** Apply the proposed SHACL shapes from `analysis.md` to `validation.shacl.ttl` and test against a sample graph containing conflicting profiles or unregistered components to verify they are successfully caught and reported by `ggen` as violations.
