# Analysis of UE4 Reflection System and Blueprint Graph Ontology

## Executive Summary
The current UE4 ontology files `reflection.ttl` and `blueprints.ttl` define only skeleton taxonomies, leaving class relationships, property specifiers, execution pins, node connectivity, and the integration between C++ reflection and Blueprint graphs unimplemented. By mapping concepts from the native `blueprint-rs` AST implementation to RDF schemas, we have designed complete ontology extensions that pass all `ggen` SHACL rules and semantic validations.

---

## 1. Direct Observations & Gap Analysis

We examined the existing ontology files in `/Users/sac/.ggen/packs/ue4_ontology/` and compared them against the Rust AST definition in `/Users/sac/rocket-craft/blueprint-rs/blueprint-core/src/types.rs` and `ast.rs`. 

### Summary of Existing Ontology Files
| File Path | Description | Existing Skeleton Classes | Missing Concepts |
|---|---|---|---|
| `reflection.ttl` | Reflection / Class metadata | `UField`, `UStruct`, `UClass`, `UProperty`, `UFunction` | Property subclass tree, structs/enums, container types, flags, member relationships (`hasField`, `hasProperty`, `superStruct`). |
| `blueprints.ttl` | Blueprint execution graphs | `UEdGraph`, `UEdGraphNode`, `UK2Node` | Subclasses of `UK2Node` (CallFunction, VariableGet/Set, Branch, etc.), pins (`UEdGraphPin`), connections, parameter wiring, and links to reflection entities. |

### Verbatim Snippets from Existing Files
In `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl` (lines 12–35):
```turtle
ue4:UField a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UField" ;
    ...
ue4:UStruct a owl:Class ;
    rdfs:subClassOf ue4:UField ;
    ...
ue4:UClass a owl:Class ;
    rdfs:subClassOf ue4:UStruct ;
    ...
ue4:UProperty a owl:Class ;
    rdfs:subClassOf ue4:UField ;
    ...
ue4:UFunction a owl:Class ;
    rdfs:subClassOf ue4:UStruct ;
```

In `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl` (lines 12–26):
```turtle
ue4:UEdGraph a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    ...
ue4:UEdGraphNode a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    ...
ue4:UK2Node a owl:Class ;
    rdfs:subClassOf ue4:UEdGraphNode ;
```

---

## 2. Ontology Gap Resolution Details

To support the physical AST components found in `blueprint-core`, we must expand the ontologies across three key dimensions:

### A. C++ Reflection Hierarchy (`reflection.ttl`)
* **UStruct Subclasses:** Add `UScriptStruct` (C++ structs) and `UEnum` (enumerations).
* **UProperty Subclasses:** Reflect the full specifier type hierarchy:
  * `UBoolProperty`
  * `UNumericProperty` (with leaf classes `UIntProperty`, `UFloatProperty`, `UDoubleProperty`, `UByteProperty`, `UInt64Property`).
  * `UObjectProperty` (with leaf classes `UClassProperty`, `USoftObjectProperty`, `USoftClassProperty`, `UWeakObjectProperty`, `ULazyObjectProperty`).
  * `UInterfaceProperty`
  * `UStructProperty` (points to inline `UScriptStruct` structs).
  * `UArrayProperty`, `UMapProperty`, `USetProperty` (collections).
  * `UStrProperty`, `UNameProperty`, `UTextProperty` (strings/text).
  * `UEnumProperty` (points to `UEnum`).
  * `UDelegateProperty`, `UMulticastDelegateProperty` (callbacks/events).
* **Semantic Relationships:**
  * `ue4:hasField` / `ue4:hasProperty` / `ue4:hasFunction` (relates scopes to members).
  * `ue4:superStruct` (reflects class and struct inheritance).
  * `ue4:propertyType` (relates properties to class/struct/enum definitions).
  * `ue4:returnProperty` / `ue4:hasParameter` (function signatures).
  * `ue4:classFlags` / `ue4:functionFlags` / `ue4:propertyFlags` (datatype strings/bitmasks).

### B. Blueprint Execution Graph Nodes (`blueprints.ttl`)
* **Core Blueprint Classes:**
  * `UBlueprint`: The master asset container.
  * `UEdGraphPin`: Connection points matching the `Pin` struct in `ast.rs`.
* **K2Node Execution Subclasses:**
  * `UK2Node_Event` and `UK2Node_CustomEvent` (execution entry points).
  * `UK2Node_CallFunction` and `UK2Node_CommutativeAssociativeBinaryOperator` (actions and operators).
  * `UK2Node_Variable`, `UK2Node_VariableGet`, and `UK2Node_VariableSet` (state access).
  * `UK2Node_ExecutionSequence` (Sequence flow).
  * `UK2Node_IfThenElse` (Branch flow).
  * `UK2Node_DynamicCast` (Cast nodes).
  * `UK2Node_Literal` (Constant values).
  * `UK2Node_InputKeyEvent`, `UK2Node_InputAction`, `UK2Node_InputAxisEvent` (input mappings).
* **Graph Topology Relationships:**
  * `ue4:hasGraph` (relates `UBlueprint` or `UObject` to `UEdGraph`).
  * `ue4:hasNode` (relates `UEdGraph` to `UEdGraphNode`).
  * `ue4:hasPin` (relates `UEdGraphNode` to `UEdGraphPin`).
  * `ue4:linkedTo` (relates output pins to input pins).
  * Datatype attributes for pins: `pinName`, `pinDirection`, `pinCategory`, `pinSubCategory`, `defaultValue`, `bIsReference`, `bIsConst`.
  * Spatial attributes for nodes: `nodePosX`, `nodePosY`.

### C. Unification Layer (Interoperability)
To ensure the graph logic can reference the structural API, we establish object properties spanning both domains:
* `ue4:calledFunction` (associates a function call node with its reflected `UFunction` definition).
* `ue4:referencedProperty` (associates a variable get/set node with its reflected `UProperty` variable definition).
* `ue4:targetType` (associates a cast node with the target `UClass` it attempts to verify).
* `ue4:pinTypeObject` (associates a typed pin to its defining C++ `UField` class/struct/enum structure).

---

## 3. Validation and Compilation Proof

We verified the structural sanity of these extensions by writing them to local files and executing the target `ggen` validator.

1. **Test Compilation Command:**
   ```bash
   /Users/sac/.local/bin/ggen sync --validate-only true
   ```
2. **Results:**
   All validations passed successfully.
   * Manifest validation: `PASS`
   * custom SHACL validation rules (`ClassLabelShape`, `ClassCommentShape`, `NamespaceSanityShape`): `PASS`
   * Validation Rules `R1`, `R2`, `R3`, `R4`: `PASS`

---

## 4. Implementation Strategy

To transition this exploration into implementation, a **Worker Agent** should perform the following automated steps:

1. **File Overwrites:**
   Overwrite the existing ontology files in `/Users/sac/.ggen/packs/ue4_ontology/` using the verified proposed content files created in the workspace:
   * `/Users/sac/rocket-craft/.agents/explorer_reflection_blueprints_1/proposed_reflection.ttl` $\rightarrow$ `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl`
   * `/Users/sac/rocket-craft/.agents/explorer_reflection_blueprints_1/proposed_blueprints.ttl` $\rightarrow$ `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl`

2. **Validation Verification:**
   Run the project validation script to ensure the active directory reflects a successful state:
   ```bash
   /Users/sac/rocket-craft/validate_ontology.sh
   ```

3. **Ontology Generation Compilation:**
   Execute `ggen sync` to generate files (such as `README.md` or other targets) defined in `ggen.toml`:
   ```bash
   /Users/sac/.local/bin/ggen sync
   ```
