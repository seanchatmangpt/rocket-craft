# Remediated Validation Gaps and Alignment Discrepancies

## Files Modified
- `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`:
  - Updated `ue4:InputExecPinConnectedShape` to use `?node a/rdfs:subClassOf* ue4:UK2Node` instead of `?node a ue4:UK2Node_CallFunction` inside `sh:select` query to dynamically check subclasses of K2 nodes.
  - Appended `ue4:InputPinConnectionShape` representing the SHACL counterpart for the custom TOML validation rule `RuleInputPinConnection` (enforcing a maximum of 1 connection on input pins).
  - Appended `ue4:UEdGraphNodeParentageShape` (targeting `rdf:type` via `sh:targetSubjectsOf`) representing the SHACL counterpart for `RuleNodeParentage` (enforcing exactly 1 `nodeOf` relationship to a valid `UEdGraph` for all `UEdGraphNode` subclasses).
  - Appended separate property shapes `ue4:USkeletalMeshComponentRenderingShape` and `ue4:UBoxComponentRenderingShape` replicating the rendering property constraints (`ue4:interactionDistanceClass`, `ue4:materialClass`, `ue4:instancingClass`, `ue4:silhouetteImportanceClass`) to validate these component subclasses without RDFS reasoning.
  - Updated `ue4:UFunctionParameterShape` property shape for `ue4:parameterIndex` by removing `sh:pattern "^[0-9]+$"` and adding `sh:minInclusive 0` constraint.

## Build Status
- **validate_ontology.sh**: PASS (compilation check passed successfully)
- **verify_all_rules.sh**: PASS (all 16 tests executed and passed successfully)

## Pending Issues
- None.
