# Changes Report - worker_reflection_blueprints_gen1

All files have been modified to address critical coverage gaps, logical bugs, and alignment discrepancies. The validation test runner has been enhanced with 5 new integration tests verifying these constraints, and all tests now pass successfully.

## Modified Files

### 1. `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl`
- Declared `ue4:UObject a owl:Class` with `rdfs:label` and `rdfs:comment`.
- Corrected class inheritance of `ue4:USoftClassProperty` to subclass `ue4:USoftObjectProperty` (previously `ue4:UObjectProperty`).
- Added missing signed/unsigned numeric properties subclassing `ue4:UNumericProperty`:
  - `ue4:UInt8Property`
  - `ue4:UShortProperty`
  - `ue4:UUInt16Property`
  - `ue4:UUInt32Property`
  - `ue4:UUInt64Property`
- Declared inner properties for collections:
  - `ue4:innerProperty` (domain: `ue4:UArrayProperty`, range: `ue4:UProperty`)
  - `ue4:keyProperty` (domain: `ue4:UMapProperty`, range: `ue4:UProperty`)
  - `ue4:valueProperty` (domain: `ue4:UMapProperty`, range: `ue4:UProperty`)
  - `ue4:elementProperty` (domain: `ue4:USetProperty`, range: `ue4:UProperty`)
- Declared `ue4:delegateSignature` with domain union of `ue4:UDelegateProperty` and `ue4:UMulticastDelegateProperty` and range `ue4:UFunction`.
- Added metadata support:
  - `ue4:UMetaData` (class, subclass of `ue4:UObject`)
  - `ue4:hasMetaData` (property, domain `ue4:UObject`, range `ue4:UMetaData`)
  - `ue4:metaKey` (datatype property, domain `ue4:UMetaData`, range `xsd:string`)
  - `ue4:metaValue` (datatype property, domain `ue4:UMetaData`, range `xsd:string`)
- Declared structured flags as boolean datatype properties:
  - `ue4:isBlueprintWritable`
  - `ue4:isBlueprintReadOnly`
- Introduced `ue4:BinaryPinDirection` (subclass of `ue4:PinDirection`) to classify graph pin directions (`ue4:Input` and `ue4:Output`).

### 2. `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl`
- Added missing `UK2Node` subclasses:
  - `ue4:UK2Node_Knot`
  - `ue4:UK2Node_Select`
  - `ue4:UK2Node_MacroInstance`
  - `ue4:UK2Node_Composite`
  - `ue4:UK2Node_Timeline`
  - `ue4:UK2Node_SpawnActorFromClass`
  - `ue4:UK2Node_ConstructObjectFromClass`
  - `ue4:UK2Node_FunctionEntry`
  - `ue4:UK2Node_FunctionResult`
  - `ue4:UK2Node_ComponentBoundEvent`
  - `ue4:UK2Node_AddDelegate`
  - `ue4:UK2Node_RemoveDelegate`
  - `ue4:UK2Node_ClearDelegate`
  - `ue4:UK2Node_AssignDelegate`
- Aligned function properties by declaring `ue4:callsFunction rdfs:subPropertyOf ue4:calledFunction`.
- Aligned connection properties by declaring `ue4:linkedTo rdfs:subPropertyOf ue4:connectedTo`.

### 3. `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
- Updated pin direction validation to enforce `sh:class ue4:BinaryPinDirection`, restricting pin directions to strictly `ue4:Input` and `ue4:Output`.
- Restricted pin category values using `sh:in` to standard categories (`exec`, `bool`, `byte`, `int`, `float`, `double`, `name`, `string`, `text`, `object`, `class`, `struct`, `interface`, `delegate`).
- Refactored `ue4:CharacterCookingStateShape` and `ue4:WorldPackagingStateShape` to target instances via SPARQL checks on `sh:targetSubjectsOf rdf:type` to capture all custom subclasses dynamically and enforce exactly 1 state constraint without requiring engine entailment.
- Added `sh:pattern "^[0-9]+$"` to `ue4:parameterIndex` to enforce non-negative parameter indices.
- Added connection count shape `ue4:InputPinShape` enforcing `sh:maxCount 1` specifically for input pins.
- Added graph node parentage shapes verifying that all `UEdGraphNode` instances have exactly 1 `nodeOf` relationship to a graph.
- Added variable property shapes ensuring getter/setter nodes reference exactly 1 valid `UProperty`.

### 4. `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`
- Updated SPARQL check for **RuleC** to check both `callsFunction`/`calledFunction` and `hasParameter`/`parameterOf`.
- Updated **RuleE** (and RuleA, RuleB, RuleH) to check symmetric and alternative paths `(ue4:connectedTo|^ue4:connectedTo|ue4:linkedTo|^ue4:linkedTo)` to handle bidirectional symmetry.
- Redesigned **RuleF** and **RuleG** to fail (return false) if a character or world has 0 or >1 cooking/packaging states.
- Excluded blank nodes from **RuleLabel** by verifying `isIRI(?class)`.
- Replaced **RuleNamespace** check to align with the SHACL pattern (`^https?://`).

### 5. `/Users/sac/rocket-craft/ggen-validation-tests/verify_all_rules.sh`
- Enhanced the test runner with 5 new integration tests targeting the newly added constraints:
  - Input pin connection count limit
  - Pin category standard limits
  - Variable getter/setter node property checks
  - Graph node parentage checks
  - Non-negative parameter index checks
- Corrected test case 12 to verify input pin connection limits without violating direction matching rules.
