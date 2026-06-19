# Handoff Report

## 1. Observation
We examined the validation ontology, graph files, and test suites. Direct quotes and configurations observed include:

* **Observation 1: Inverse Property Mismatch in RuleC**
  In `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (lines 163-174):
  ```toml
  [[validation.rules]]
  name = "RuleC"
  description = "Function Call Parameter Mapping Target Integrity: Pin maps to a parameter that does not belong to the node's target called function."
  ask = """
  PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
  ASK {
    FILTER NOT EXISTS {
      ?pin ue4:pinOf ?node .
      ?node ue4:callsFunction ?func .
      ?pin ue4:mapsToParameter ?param .
      FILTER NOT EXISTS { ?func ue4:hasParameter ?param }
    }
  }
  """
  ```
  However, in `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 40-47), the schema only enforces `ue4:parameterOf`:
  ```turtle
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
  ```

* **Observation 2: Asymmetric Connection Check in RuleE**
  In `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 164-179):
  ```turtle
  # Rule E: Exec vs. Data Pin Separation
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
              } ORDER BY $this ?other
          """ ;
      ] .
  ```
  This query assumes connections are always traversed starting from the execution pin as the subject.

* **Observation 3: Subclass Blindness in SHACL shapes**
  In `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 186-195):
  ```turtle
  ue4:CharacterCookingStateShape
      a sh:NodeShape ;
      sh:targetClass ue4:ACharacter ;
      sh:property [
          sh:path ue4:hasCookingState ;
          sh:minCount 1 ;
          sh:maxCount 1 ;
          sh:class ue4:CookingTypestate ;
          sh:message "A character must have exactly one cooking state of type CookingTypestate." ;
      ] .
  ```
  And in `/Users/sac/rocket-craft/ggen-validation-tests/gundam_character.ttl` (lines 17-20):
  ```turtle
  gundam:AGundamCharacter a owl:Class ;
      rdfs:subClassOf ue4:ACharacter ;
  ```

* **Observation 4: Namespace Rule Discrepancy**
  In `/Users/sac/.ggen/packs/ue4_ontology/shacl/validation.shacl.ttl` (lines 29-35):
  ```turtle
  ue4:NamespaceSanityShape
      a sh:NodeShape ;
      sh:targetClass rdfs:Class , owl:Class , rdf:Property , owl:ObjectProperty , owl:DatatypeProperty ;
      sh:nodeKind sh:IRI ;
      sh:pattern "^https?://" ;
  ```
  But in `/Users/sac/.ggen/packs/ue4_ontology/ggen.toml` (lines 280-293):
  ```toml
  [[validation.rules]]
  name = "RuleNamespace"
  ask = """
  ASK {
    FILTER NOT EXISTS {
      ?subj a ?type .
      FILTER (?type = owl:Class || ?type = rdfs:Class || ?type = rdf:Property || ?type = owl:ObjectProperty || ?type = owl:DatatypeProperty)
      FILTER (strstarts(str(?subj), "urn:"))
    }
  }
  """
  ```

---

## 2. Logic Chain
1. **Inverse Property (Observation 1)**: Simple SPARQL engines (like the one in `ggen`) do not perform RDFS/OWL reasoning out-of-the-box. If the instance graph defines parameter structures using `ue4:parameterOf` (as standard in `UFunctionParameterShape`), the triple `?func ue4:hasParameter ?param` will be absent. Consequently, RuleC will always find that the parameter does not belong to the function, producing 100% false positives.
2. **Symmetry (Observation 2)**: `ue4:connectedTo` is declared symmetric, but if symmetric reasoning is not active in the SPARQL evaluator, a unidirectional triple like `pin_float ue4:connectedTo pin_exec` will fail the matching pattern. Because the check only validates when the subject is the execution pin, this mismatch will be silently admitted.
3. **Subclass Blindness (Observation 3)**: Standard SHACL engines targeting classes via `sh:targetClass` do not automatically evaluate subclass hierarchies. Since `gundam:AGundamCharacter` is a subclass of `ue4:ACharacter`, instances of custom character models (such as `gundam:MyGundam`) will bypass `ue4:CharacterCookingStateShape` validation entirely in SHACL.
4. **Namespace Discrepancy (Observation 4)**: The SHACL shape enforces that URIs must start with `http://` or `https://`. The SPARQL rule only blocks `urn:`. This means relative or other opaque schemes (e.g., `uuid:`) will cause inconsistencies, failing SHACL while passing SPARQL.

---

## 3. Caveats
* **Assumptions**: We assume `ggen` uses standard, lightweight RDF engines that do not automatically perform full OWL Reasoning/Entailment during SPARQL ask checks. If an external reasoner is used prior to validation, some issues (like inverse properties and symmetry) might be partially masked, but the rules should be written to be robust regardless.
* **Scope**: We did not execute build/test commands as per constraints. All findings are derived via direct static semantic analysis.

---

## 4. Conclusion
The validation suite has significant logical gaps that allow invalid configs to pass, and valid configurations to fail (false positives):
1. **False Positives in RuleC**: Without OWL reasoning, RuleC incorrectly flags all correct parameter mappings.
2. **Symmetry and Subclass Bypasses**: Unidirectional connections and custom character/world subclasses easily bypass validation rules.
3. **Multiplicity Defect**: SPARQL checks for RuleF and RuleG only check `minCount 1` equivalent, allowing multiple conflicting typestates.
4. **Blank Node Failures**: RuleLabel will break on any valid anonymous OWL class expressions.
5. **Missing DAG/Parentage Checks**: No cycle checks exist for data flow, and no parent graph constraints exist for nodes.

---

## 5. Verification Method
To verify these findings:
1. Run the test script `verify_all_rules.sh` located in `/Users/sac/rocket-craft/ggen-validation-tests/`.
2. Inspect the validation outputs and check for inconsistency between SHACL output and SPARQL output.
3. Introduce a custom world class (subclass of `UWorld`) and verify that the SHACL shape `ue4:WorldPackagingStateShape` fails to flag a missing packaging state.
4. Introduce a blank node class and verify that `RuleLabel` flags it as a defect.
5. Introduce a floating node with no `ue4:nodeOf` relationship connected to a node in a different graph, and verify that `RuleB` fails to catch this cross-graph connection.
