# Forensic Audit & Review Handoff Report

This report presents a thorough review and adversarial critique of the validation integration in the `ue4_ontology` pack, focusing on `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and the `validation.rules` section of `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml`.

---

## Review Summary

**Verdict**: REQUEST_CHANGES

### Summary of Findings

1. **Critical Parentage Check Mismatch (Rule H vs. Shape)**: The SHACL shape `ue4:InputExecPinConnectedShape` checks for input execution pin connections only on direct instances of `ue4:UK2Node_CallFunction` using `?node a ue4:UK2Node_CallFunction .`. It does not perform a dynamic, transitive subclass check. This fails to target subclasses of `ue4:UK2Node_CallFunction` (e.g. `ue4:UK2Node_CommutativeAssociativeBinaryOperator`) and completely ignores other execution-flow K2 nodes (e.g. `ue4:UK2Node_IfThenElse`, `ue4:UK2Node_ExecutionSequence`). This directly contradicts `RuleH` in `ggen.toml` which correctly uses `?node a/rdfs:subClassOf* ue4:UK2Node .` to check all subclasses dynamically.
2. **Major Coverage Gaps (Missing SHACL Shapes)**: Two custom SPARQL rules in `ggen.toml` (`RuleInputPinConnection` and `RuleNodeParentage`) have no corresponding SHACL shapes in `validation.shacl.ttl`. Consequently, input pin connection count limits and node parentage constraints are not validated during the SHACL phase.
3. **Major Subclass Targeting Gaps**: `ue4:SceneComponentRenderingShape` targets `ue4:USceneComponent` using `sh:targetClass`. In reasoner-free SHACL execution, this will fail to validate custom scene component subclasses (e.g. static mesh or skeletal mesh components).
4. **Minor Pattern Constraint Issue**: `ue4:UFunctionParameterShape` enforces `sh:pattern "^[0-9]+$"` on a property with `sh:datatype xsd:integer`. This is non-standard and prone to engine-specific failures; standard practice is to use `sh:minInclusive 0`.

---

## 1. Observation

Direct observations from the codebase files:

### Mismatch in Rule H / Exec Pin Connection
- **SHACL Shape** (`/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` lines 255-272):
  ```turtle
  # Rule H: Input execution pins on function call nodes must be connected
  ue4:InputExecPinConnectedShape
      a sh:NodeShape ;
      sh:targetClass ue4:UEdGraphPin ;
      sh:sparql [
          sh:message "Input execution pin on a function call node must be connected to an execution pin." ;
          sh:select """
              PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
              SELECT $this
              WHERE {
                  $this ue4:pinOf ?node .
                  ?node a ue4:UK2Node_CallFunction .
                  $this ue4:pinDirection ue4:Input .
                  $this ue4:pinCategory "exec" .
                  FILTER NOT EXISTS { $this (ue4:connectedTo|^ue4:connectedTo|ue4:linkedTo|^ue4:linkedTo) ?other }
              } ORDER BY $this
          """ ;
      ] .
  ```
- **TOML Rule** (`/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` lines 273-290):
  ```toml
  [[validation.rules]]
  name = "RuleH"
  description = "Input Exec Pin Connected Constraint (Broken execution flow): Input execution pins on function call nodes must be connected to an output execution pin."
  ask = """
  PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
  PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

  ASK {
    FILTER NOT EXISTS {
      ?pin ue4:pinOf ?node .
      ?node a/rdfs:subClassOf* ue4:UK2Node .
      ?pin ue4:pinDirection ue4:Input .
      ?pin ue4:pinCategory "exec" .
      FILTER NOT EXISTS { ?pin (ue4:connectedTo|^ue4:connectedTo|ue4:linkedTo|^ue4:linkedTo) ?other }
    }
  }
  """
  ```

### Missing SHACL Shapes
- There is no shape corresponding to `RuleInputPinConnection` (TOML lines 324-337) in `validation.shacl.ttl`.
- There is no shape corresponding to `RuleNodeParentage` (TOML lines 340-363) in `validation.shacl.ttl`.

### Static Target Class Constraints
- **Scene Component Shape** (`validation.shacl.ttl` lines 275-277):
  ```turtle
  ue4:SceneComponentRenderingShape
      a sh:NodeShape ;
      sh:targetClass ue4:USceneComponent ;
  ```

---

## 2. Logic Chain

1. **Rule H Mismatch**:
   - The SHACL shape `ue4:InputExecPinConnectedShape` queries: `?node a ue4:UK2Node_CallFunction`.
   - In RDF, without an active RDFS reasoner, direct type queries (`?x a :Class`) will not match instances of subclasses of `:Class`.
   - In `blueprints.ttl`, `ue4:UK2Node_CommutativeAssociativeBinaryOperator` is defined as a subclass of `ue4:UK2Node_CallFunction`. Thus, any instances of `UK2Node_CommutativeAssociativeBinaryOperator` will bypass this SHACL check.
   - Furthermore, other subclasses of `ue4:UK2Node` (such as `ue4:UK2Node_IfThenElse` or `ue4:UK2Node_ExecutionSequence`) which have input execution pins are completely skipped.
   - Conversely, the TOML `RuleH` checks `?node a/rdfs:subClassOf* ue4:UK2Node`, ensuring that *all* K2 nodes' input execution pins are connected.
   - This creates a logical inconsistency: the TOML validator will fail on unconnected input execution pins of sequence or branch nodes, while the SHACL validator will pass them.

2. **Missing SHACL Shapes**:
   - `ggen.toml` runs `RuleInputPinConnection` and `RuleNodeParentage`.
   - `validation.shacl.ttl` does not implement these constraints.
   - This leads to asymmetric validation, where running only SHACL validation will miss structural anomalies like nodes belonging to multiple graphs or input pins with multiple connections.

3. **Subclass Targeting in Property Shapes**:
   - `ue4:SceneComponentRenderingShape` uses `sh:targetClass ue4:USceneComponent`.
   - If a custom scene component class is defined (e.g. `UMyCustomMeshComponent rdfs:subClassOf ue4:USceneComponent`), its instances will not be validated by this shape unless the SHACL engine is forced to run with full RDFS inference (which is slow and often omitted in lightweight validation scripts).
   - This stands in contrast to `ue4:CharacterCookingStateShape` and `ue4:WorldPackagingStateShape`, which correctly use `sh:targetSubjectsOf rdf:type` and a SPARQL query to dynamically resolve all subclasses via `rdfs:subClassOf*`.

---

## 3. Caveats

- **No runtime commands executed**: As instructed by the constraint "Run no build/test commands," no compiler or validation scripts were executed. All observations are based strictly on static analysis of the Turtle schemas and TOML configurations.
- **Reasoner dependency**: Some SHACL engines can be configured to infer subclass targets automatically. If such a reasoner is always present, the `sh:targetClass` issue for `USceneComponent` is mitigated, but the `ue4:InputExecPinConnectedShape` query itself remains broken because it explicitly uses `?node a ue4:UK2Node_CallFunction` inside a SPARQL SELECT block, which overrides any engine-level target subclass resolution.

---

## 4. Conclusion

The validation integration contains a **critical logical discrepancy** and **dynamic subclass targeting failure** in Rule H / `ue4:InputExecPinConnectedShape`, alongside **coverage gaps** where TOML rules are missing their SHACL counterparts. The work requires changes to align the two validation suites and ensure all subclass instances are dynamically and correctly verified.

---

## 5. Verification Method

### Inspections
- Open `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` and verify the SPARQL query for `ue4:InputExecPinConnectedShape` (lines 255–272).
- Compare it to the SPARQL query for `RuleH` in `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (lines 273–290).

### Invalidation Condition
- Create a test graph containing an instance of `ue4:UK2Node_IfThenElse` with an unconnected input execution pin.
- Run `ggen validate` or the custom SPARQL/SHACL validation engines.
- The TOML rule `RuleH` must fail the graph, whereas the SHACL shape `ue4:InputExecPinConnectedShape` will incorrectly report `PASS`.

---

# Adversarial Review Critique

## Challenge Summary

**Overall risk assessment**: MEDIUM

### Challenges

#### [High] Challenge 1: SHACL Bypass on Dead Execution Flow
- **Assumption challenged**: That the SHACL validator protects the pipeline against compilation of dead/unreachable execution flow nodes.
- **Attack scenario**: A user generates a Blueprint containing a `UK2Node_IfThenElse` node with an unconnected input execution pin. Since this is not a `UK2Node_CallFunction`, the SHACL validator passes the asset.
- **Blast radius**: The generated WASM module will compile but contain a dead branch node that never triggers, causing silent runtime failures in the HTML5 environment.
- **Mitigation**: Update `ue4:InputExecPinConnectedShape` in `validation.shacl.ttl` to check all subclasses of `ue4:UK2Node` dynamically:
  ```sparql
  SELECT $this
  WHERE {
      $this ue4:pinOf ?node .
      ?node a/rdfs:subClassOf* ue4:UK2Node .
      $this ue4:pinDirection ue4:Input .
      $this ue4:pinCategory "exec" .
      FILTER NOT EXISTS { $this (ue4:connectedTo|^ue4:connectedTo|ue4:linkedTo|^ue4:linkedTo) ?other }
  } ORDER BY $this
  ```

#### [Medium] Challenge 2: Static Targeting Failure on Component Extensions
- **Assumption challenged**: That custom or generated Scene Components will have their rendering properties validated by `ue4:SceneComponentRenderingShape`.
- **Attack scenario**: A generator outputs a mesh component class that inherits from `ue4:USceneComponent` and defines an invalid datatype (e.g. string) for `interactionDistanceClass`.
- **Blast radius**: Since `sh:targetClass` is static and does not traverse subclasses without a reasoner, the validation succeeds, but the downstream C++ generator emits invalid float-to-string code, breaking the build.
- **Mitigation**: Redefine `ue4:SceneComponentRenderingShape` to dynamically target all scene component subclasses:
  ```turtle
  ue4:SceneComponentRenderingShape
      a sh:NodeShape ;
      sh:targetSubjectsOf rdf:type ;
      sh:sparql [
          sh:message "Scene component rendering property validation error." ;
          sh:select """
              PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
              PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
              SELECT $this
              WHERE {
                  $this a ?class .
                  ?class rdfs:subClassOf* ue4:USceneComponent .
                  # (Property check logic or use sh:property constraints targeting the selected nodes)
              }
          """
      ] .
  ```
