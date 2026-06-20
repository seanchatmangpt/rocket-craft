# UE4 Reflection and Blueprint Ontologies Analysis Report

## Executive Summary
This report analyzes the structural and semantic constraints of Unreal Engine 4 (UE4) Reflection and Blueprint ontologies, proposing a comprehensive RDF schema and SHACL validation layer to map parameter passing for `UFunction` and pins (`UEdGraphPin`). By extending the existing models, we ensure complete type safety, execution flow integrity, and graph isolation, fully compliant with the project's 4-Tier Acceptance Testing Framework.

---

## 1. Current State & Directory Analysis
The target workspace contains the following core files in `/Users/sac/.ggen/packs/ue4_ontology`:
*   `core.ttl`: Contains the C++ backbone hierarchy (`UObject` -> `AActor` -> `APawn` -> `ACharacter`).
*   `reflection.ttl`: Declares C++ reflection metadata classes (`UField` -> `UStruct` -> `UClass`/`UFunction` and `UProperty`).
*   `blueprints.ttl`: Defines editor graph structures (`UEdGraph`, `UEdGraphNode`, and `UK2Node`).
*   `shacl/validation.shacl.ttl`: Enforces basic naming conventions (labels, descriptions, namespace sanity).
*   `ggen.toml`: Configures the compiler and runs 4 SPARQL-based validation rules (`R1`–`R4`).

Running `/Users/sac/rocket-craft/validate_ontology.sh` completes successfully, verifying that all rules currently pass. However, there is currently no formal representation of function parameters, connection pins, pin directions, or mapping between blueprint node pins and reflected C++ signatures.

---

## 2. Proposed RDF Ontological Mapping for Parameters and Pins

To map parameter passing and graph actuation, we introduce classes for parameters and graph pins, define direction vectors, and establish connection properties.

### 2.1 Schema Class Definitions
1.  **`ue4:UFunctionParameter`**: A subclass of `ue4:UProperty` that represents an individual parameter within a `UFunction` signature.
2.  **`ue4:UEdGraphPin`**: A subclass of `ue4:UObject` representing connection pins on graph nodes.
3.  **`ue4:PinDirection`**: An enumeration class defining direction.

### 2.2 Property Definitions and Cross-Domain Mappings
*   `ue4:hasParameter` / `ue4:parameterOf`: Relates `ue4:UFunction` to `ue4:UFunctionParameter`.
*   `ue4:parameterDirection`: Relates a parameter to a `ue4:PinDirection` (with individuals: `ue4:Input`, `ue4:Output`, `ue4:InOut`, and `ue4:Return`).
*   `ue4:parameterIndex`: Represents the 0-based index of the parameter in the signature.
*   `ue4:hasPin` / `ue4:pinOf`: Relates `ue4:UEdGraphNode` to `ue4:UEdGraphPin`.
*   `ue4:pinDirection`: Relates a pin to a `ue4:PinDirection` (`ue4:Input` or `ue4:Output`).
*   `ue4:pinCategory`: Indicates the data type category of a pin (e.g., `"exec"`, `"bool"`, `"int"`, `"float"`, `"object"`).
*   `ue4:pinSubCategoryObject`: Relates a pin to a specific C++ class or struct type (e.g., linking an object pin to a `ue4:UClass`).
*   `ue4:connectedTo`: A symmetric object property representing a bidirectional link between pins.
*   `ue4:callsFunction`: Relates a `ue4:UK2Node` (representing a call function node) to the called `ue4:UFunction`.
*   `ue4:mapsToParameter`: Maps a pin on a function call node to the corresponding parameter of the called function.

### 2.3 Proposed Schema Additions (Turtle Patch)

To implement these properties, the following blocks should be appended to `reflection.ttl` and `blueprints.ttl` respectively:

#### Append to `reflection.ttl`:
```turtle
# =========================================================================
# Function Parameters & Directions
# =========================================================================

ue4:UFunctionParameter a owl:Class ;
    rdfs:subClassOf ue4:UProperty ;
    rdfs:label "UFunctionParameter" ;
    rdfs:comment "Represents an individual parameter in a UFunction signature." .

ue4:PinDirection a owl:Class ;
    rdfs:label "PinDirection" ;
    rdfs:comment "Specifies the passing direction of a parameter or graph pin." .

ue4:Input a ue4:PinDirection ;
    rdfs:label "Input" ;
    rdfs:comment "Represents input direction." .

ue4:Output a ue4:PinDirection ;
    rdfs:label "Output" ;
    rdfs:comment "Represents output direction." .

ue4:InOut a ue4:PinDirection ;
    rdfs:label "InOut" ;
    rdfs:comment "Represents a bidirectional parameter passed by reference." .

ue4:Return a ue4:PinDirection ;
    rdfs:label "Return" ;
    rdfs:comment "Represents the return value of a function." .

ue4:hasParameter a owl:ObjectProperty ;
    rdfs:label "hasParameter" ;
    rdfs:comment "Relates a UFunction to one of its parameters." ;
    rdfs:domain ue4:UFunction ;
    rdfs:range ue4:UFunctionParameter .

ue4:parameterOf a owl:ObjectProperty ;
    rdfs:label "parameterOf" ;
    rdfs:comment "Relates a parameter to its parent UFunction." ;
    rdfs:domain ue4:UFunctionParameter ;
    rdfs:range ue4:UFunction ;
    owl:inverseOf ue4:hasParameter .

ue4:parameterDirection a owl:ObjectProperty ;
    rdfs:label "parameterDirection" ;
    rdfs:comment "Defines the direction of the function parameter." ;
    rdfs:domain ue4:UFunctionParameter ;
    rdfs:range ue4:PinDirection .

ue4:parameterIndex a owl:DatatypeProperty ;
    rdfs:label "parameterIndex" ;
    rdfs:comment "The 0-based sequence index of the parameter in the function signature." ;
    rdfs:domain ue4:UFunctionParameter ;
    rdfs:range xsd:integer .
```

#### Append to `blueprints.ttl`:
```turtle
# =========================================================================
# Graph Pins & Actuation Mapping
# =========================================================================

ue4:UEdGraphPin a owl:Class ;
    rdfs:subClassOf ue4:UObject ;
    rdfs:label "UEdGraphPin" ;
    rdfs:comment "Represents a connection pin on a UEdGraphNode." .

ue4:hasPin a owl:ObjectProperty ;
    rdfs:label "hasPin" ;
    rdfs:comment "Relates an editor graph node to one of its connection pins." ;
    rdfs:domain ue4:UEdGraphNode ;
    rdfs:range ue4:UEdGraphPin .

ue4:pinOf a owl:ObjectProperty ;
    rdfs:label "pinOf" ;
    rdfs:comment "Relates a connection pin to its parent node." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range ue4:UEdGraphNode ;
    owl:inverseOf ue4:hasPin .

ue4:pinDirection a owl:ObjectProperty ;
    rdfs:label "pinDirection" ;
    rdfs:comment "Specifies if a pin is Input or Output." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range ue4:PinDirection .

ue4:connectedTo a owl:ObjectProperty , owl:SymmetricProperty ;
    rdfs:label "connectedTo" ;
    rdfs:comment "Represents a connection between two pins in a graph." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range ue4:UEdGraphPin .

ue4:pinCategory a owl:DatatypeProperty ;
    rdfs:label "pinCategory" ;
    rdfs:comment "The general category of data for the pin (e.g. exec, bool, int, float, object)." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range xsd:string .

ue4:pinSubCategoryObject a owl:ObjectProperty ;
    rdfs:label "pinSubCategoryObject" ;
    rdfs:comment "For object or struct pins, references the specific class or struct type." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range ue4:UStruct .

ue4:callsFunction a owl:ObjectProperty ;
    rdfs:label "callsFunction" ;
    rdfs:comment "Links a function call node (UK2Node) to the reflected UFunction signature." ;
    rdfs:domain ue4:UK2Node ;
    rdfs:range ue4:UFunction .

ue4:mapsToParameter a owl:ObjectProperty ;
    rdfs:label "mapsToParameter" ;
    rdfs:comment "Maps an individual node pin to its target function parameter." ;
    rdfs:domain ue4:UEdGraphPin ;
    rdfs:range ue4:UFunctionParameter .

ue4:hasNode a owl:ObjectProperty ;
    rdfs:label "hasNode" ;
    rdfs:comment "Relates a UEdGraph to a node it contains." ;
    rdfs:domain ue4:UEdGraph ;
    rdfs:range ue4:UEdGraphNode .

ue4:nodeOf a owl:ObjectProperty ;
    rdfs:label "nodeOf" ;
    rdfs:comment "Relates a node to the UEdGraph containing it." ;
    rdfs:domain ue4:UEdGraphNode ;
    rdfs:range ue4:UEdGraph ;
    owl:inverseOf ue4:hasNode .
```

---

## 3. Proposed SHACL Shape Validation Rules

To prevent ontological defects (which compile but violate runtime invariants), we design SHACL shapes that validate both local nodes and cross-node connections.

### 3.1 Node Constraints (Cardinality & Types)
These enforce that parameter and pin records are structurally complete and correctly formatted.

```turtle
# Enforce UFunctionParameter properties
ue4:UFunctionParameterShape
    a sh:NodeShape ;
    sh:targetClass ue4:UFunctionParameter ;
    sh:property [
        sh:path ue4:parameterOf ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:UFunction ;
        sh:message "A function parameter must belong to exactly one UFunction." ;
    ] ;
    sh:property [
        sh:path ue4:parameterDirection ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:in ( ue4:Input ue4:Output ue4:InOut ue4:Return ) ;
        sh:message "A parameter must have exactly one direction (Input, Output, InOut, or Return)." ;
    ] ;
    sh:property [
        sh:path ue4:parameterIndex ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
        sh:message "A parameter must have a single non-negative integer parameterIndex." ;
    ] .

# Enforce UEdGraphPin properties
ue4:UEdGraphPinShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:property [
        sh:path ue4:pinOf ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:UEdGraphNode ;
        sh:message "A pin must belong to exactly one UEdGraphNode." ;
    ] ;
    sh:property [
        sh:path ue4:pinDirection ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:in ( ue4:Input ue4:Output ) ;
        sh:message "A pin must have exactly one direction (Input or Output)." ;
    ] ;
    sh:property [
        sh:path ue4:pinCategory ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:message "A pin must have a single data category string." ;
    ] .
```

### 3.2 Actuation & Connection Constraints (SPARQL-based SHACL Shapes)
These enforce logic and flow constraints that involve navigation of the RDF graph.

#### Rule A: Pin Connection Direction Check
Pins cannot connect to pins of the same direction (e.g. Input to Input, Output to Output).
```turtle
ue4:PinConnectionDirectionShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:sparql [
        sh:message "Pin connection direction mismatch: A pin cannot be connected to another pin of the same direction." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?other
            WHERE {
                $this ue4:connectedTo ?other .
                $this ue4:pinDirection ?dir .
                ?other ue4:pinDirection ?dir .
            }
        """ ;
    ] .
```

#### Rule B: Graph Isolation Check
Pins can only be connected to other pins if they reside within the same parent graph (`ue4:UEdGraph`).
```turtle
ue4:PinConnectionGraphShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:sparql [
        sh:message "Graph isolation violation: Connected pins must belong to nodes within the same UEdGraph." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?other
            WHERE {
                $this ue4:connectedTo ?other .
                $this ue4:pinOf/ue4:nodeOf ?graph1 .
                ?other ue4:pinOf/ue4:nodeOf ?graph2 .
                FILTER (?graph1 != ?graph2)
            }
        """ ;
    ] .
```

#### Rule C: Function Call Parameter Mapping Target Integrity
For a function call node (`UK2Node`), its pins must map to parameters that actually belong to the called function.
```turtle
ue4:FunctionCallPinMappingShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:sparql [
        sh:message "Pin mapping mismatch: Pin maps to a parameter that does not belong to the node's target called function." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?param ?func ?node
            WHERE {
                $this ue4:pinOf ?node .
                ?node ue4:callsFunction ?func .
                $this ue4:mapsToParameter ?param .
                FILTER NOT EXISTS { ?func ue4:hasParameter ?param }
            }
        """ ;
    ] .
```

#### Rule D: Pin Parameter Direction Match
Input pins on a call node must map only to Input or InOut parameters, while Output pins must map to Output, InOut, or Return parameters.
```turtle
ue4:PinParameterDirectionMatchShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:sparql [
        sh:message "Pin and parameter direction mismatch: Input pins must map to Input/InOut parameters; Output pins must map to Output/InOut/Return parameters." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?param ?pinDir ?paramDir
            WHERE {
                $this ue4:mapsToParameter ?param .
                $this ue4:pinDirection ?pinDir .
                ?param ue4:parameterDirection ?paramDir .
                FILTER (
                    ( ?pinDir = ue4:Input && ?paramDir != ue4:Input && ?paramDir != ue4:InOut ) ||
                    ( ?pinDir = ue4:Output && ?paramDir != ue4:Output && ?paramDir != ue4:InOut && ?paramDir != ue4:Return )
                )
            }
        """ ;
    ] .
```

#### Rule E: Exec vs. Data Pin Separation
An execution pin (`pinCategory "exec"`) can only connect to another execution pin.
```turtle
ue4:ExecPinConnectionShape
    a sh:NodeShape ;
    sh:targetClass ue4:UEdGraphPin ;
    sh:sparql [
        sh:message "Execution pin mismatch: An execution pin ('exec') can only connect to another execution pin." ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            SELECT $this ?other
            WHERE {
                $this ue4:connectedTo ?other .
                $this ue4:pinCategory "exec" .
                ?other ue4:pinCategory ?cat .
                FILTER (?cat != "exec")
            }
        """ ;
    ] .
```

---

## 4. 4-Tier Acceptance Testing Integration & Compliance

To integrate these properties and rules, we propose updating `TEST_INFRA.md` and `TEST_READY.md` to incorporate the parameter passing and connection checks.

### Tier 1: Feature Coverage (Feature 3: Reflection & Blueprints)
Add cases to verify core definitions are parsed by `ggen` compiler:
*   **Case 3.7: Parameter Base Class (`ue4:UFunctionParameter`)**: Verify `UFunctionParameter` exists and is a subclass of `ue4:UProperty`.
*   **Case 3.8: Pin Class (`ue4:UEdGraphPin`)**: Verify `UEdGraphPin` exists and is a subclass of `ue4:UObject`.
*   **Case 3.9: Association Properties**: Verify properties `ue4:hasParameter`, `ue4:hasPin`, `ue4:parameterDirection`, `ue4:pinDirection`, `ue4:connectedTo` are declared.

### Tier 2: Boundary & Corner Cases (Boundary Checks)
Add validation checks for invalid graphs:
*   **Case 2.6: Direction Violation**: Inject a test instance with two connected input pins. The pipeline must reject it under SHACL `ue4:PinConnectionDirectionShape`.
*   **Case 2.7: Graph Separation Violation**: Inject pins from different graphs connected directly. The pipeline must reject it under SHACL `ue4:PinConnectionGraphShape`.
*   **Case 2.8: Call Parameter Mismatch**: Inject a pin on a call node mapping to a parameter of a different function. The pipeline must reject it under `ue4:FunctionCallPinMappingShape`.

### Tier 3: Cross-Feature Combinations
Add checks for compilation integrity:
*   **Case 3.4: Function Signature Generation Validation**: Verify that the generated C++ header file signature (`.h`) matches the compiled RDF graph parameters (`ue4:UFunctionParameter` ordered by `ue4:parameterIndex`).
*   **Case 3.5: Pin Type Compatibility**: Ensure data pins connected via `ue4:connectedTo` share compatible categories (e.g., float pins connect only to float pins, or object pins represent compatible subclasses).

### Tier 4: Real-World Application Scenarios (Gundam Weapon Fire Loop)
Define a test scenario to prove execution:
*   **Case 4.2: Gundam Weapon Actuation Loop**:
    *   Define a C++ class `AGundamCharacter` with a function `FireWeapon(int32 WeaponIndex, float DamageMultiplier)` exposed via `UFunction`.
    *   Declare two parameters: index 0 (`ue4:Input`, int) and index 1 (`ue4:Input`, float).
    *   Model a Blueprint graph `EventGraph` on a Gundam character blueprint containing an event node and a function call node connected via execution flow.
    *   Verify that `callsFunction` and `mapsToParameter` link the blueprint node pins correctly to the C++ reflection parameters.
    *   Run validation to prove the graph contains no dangling execution flows or type mismatches.

---

## 5. Implementation Strategy and Verification

### 5.1 Step-by-Step Deployment
1.  **Schema Update**: Append the Turtle snippets in Section 2 to `/Users/sac/.ggen/packs/ue4_ontology/reflection.ttl` and `/Users/sac/.ggen/packs/ue4_ontology/blueprints.ttl`.
2.  **SHACL Addition**: Write the validation shapes in Section 3 to a new shapes file or append to `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl`.
3.  **Harness Update**: Add the path to any new SHACL shapes in the `[validation]` block of `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.
4.  **Execute Validation**: Run `/Users/sac/rocket-craft/validate_ontology.sh` to compile and execute the updated SHACL validation engine.
5.  **Audit Logs**: Check `deploy.log` and target receipts in `.ggen/receipts/latest.json` for validation reports and cryptographic BLAKE3 receipts.
