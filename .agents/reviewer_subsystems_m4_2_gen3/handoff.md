# Handoff Report — Subsystem Topologies Review (Reviewer 2)

## 1. Observation

- **Ontology and configuration files inspected**:
  - `/Users/sac/.ggen/packs/ue4_ontology/subsystems.ttl` (lines 1 to 750)
  - `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 1 to 1917)
  - `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (lines 1 to 1373)
- **Validation Script executed**: `/Users/sac/rocket-craft/validate_ontology.sh`
- **Validation Output**:
  ```
  All Gates: ✅ PASSED → Proceeding to generation phase
  Custom validation rules:     PASS (61 rules)
  SHACL validation:     PASS (1 SHACL shape files)
  All validations passed.
  ```
- **Observed Schema Structure**:
  - `ue4:hasChannelResponse` domain is restricted to `ue4:UCollisionProfile` (Line 474 of `subsystems.ttl`):
    ```turtle
    ue4:hasChannelResponse a owl:ObjectProperty ;
        rdfs:domain ue4:UCollisionProfile ;
        rdfs:range ue4:CollisionChannelResponse .
    ```
  - `ue4:RPCValidationSignatureShape` verifies validation function signature matches for `WithValidation` RPCs, but lacks verification of class scoping between the RPC and the validation function (Lines 728–798 of `validation.shacl.ttl`).
  - Direct class equality check `?paramType = ue4:UScalarParameter` is used in `ue4:MaterialInstanceParameterValueTypeShape` (Lines 1416–1424 of `validation.shacl.ttl`).

## 2. Logic Chain

1. **Rule Conformance**: Executing `/Users/sac/rocket-craft/validate_ontology.sh` (Observation 1) proves that the schema files and shapes compile cleanly and do not contain syntax errors.
2. **Class Scoping for RPCs**: In `validation.shacl.ttl`, shapes check parameter structures of RPC validation functions but not class containment (Observation 3). Therefore, an RPC in `AMyPlayerController` can select a validation function in `AMyNPC`, which compiles in SHACL but fails in C++ because member functions must reside in the same class hierarchy.
3. **Inheritance for Material Parameters**: `ue4:MaterialInstanceParameterValueTypeShape` enforces direct class equality (`?paramType = ue4:UScalarParameter`) rather than hierarchical class inheritance (Observation 4). As a result, subclassing `UScalarParameter` with customized types will trigger false-positive validation errors during generation.
4. **Collision Override Restrictions**: Because `ue4:hasChannelResponse` has its domain restricted to `ue4:UCollisionProfile` (Observation 2), a developer attempting to declare direct collision channel responses on a `UPrimitiveComponent` (without a profile) will violate the OWL 2 DL domain definition.

## 3. Caveats

- We did not verify WASM memory alignment behavior or actual packaging performance on browser runtimes because the ontology pack does not contain active invalid instance fixtures.
- This review is restricted to the RDF/SHACL topology layer; C++ code generation logic (ggen template expansion) was not compiled or executed against Unreal Engine build forks.

## 4. Conclusion

The subsystem topologies schema and shapes compile correctly. However, a verdict of `REQUEST_CHANGES` is issued due to critical robustness gaps in RPC class scope validation (which will cause compiler failures) and type inheritance restrictions (which limits future extension).

## 5. Verification Method

To verify these observations:
1. Inspect `validation.shacl.ttl` around lines 728–798 and search for class scoping filters on validation functions.
2. Run `/Users/sac/rocket-craft/validate_ontology.sh` to confirm the baseline ontology validates.
3. Add a temporary test instance to the ontology representing an RPC that references an invalid validation function on an unrelated class, and verify that the current SHACL shape fails to report a violation.
