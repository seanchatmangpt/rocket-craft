## 2026-06-18T21:45:18-07:00
Objective: Refactor and enhance `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl`, `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl`, `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`, and `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` to address all critical coverage gaps, logical bugs, and alignment discrepancies identified during exploration.

## Specific Task Instructions

### 1. Refactor `reflection.ttl`
- Explicitly declare `ue4:UObject a owl:Class` with rdfs:label and rdfs:comment.
- Fix the hierarchy of `ue4:USoftClassProperty` by making it a subclass of `ue4:USoftObjectProperty` instead of `ue4:UObjectProperty`.
- Add missing signed/unsigned numeric properties: `ue4:UInt8Property`, `ue4:UShortProperty`, `ue4:UUInt16Property`, `ue4:UUInt32Property`, `ue4:UUInt64Property`.
- Declare collection inner properties:
  - `ue4:innerProperty` (domain: `ue4:UArrayProperty`, range: `ue4:UProperty`)
  - `ue4:keyProperty` (domain: `ue4:UMapProperty`, range: `ue4:UProperty`)
  - `ue4:valueProperty` (domain: `ue4:UMapProperty`, range: `ue4:UProperty`)
  - `ue4:elementProperty` (domain: `ue4:USetProperty`, range: `ue4:UProperty`)
- Declare `ue4:delegateSignature` (domain union of `ue4:UDelegateProperty` and `ue4:UMulticastDelegateProperty`, range: `ue4:UFunction`).
- Define metadata support: `ue4:UMetaData` (class), `ue4:hasMetaData` (property), `ue4:metaKey` (datatype property), `ue4:metaValue` (datatype property).
- Define structured flags: `ue4:isBlueprintWritable` and `ue4:isBlueprintReadOnly` as boolean datatype properties.

### 2. Refactor `blueprints.ttl`
- Add missing `UK2Node` subclasses: `ue4:UK2Node_Knot`, `ue4:UK2Node_Select`, `ue4:UK2Node_MacroInstance`, `ue4:UK2Node_Composite`, `ue4:UK2Node_Timeline`, `ue4:UK2Node_SpawnActorFromClass`, `ue4:UK2Node_ConstructObjectFromClass`, `ue4:UK2Node_FunctionEntry`, `ue4:UK2Node_FunctionResult`, `ue4:UK2Node_ComponentBoundEvent`, `ue4:UK2Node_AddDelegate`, `ue4:UK2Node_RemoveDelegate`, `ue4:UK2Node_ClearDelegate`, `ue4:UK2Node_AssignDelegate`.
- Consolidate and reconcile properties:
  - Establish `ue4:calledFunction` and `ue4:callsFunction` equivalence or alignment.
  - Establish `ue4:connectedTo` and `ue4:linkedTo` equivalence or alignment (so connections using either are checked by SHACL/SPARQL rules).
  - Ensure graph pins strictly use binary directions `Input` and `Output`, while parameters can use `InOut`/`Return`.

### 3. Refactor `validation.shacl.ttl`
- Add Input pin connection count limit: enforce `sh:maxCount 1` on `ue4:connectedTo` specifically for input pins.
- Limit pin categories using `sh:in` to standard categories: `exec`, `bool`, `byte`, `int`, `float`, `double`, `name`, `string`, `text`, `object`, `class`, `struct`, `interface`, `delegate`.
- Update `CharacterCookingStateShape` and `WorldPackagingStateShape` to use `sh:SPARQLTarget` to capture custom subclasses (e.g. subclass of `ACharacter` or `UWorld`) and enforce exactly one state.
- Add `sh:minInclusive 0` to `ue4:parameterIndex`.
- Add a node graph parentage shape checking that all `UEdGraphNode` instances have exactly one `nodeOf` relationship.
- Add a shape to enforce that variable getter/setter nodes reference a valid property.

### 4. Refactor `ggen.toml` (Validation Section)
- Update validation queries to be robust:
  - **RuleC**: Check both `hasParameter` and `parameterOf` to handle inverse properties correctly without OWL entailment.
  - **RuleE**: Handle bidirectional symmetry of `connectedTo` so unidirectional RDF triples are caught.
  - **RuleF & RuleG**: Check multiplicity of states (raise error if 0 or >1 cooking/packaging states exist).
  - **RuleLabel**: Exclude blank nodes by verifying `isIRI(?class)`.
  - **RuleNamespace**: Update check to match SHACL pattern (`http://` or `https://` starts).

### 5. Verification
- Run `/Users/sac/rocket-craft/validate_ontology.sh` to compile and check syntax.
- Run `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh` to ensure all tests pass.
- Write a report of changes to `/Users/sac/rocket-craft/.agents/worker_reflection_blueprints_gen1/changes.md` and handoff at `/Users/sac/rocket-craft/.agents/worker_reflection_blueprints_gen1/handoff.md`.
