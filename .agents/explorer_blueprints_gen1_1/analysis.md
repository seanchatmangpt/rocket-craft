# Blueprint Ontology Coverage & Correctness Analysis — explorer_blueprints_gen1_1

## 1. Executive Summary

This report presents a structural analysis of `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl` and its integration with `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`. 

The core Blueprint execution graph representation (`UEdGraph`, `UEdGraphNode`, `UEdGraphPin`, `UK2Node` and standard subclasses) is mostly covered. However, several critical structural gaps, semantic redundancies, type safety deficiencies, and deviations from the Unreal Engine C++ source code model were identified. These issues present a risk to **GATE 0 (Source Admission)** and subsequent compilation gates by allowing invalid graph topologies to pass validation.

---

## 2. Coverage Analysis of Blueprint Execution Graph Nodes

### 2.1 Present UK2Node Subclasses
The ontology defines the following subclasses of `ue4:UK2Node`:
*   `ue4:UK2Node_Event` & `ue4:UK2Node_CustomEvent`
*   `ue4:UK2Node_CallFunction` & `ue4:UK2Node_CommutativeAssociativeBinaryOperator`
*   `ue4:UK2Node_Variable`, `ue4:UK2Node_VariableGet`, & `ue4:UK2Node_VariableSet`
*   `ue4:UK2Node_ExecutionSequence`
*   `ue4:UK2Node_IfThenElse` (Branch node)
*   `ue4:UK2Node_DynamicCast`
*   `ue4:UK2Node_Literal`
*   `ue4:UK2Node_InputKeyEvent`, `ue4:UK2Node_InputAction`, & `ue4:UK2Node_InputAxisEvent`

### 2.2 Omitted Subclasses (Structural Gaps)
The ontology completely omits the following C++ `UK2Node` subclasses that are standard for game logic execution:
1.  **`UK2Node_Knot` (Reroute Nodes):** Standard node for layout organization and routing pin connections.
2.  **`UK2Node_Select`:** Conditional evaluation node (`Select` in Blueprints).
3.  **`UK2Node_MacroInstance`:** Necessary to model macro execution flow.
4.  **`UK2Node_Composite`:** For collapsed nodes / subgraphs.
5.  **`UK2Node_Timeline`:** Controls animations/curves over time.
6.  **`UK2Node_SpawnActorFromClass`:** Standard mechanism for spawning actors dynamically.
7.  **`UK2Node_ConstructObjectFromClass`:** Standard mechanism for instantiating raw `UObject` subclasses.
8.  **`UK2Node_FunctionEntry` & `UK2Node_FunctionResult`:** Necessary to declare input/output bounds of Blueprint-defined functions.
9.  **`UK2Node_ComponentBoundEvent`:** Events bound to specific actor component delegate calls.
10. **Delegate Bindings:** `UK2Node_AddDelegate`, `UK2Node_RemoveDelegate`, `UK2Node_ClearDelegate`, and `UK2Node_AssignDelegate`.

---

## 3. Node Graph Topology and Pin Connectivity Verification

### 3.1 Pin Properties and C++ Naming Mismatches
*   **Duplicate Reference Properties (`pinTypeObject` vs `pinSubCategoryObject`):**
    *   `ue4:pinTypeObject` (domain: `ue4:UEdGraphPin`, range: `ue4:UField`)
    *   `ue4:pinSubCategoryObject` (domain: `ue4:UEdGraphPin`, range: `ue4:UStruct`)
    *   *C++ Discrepancy:* In Unreal C++, `FEdGraphPinType` uses `PinSubCategoryObject` to point to the `UObject` representing the type (Class, Struct, or Enum). There is no `PinTypeObject` in the engine.
*   **Invalid Pin Direction Range:**
    *   The ontology defines `ue4:pinDirection` range as `ue4:PinDirection`.
    *   `PinDirection` contains `ue4:Input`, `ue4:Output`, `ue4:InOut`, and `ue4:Return`.
    *   *C++ Discrepancy:* In Unreal C++, `EEdGraphPinDirection` is strictly binary (`EGPD_Input` and `EGPD_Output`). Assigning `InOut` or `Return` as a pin direction is syntactically allowed by OWL but violates the engine C++ model.

### 3.2 Verification of Pin Connectivity
*   **`ue4:linkedTo` vs `ue4:connectedTo` Disconnect:**
    *   `ue4:linkedTo` is defined as a directed object property from pin to pin.
    *   `ue4:connectedTo` is defined as a symmetric property.
    *   *Gap:* In `validation.shacl.ttl` and `ggen.toml`, SHACL and SPARQL validation rules only check `connectedTo`. Because `linkedTo` has no logical connection (e.g. subproperty relationship) to `connectedTo`, a generator using `linkedTo` will bypass all SHACL checks.

### 3.3 Pin Connection Gaps in SHACL Validation
1.  **Pin Cardinality (Max Count):**
    *   In Unreal Engine, an `Input` pin can only have **at most one** incoming connection. An `Output` pin can connect to multiple inputs.
    *   *Gap:* Neither the ontology nor SHACL limits the connection count of Input pins. A shape must enforce `sh:maxCount 1` on `ue4:connectedTo` for pins with `ue4:pinDirection ue4:Input`.
2.  **Lack of Data Type Compatibility Check:**
    *   SHACL validates exec pin connections (Rule E), but does not validate data pin connections.
    *   *Gap:* There are no rules preventing a `boolean` pin from connecting to an `object` or `struct` pin.
3.  **Untyped Category Vocabulary:**
    *   `ue4:pinCategory` is typed as a raw `xsd:string`.
    *   *Gap:* Typos (e.g., `"flot"`) are permitted. It should be constrained to a closed set of categories (`exec`, `bool`, `byte`, `int`, `float`, `double`, `name`, `string`, `text`, `object`, `class`, `struct`, `interface`, `delegate`).

---

## 4. Node Reference to Reflection Metadata

The connection between Graph Nodes and Reflection classes (`UFunction`, `UProperty`, `UFunctionParameter`) was analyzed.

### 4.1 Property Duplication (`callsFunction` vs `calledFunction`)
*   `ue4:calledFunction` (domain: `ue4:UEdGraphNode`, range: `ue4:UFunction`)
*   `ue4:callsFunction` (domain: `ue4:UK2Node`, range: `ue4:UFunction`)
*   *Gap:* Both properties perform the same function but on different domains. `validation.shacl.ttl` Rule C (Function Call Parameter Mapping) uses `ue4:callsFunction` to retrieve the called function. If a generator uses `ue4:calledFunction`, the parameter mapping checks will be bypassed.

### 4.2 Pin to Parameter Mapping Gaps
1.  **Lack of Parameter Type Validation:**
    *   `ue4:mapsToParameter` links a pin to a `UFunctionParameter`.
    *   *Gap:* There is no SHACL shape validating that the data type of the pin (`pinCategory`/`pinSubCategoryObject`) is compatible with the type of the parameter (`propertyType` in `reflection.ttl`).
2.  **Lack of Parameter Mapping Completeness:**
    *   There are no constraints enforcing that a function call node must map pins to all non-optional parameters of the called function.
3.  **Missing Variable Node Integrity Checks:**
    *   `ue4:referencedProperty` links a variable node (`UK2Node_Variable`) to a `UProperty`.
    *   *Gap:* There are no SHACL shapes verifying that a `UK2Node_VariableGet` or `UK2Node_VariableSet` node has exactly one `ue4:referencedProperty`, nor that the node's data pin type matches the referenced property type.

---

## 5. Structural Gap Summary Matrix

| Gap ID | Area | Verbatim Code Location / Context | Description / Impact | Proposed Remedy |
|---|---|---|---|---|
| **GAP_01** | Node Vocabulary | `blueprints.ttl` Line 41 | Omission of standard execution nodes (e.g. `UK2Node_Knot`, `UK2Node_Select`, `UK2Node_SpawnActorFromClass`). | Cannot represent complete execution graphs (e.g., reroute nodes, spawners). | Define missing subclasses inheriting from `ue4:UK2Node`. |
| **GAP_02** | Pin Topology | `blueprints.ttl` Line 151 & 157 | Redundant properties `linkedTo` and `connectedTo`. Only `connectedTo` is validated. | Connections represented with `linkedTo` bypass SHACL validation. | Define `connectedTo` as a symmetric super-property or equivalence of `linkedTo`. |
| **GAP_03** | Pin Topology | `validation.shacl.ttl` Line 64 | No constraint on Input pin connection counts. | Multiple outputs can connect to one input, causing engine compiler errors. | Add `sh:property [ sh:path ue4:connectedTo ; sh:maxCount 1 ]` specifically for Input pins. |
| **GAP_04** | Type Safety | `validation.shacl.ttl` Line 64 | Mismatched data pins (e.g. `bool` to `struct`) pass validation. | Lack of type safety at GATE 0 allows invalid blueprints to proceed to compile gates. | Add SHACL shapes mapping pin categories to their compatible categories. |
| **GAP_05** | Reflection | `blueprints.ttl` Line 227 & 233 | Redundant properties `callsFunction` and `calledFunction`. | Generator using `calledFunction` bypasses SHACL parameter validation rules. | Consolidate into a single property or declare subproperty relationships. |
| **GAP_06** | Reflection | `validation.shacl.ttl` Line 125 | Missing pin-to-parameter type matching. | A pin can map to a function parameter of a different type (e.g. float to string). | Add SHACL sparql check enforcing type equivalence between pin category/type and parameter type. |
| **GAP_07** | Variable Nodes | `validation.shacl.ttl` Line 64 | No validation shapes exist for variable getter/setter nodes. | Getter/setter nodes can be created without referencing a property. | Add shapes for `UK2Node_VariableGet` and `UK2Node_VariableSet` enforcing property references. |
| **GAP_08** | C++ Alignment | `blueprints.ttl` Line 169 & `reflection.ttl` Line 244 | Pin direction includes `InOut` and `Return`. | Violates Unreal's binary direction model, risking translation errors. | Constrain `ue4:pinDirection` of `ue4:UEdGraphPin` to only `ue4:Input` and `ue4:Output`. |
