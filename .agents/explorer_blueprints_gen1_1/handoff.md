# Handoff Report — explorer_blueprints_gen1_1

## 1. Observation

Direct observations from the ontology and test files:
*   **File Path:** `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl`
    *   *Redundant connection properties:*
        ```turtle
        151: ue4:linkedTo a owl:ObjectProperty ;
        ...
        157: ue4:connectedTo a owl:ObjectProperty , owl:SymmetricProperty ;
        ```
    *   *Redundant function call properties:*
        ```turtle
        227: ue4:calledFunction a owl:ObjectProperty ;
        ...
        233: ue4:callsFunction a owl:ObjectProperty ;
        ```
*   **File Path:** `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`
    *   *SHACL Rule C (lines 125-140) only uses `callsFunction`:*
        ```turtle
        135:                 ?node ue4:callsFunction ?func .
        136:                 $this ue4:mapsToParameter ?param .
        ```
    *   *Exec connection check (Rule E, lines 164-179) only restricts `exec` categories:*
        ```turtle
        173:                 $this ue4:connectedTo ?other .
        174:                 $this ue4:pinCategory "exec" .
        175:                 ?other ue4:pinCategory ?cat .
        176:                 FILTER (?cat != "exec")
        ```
    *   *Pin Shape definition (lines 63-87) has no cardinality check for connections:*
        ```turtle
        64: ue4:UEdGraphPinShape
        65:     a sh:NodeShape ;
        66:     sh:targetClass ue4:UEdGraphPin ;
        ...
        ```
*   **File Path:** `/Users/sac/rocket-craft/ggen-validation-tests/gundam_character.ttl`
    *   *Mismatched categories connected together (lines 140-156):*
        ```turtle
        141: gundam:W_KeyPressedPinOut a ue4:UEdGraphPin ;
        ...
        146:     ue4:pinCategory "exec" ;
        147:     ue4:connectedTo gundam:MoveForwardPinIn .
        ...
        150: gundam:MoveForwardPinIn a ue4:UEdGraphPin ;
        ...
        155:     ue4:pinCategory "float" ;
        156:     ue4:connectedTo gundam:W_KeyPressedPinOut .
        ```

---

## 2. Logic Chain

1.  **Symmetric vs. Directed Redundancy (GAP_02):** In `blueprints.ttl`, `ue4:linkedTo` and `ue4:connectedTo` are both defined. However, `validation.shacl.ttl` only references `ue4:connectedTo` in all connection-related shapes (Rules A, B, E, H). Because there is no logical relationship between them (such as subproperty declarations), any graph generator using `ue4:linkedTo` will bypass these rules entirely.
2.  **Input Pin Connection Cardinality Loophole (GAP_03):** Blueprint compilation rules specify that an input pin cannot receive more than one connection. In `validation.shacl.ttl`, the `UEdGraphPinShape` node shape does not define a `sh:maxCount 1` limit for `ue4:connectedTo` on input pins. Thus, multi-connected inputs will pass the validation gate.
3.  **Data Pin Connection Type Safety Loophole (GAP_04):** Rule E in `validation.shacl.ttl` checks that execution pins (`exec` category) only connect to other execution pins. However, there are no checks for data pins, meaning a `boolean` pin connected to a `struct` or `object` pin passes validation.
4.  **C++ Alignment & Invalid Pin Directions (GAP_08):** In `blueprints.ttl`, `ue4:pinDirection` has the range `ue4:PinDirection`. `reflection.ttl` includes `ue4:InOut` and `ue4:Return` as valid individuals. Since Unreal Engine's C++ enum `EEdGraphPinDirection` is strictly binary (Input or Output), permitting `InOut` or `Return` on graph pins violates the engine's compilation model.
5.  **Function Call Property Redundancy (GAP_05):** `ue4:calledFunction` (domain: `UEdGraphNode`) and `ue4:callsFunction` (domain: `UK2Node`) are duplicate properties. Rule C of SHACL validation only queries `ue4:callsFunction`. If a generator defines a function call node using `ue4:calledFunction`, the parameter mapping checks are bypassed.
6.  **Pin to Parameter Type Verification Gap (GAP_06):** While Rule C checks that a pin's mapped parameter belongs to the called function, there is no validation to verify that the pin's data type matches the parameter's reflected data type (e.g., passing a string pin to a float parameter).

---

## 3. Caveats

*   **No Active Execution:** We performed no build or test commands (`ggen sync` or similar) in compliance with the read-only constraint.
*   **SHACL Engine Assumption:** We assume the validation engine enforces SHACL constraints and SPARQL ASK queries precisely as written.
*   **Coverage Boundaries:** We scoped the investigation strictly to the ontology packs and target test cases in `/Users/sac/.ggen/packs/ue4_ontology/` and `/Users/sac/rocket-craft/ggen-validation-tests/`.

---

## 4. Conclusion

The Blueprint execution graph mapping in `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl` has several critical gaps:
1.  **Redundant Properties:** Duplicate properties (`callsFunction`/`calledFunction` and `linkedTo`/`connectedTo`) allow generators to bypass SHACL validation constraints by using the unvalidated variants.
2.  **Topology & Type Safety Lapses:** Input pin cardinality (max 1 connection) and data pin type matching are not enforced by SHACL. Mismatched connections will pass GATE 0 validation and fail later during compilation.
3.  **Missing Node Subclasses:** Common nodes like `UK2Node_Knot` (reroutes) and `UK2Node_Select` are missing from the ontology vocabulary.

---

## 5. Verification Method

Verification of the findings can be performed manually:
1.  **Manual Inspection:** Open `blueprints.ttl` and verify the declarations of `linkedTo`, `connectedTo`, `callsFunction`, and `calledFunction`.
2.  **SHACL Trace:** Inspect `shacl/validation.shacl.ttl` to confirm:
    *   No occurrence of `ue4:linkedTo` in SHACL validation shapes.
    *   No connection cardinality shape (`sh:maxCount 1` for inputs) exists.
    *   No parameter type comparison rules are defined in the SPARQL queries.
3.  **Anomalous RDF Instance Verification:** A future implementing agent can write an RDF file with a type mismatch or multiple connections to an input pin and confirm that running `/Users/sac/.local/bin/ggen sync --validate-only true` reports success instead of a validation failure.
