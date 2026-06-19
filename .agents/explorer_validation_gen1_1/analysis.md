# UE4 Ontology & Graph Validation Analysis

## Executive Summary
This analysis evaluates the 10 custom validation rules (RuleA-H, RuleLabel, RuleNamespace) and their corresponding SHACL shapes in `validation.shacl.ttl` against the SPARQL rules in `ggen.toml`. While the core validation framework is structured correctly, several logical flaws, SHACL-SPARQL discrepancies, and missing checks (such as lack of symmetry, subclass target omissions, blank node failures, missing node-to-graph requirements, and lack of cyclic data-flow validation) render the current validation system vulnerable to admitting invalid configurations.

---

## 1. Individual Rule Analysis

### RuleA: Pin Connection Direction Check
* **SHACL Shape**: `ue4:PinConnectionDirectionShape`
* **SPARQL Rule**: `RuleA`
* **Objective**: A pin cannot be connected to another pin of the same direction.
* **Findings**:
  * **Evaluation**: The logic checks that if `?pin1 ue4:connectedTo ?pin2`, their `ue4:pinDirection` values cannot be equal.
  * **Flaw/Edge Case**: The rule depends on the presence of `ue4:pinDirection` on both pins. If a pin is missing its direction, the rule silently passes without flagging the connection (though SHACL checks presence via a separate shape).
  * **Flaw/Edge Case**: It does not verify that pin directions are restricted to only `ue4:Input` or `ue4:Output`. In Unreal, pins can only be input/output (parameters can be in/out/inout/return). If a pin is incorrectly assigned `ue4:Return` or `ue4:InOut`, RuleA might not flag it unless it connects to another pin of the same direction.

### RuleB: Graph Isolation Check
* **SHACL Shape**: `ue4:PinConnectionGraphShape`
* **SPARQL Rule**: `RuleB`
* **Objective**: Connected pins must belong to nodes within the same `UEdGraph`.
* **Findings**:
  * **Critical Flaw (Missing Node-to-Graph Constraints)**: There is no SHACL shape enforcing that a `ue4:UEdGraphNode` must belong to a graph (i.e. having exactly one `ue4:nodeOf` relation). If a node is completely detached from any graph, its `ue4:nodeOf` is missing. Thus, the path `?pin ue4:pinOf/ue4:nodeOf ?graph` fails to bind a graph, and RuleB's `FILTER (?graph1 != ?graph2)` evaluates to empty, meaning connections to/from this detached node are **silently ignored and pass**.
  * **Critical Flaw (Multi-Graph Assignment)**: If a node is accidentally assigned to multiple graphs (e.g. `node1 ue4:nodeOf graph1`, `node1 ue4:nodeOf graph2`), valid connections within the same graph will be incorrectly flagged as violations (false positives).

### RuleC: Function Call Parameter Mapping Target Integrity
* **SHACL Shape**: `ue4:FunctionCallPinMappingShape`
* **SPARQL Rule**: `RuleC`
* **Objective**: Pin maps to a parameter that does not belong to the node's target called function.
* **Findings**:
  * **Critical Flaw (Reasoning Dependency & False Positives)**: In the SPARQL rule, the query checks `FILTER NOT EXISTS { ?func ue4:hasParameter ?param }`. However, in the instance data and SHACL shapes, parameter relationships are represented primarily via `ue4:parameterOf` pointing from the parameter to the function. Although `reflection.ttl` declares `ue4:parameterOf owl:inverseOf ue4:hasParameter`, a simple SPARQL engine (such as the one in `ggen`) operates without OWL entailment. If instance data only defines `ue4:parameterOf` and not `ue4:hasParameter`, RuleC will **falsely flag every correct parameter mapping as a violation** (100% false-positive rate).
  * **Flaw**: If `ue4:callsFunction` is missing from the node, the rule does not execute its body, resulting in a silent pass. No SHACL constraint forces a function call node to have `ue4:callsFunction`.

### RuleD: Pin Parameter Direction Match
* **SHACL Shape**: `ue4:PinParameterDirectionMatchShape`
* **SPARQL Rule**: `RuleD`
* **Objective**: Input pins must map to Input/InOut parameters; Output pins must map to Output/InOut/Return parameters.
* **Findings**:
  * **Evaluation**: The logic correctly identifies invalid mappings (e.g. input pin mapping to an output parameter).
  * **Edge Case**: If a pin is given an invalid direction like `ue4:Return` (which is only valid for parameters), RuleD will not catch it because the filter is scoped only to `?pinDir = ue4:Input` and `?pinDir = ue4:Output`.

### RuleE: Exec vs. Data Pin Separation
* **SHACL Shape**: `ue4:ExecPinConnectionShape`
* **SPARQL Rule**: `RuleE`
* **Objective**: An execution pin ('exec') can only connect to another execution pin.
* **Findings**:
  * **Critical Flaw (Lack of Symmetry / Silent Pass)**: The property `ue4:connectedTo` is declared as `owl:SymmetricProperty`. However, standard SPARQL/SHACL engines do not perform symmetry reasoning. The queries check:
    `?pin1 ue4:connectedTo ?pin2 . ?pin1 ue4:pinCategory "exec" . ?pin2 ue4:pinCategory ?cat . FILTER (?cat != "exec")`
    If a connection is written unidirectionally in the RDF store with the non-exec pin as the subject (e.g. `pin_float ue4:connectedTo pin_exec`), `?pin1 ue4:pinCategory "exec"` fails to match. The violation is **completely missed and passes**.
  * **Flaw (Type Compatibility Lack)**: The rule only protects execution pins. It does not verify that different types of data pins (e.g. `float` and `string`, or mismatched object class types) are prevented from connecting.

### RuleF: Character Cooking State Constraint
* **SHACL Shape**: `ue4:CharacterCookingStateShape`
* **SPARQL Rule**: `RuleF`
* **Objective**: A character must have exactly one cooking state of type `CookingTypestate`.
* **Findings**:
  * **Critical Flaw (SHACL Subclass Target Omission)**: The SHACL shape targets `ue4:ACharacter` via `sh:targetClass`. Standard SHACL does not perform RDFS/OWL subclass inheritance for target classes. A custom subclass like `gundam:AGundamCharacter` will **never be validated by the SHACL shape** unless it is explicitly typed with `ue4:ACharacter`.
  * **Critical Flaw (SPARQL Multiplicity Failure)**: The SPARQL query in `ggen.toml` checks if a character has *at least one* cooking state (using `FILTER NOT EXISTS` containing another `FILTER NOT EXISTS`). If a character is assigned **multiple** cooking states (e.g. both `gundam:Cooked` and `gundam:Uncooked`), the SPARQL query still finds one, evaluates to valid, and **fails to flag the multiple-state defect**.

### RuleG: World Packaging State Constraint
* **SHACL Shape**: `ue4:WorldPackagingStateShape`
* **SPARQL Rule**: `RuleG`
* **Objective**: A world must have exactly one packaging state of type `WasmPackagingTypestate`.
* **Findings**:
  * **Critical Flaw (SHACL Subclass Target Omission)**: Same as RuleF; custom subclasses of `ue4:UWorld` will bypass the SHACL validation entirely.
  * **Critical Flaw (SPARQL Multiplicity Failure)**: Same as RuleF; assigning multiple packaging states to a single world will not be caught by the SPARQL validation.

### RuleH: Input Exec Pin Connected Constraint
* **SHACL Shape**: `ue4:InputExecPinConnectedShape`
* **SPARQL Rule**: `RuleH`
* **Objective**: Input execution pins on function call nodes must be connected to an execution pin.
* **Findings**:
  * **Flaw (Subclass Targeting)**: The SPARQL rule uses `?node a ue4:UK2Node_CallFunction`. This does not match subclasses of `ue4:UK2Node_CallFunction` (e.g. custom function call nodes) unless they are explicitly typed as the parent, due to lack of subClassOf matching on `a` (rdf:type) in standard SPARQL.
  * **Flaw (Narrow Scope)**: The rule only protects function call nodes (`ue4:UK2Node_CallFunction`). Other control flow nodes that require execution input (such as `ue4:UK2Node_IfThenElse`, `ue4:UK2Node_ExecutionSequence`, or loops) will not be checked, allowing dangling input execution paths on those nodes to pass.

### RuleLabel: Public Class Label Check
* **SHACL Shape**: `ue4:ClassLabelShape`
* **SPARQL Rule**: `RuleLabel`
* **Objective**: All public classes must have at least one `rdfs:label`.
* **Findings**:
  * **Flaw (Blank Node Failure)**: The rules match any instance of `rdfs:Class` or `owl:Class`. In complex OWL ontologies, blank nodes are frequently used to express class restrictions (e.g., `[ a owl:Class ; owl:unionOf (...) ]`). Because blank nodes cannot have an `rdfs:label`, the rule will **erroneously flag all blank node classes as violations**.
  * **Discrepancy (Missing Warning Rules)**: The SHACL file includes `ue4:ClassCommentShape` as a Warning shape for `rdfs:comment` presence. This Warning shape is completely missing from `ggen.toml`'s validation rules, meaning warnings are not checked during SPARQL validations.

### RuleNamespace: Namespace Sanity Check
* **SHACL Shape**: `ue4:NamespaceSanityShape`
* **SPARQL Rule**: `RuleNamespace`
* **Objective**: Subjects must use public HTTP/HTTPS IRIs.
* **Findings**:
  * **Critical Mismatch (Weak SPARQL Check)**: The SHACL shape enforces that subjects must start with `http://` or `https://` via `sh:pattern "^https?://"`. However, the SPARQL rule in `ggen.toml` only checks if the subject starts with `urn:` (via `strstarts(str(?subj), "urn:")`). Any other private scheme (e.g. `uuid:`, `mailto:`, or custom opaque URIs) will bypass the SPARQL check while failing the SHACL shape, causing inconsistent results.

---

## 2. Key Discrepancies and Omissions

| Rule / Concept | SHACL Behavior | SPARQL (ggen.toml) Behavior | Severity / Impact |
| --- | --- | --- | --- |
| **Symmetry (`ue4:connectedTo`)** | Fails to detect mismatch if exec pin is object and connection is unidirectional. | Fails to detect mismatch if exec pin is object and connection is unidirectional. | **High** (Allows invalid graph wiring to bypass validation). |
| **Subclass Targeting** | Omitted (does not target subclasses of `ACharacter` or `UWorld`). | Supported (explicitly checks `rdfs:subClassOf*`). | **High** (SHACL validation completely bypasses custom character/world instances). |
| **Multiplicity Enforcement** | Enforced (`sh:maxCount 1` on states). | Omitted (only checks `minCount 1` equivalent). | **Medium** (Allows characters/worlds to have multiple conflicting typestates). |
| **Blank Nodes** | Triggers validation error on anonymous OWL classes. | Triggers validation error on anonymous OWL classes. | **Medium** (False positives on standard OWL ontology structures). |
| **Namespace Sanity** | Enforces `http(s)://` strictly. | Only forbids `urn:`. | **Medium** (Opaque namespaces bypass SPARQL validation). |

---

## 3. Structural and Logical Gaps (Missing Checks)

The current validation boundaries are missing the following checks necessary for production-ready, branchless graph compilation:

1. **Node Graph Belonging (Parentage check)**:
   There is no validation enforcing that every `ue4:UEdGraphNode` belongs to a `ue4:UEdGraph` via a `ue4:nodeOf` relationship. Without this, nodes can float detached, and their connections will bypass RuleB.
2. **Data Pin Type Compatibility (Data flow safety)**:
   There is no verification that connected data pins have compatible types. For example, a `float` output pin can connect to a `string` input pin, or an incompatible object subclass pin can connect to another without triggering a violation.
3. **Pin-to-Parameter Type Compatibility (Call signature safety)**:
   No rule checks that the `pinCategory` and `pinSubCategoryObject` of a node pin match the expected type of the `UProperty` parameter it maps to.
4. **Data Flow Cycle Prevention (DAG validation)**:
   In Blueprints, while execution flows can have cycles, data flow connections must be a Directed Acyclic Graph (DAG). There is no validation to catch circular data dependencies, which would cause compiler hangs or stack overflows during code generation.
5. **Input Data Pin Value Completeness**:
   Every input data pin must either be connected (`ue4:connectedTo`) OR have a `ue4:defaultValue`. If it has neither, the graph is incomplete.
6. **Out-of-Bounds/Negative Ranges**:
   * `ue4:parameterIndex` is described as a "non-negative integer", but the SHACL shape lacks a `sh:minInclusive 0` constraint, allowing negative indices.
   * `ue4:interactionDistanceClass` has no minimum constraint, allowing negative distances.

---

## 4. Remediation Proposals

To address the findings, we propose the following concrete modifications:

### A. Corrected SPARQL Queries (in `ggen.toml`)

#### RuleC (Inverse Property & Existential Robustness)
Query `parameterOf` directly in case `hasParameter` is not populated:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
ASK {
  FILTER NOT EXISTS {
    ?pin ue4:pinOf ?node .
    ?node ue4:callsFunction ?func .
    ?pin ue4:mapsToParameter ?param .
    FILTER NOT EXISTS {
      { ?func ue4:hasParameter ?param } UNION { ?param ue4:parameterOf ?func }
    }
  }
}
```

#### RuleE (Symmetry Protection)
Ensure connection direction does not bypass category validation:
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
ASK {
  FILTER NOT EXISTS {
    { ?pin1 ue4:connectedTo ?pin2 } UNION { ?pin2 ue4:connectedTo ?pin1 }
    ?pin1 ue4:pinCategory "exec" .
    ?pin2 ue4:pinCategory ?cat .
    FILTER (?cat != "exec")
  }
}
```

#### RuleF & RuleG (Multiplicity Checking)
Validate that there is exactly one state (fail if 0 or >1):
```sparql
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
ASK {
  # Flag if character lacks a state OR has multiple states
  FILTER NOT EXISTS {
    ?char a ?charClass .
    ?charClass rdfs:subClassOf* ue4:ACharacter .
    
    # Must have at least one
    FILTER NOT EXISTS {
      ?char ue4:hasCookingState ?state .
      ?state a/rdfs:subClassOf* ue4:CookingTypestate .
    }
  }
  FILTER NOT EXISTS {
    ?char a ?charClass .
    ?charClass rdfs:subClassOf* ue4:ACharacter .
    
    # Must not have more than one
    ?char ue4:hasCookingState ?state1 .
    ?state1 a/rdfs:subClassOf* ue4:CookingTypestate .
    ?char ue4:hasCookingState ?state2 .
    ?state2 a/rdfs:subClassOf* ue4:CookingTypestate .
    FILTER (?state1 != ?state2)
  }
}
```

#### RuleLabel (Blank Node Exclusions)
```sparql
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
ASK {
  FILTER NOT EXISTS {
    ?class a ?type .
    FILTER (?type = owl:Class || ?type = rdfs:Class)
    FILTER (isIRI(?class))
    FILTER NOT EXISTS { ?class rdfs:label ?label }
  }
}
```

#### RuleNamespace (Inclusion vs. Exclusion)
```sparql
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
ASK {
  FILTER NOT EXISTS {
    ?subj a ?type .
    FILTER (?type = owl:Class || ?type = rdfs:Class || ?type = rdf:Property || ?type = owl:ObjectProperty || ?type = owl:DatatypeProperty)
    FILTER (!strstarts(str(?subj), "http://") && !strstarts(str(?subj), "https://"))
  }
}
```

### B. Corrected SHACL Shapes (in `validation.shacl.ttl`)

#### Subclass-Aware Targets using `sh:target`
For `ue4:CharacterCookingStateShape` and `ue4:WorldPackagingStateShape`, replace `sh:targetClass` with SPARQL targets to capture subclasses:
```turtle
ue4:CharacterCookingStateShape
    a sh:NodeShape ;
    sh:target [
        a sh:SPARQLTarget ;
        sh:select """
            PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            SELECT ?this WHERE {
                ?this a ?class .
                ?class rdfs:subClassOf* ue4:ACharacter .
            }
        """ ;
    ] ;
    sh:property [
        sh:path ue4:hasCookingState ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class ue4:CookingTypestate ;
        sh:message "A character must have exactly one cooking state of type CookingTypestate." ;
    ] .
```

#### Range Checking on `ue4:parameterIndex`
Add `sh:minInclusive 0` to enforce non-negative parameter indices:
```turtle
    sh:property [
        sh:path ue4:parameterIndex ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
        sh:minInclusive 0 ;
        sh:message "A parameter must have a single non-negative integer parameterIndex." ;
    ] .
```
