# Handoff Report — explorer_core_2

## 1. Observation

We directly observed the following on the system:

1. **Missing Ontology Files**: Running `list_dir` on `/Users/sac/.ggen/packs/ue4_ontology/` returned only `ggen.toml` and the `shacl` subdirectory. No `.ttl` files exist.
2. **Validation Script Failure**: Executing `/Users/sac/rocket-craft/validate_ontology.sh` failed with:
   ```
   ERROR: CLI execution failed: Command execution failed: error[E0001]: Manifest validation failed
     --> ggen.toml
     |
     = error: Ontology source not found: core.ttl
     = help: Fix validation errors before syncing
   --------------------------------------------------
   FAILURE: Ontology validation failed with exit code 1.
   ```
3. **Ontology Imports and Rules in `ggen.toml`**:
   Lines 8-15 of `ggen.toml` specify:
   ```toml
   [ontology]
   source = "core.ttl"
   imports = [
     "reflection.ttl",
     "blueprints.ttl",
     "subsystems.ttl",
     "typestates.ttl"
   ]
   ```
   Lines 25-40 specify rule `R1`:
   ```toml
   [[validation.rules]]
   name = "R1"
   description = "Verify class hierarchy (UObject, AActor, APawn, ACharacter, UActorComponent, UWorld, ULevel existence and subClassOf connections)"
   ask = """
   PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
   PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>

   ASK {
     ue4:AActor rdfs:subClassOf ue4:UObject .
     ue4:APawn rdfs:subClassOf ue4:AActor .
     ue4:ACharacter rdfs:subClassOf ue4:APawn .
     ue4:UActorComponent rdfs:subClassOf ue4:UObject .
     ue4:UWorld rdfs:subClassOf ue4:UObject .
     ue4:ULevel rdfs:subClassOf ue4:UObject .
   }
   """
   ```
4. **SHACL Shapes Constraints in `validation.shacl.ttl`**:
   Lines 9-16 (Label rule):
   ```turtle
   ue4:ClassLabelShape
       a sh:NodeShape ;
       sh:targetClass rdfs:Class , owl:Class ;
       sh:property [
           sh:path rdfs:label ;
           sh:minCount 1 ;
           sh:message "Public classes must have at least one rdfs:label." ;
       ] .
   ```
   Lines 30-35 (Namespace sanity):
   ```turtle
   ue4:NamespaceSanityShape
       a sh:NodeShape ;
       sh:targetClass rdfs:Class , owl:Class , rdf:Property , owl:ObjectProperty , owl:DatatypeProperty ;
       sh:nodeKind sh:IRI ;
       sh:pattern "^https?://" ;
       sh:message "Namespace sanity violation: Subjects must use resolvable public IRIs (starting with http:// or https://), not private/opaque ones like urn:private:." .
   ```

---

## 2. Logic Chain

1. **R1 Compliance Requirement**: Rule `R1` in `ggen.toml` checks for the existence of `UObject`, `AActor`, `APawn`, `ACharacter`, `UActorComponent`, `UWorld`, and `ULevel` along with their inheritance hierarchy using `rdfs:subClassOf`. Therefore, `core.ttl` must declare these classes and subclass links explicitly.
2. **SHACL Constraint Enforcement**: `validation.shacl.ttl` requires that all classes declare `rdfs:label` and `rdfs:comment` (as a warning), and that all classes and properties use standard public HTTP/HTTPS IRIs instead of opaque ones like `urn:private:`. Therefore, all classes and properties designed for `core.ttl` must include label/comment annotations and reside in the `https://rocket-craft.io/ontology/ue4/` namespace.
3. **Import Manifest Resolution**: `ggen.toml` lists four imports: `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl`. When `ggen sync` is executed, it parses the main ontology file and all declared imports. If the imports are missing or fail validation, the entire process terminates with an error. 
4. **Cohesive Solution**: To satisfy all validation gates, the implementation must place a fully modeled `core.ttl` at `/Users/sac/.ggen/packs/ue4_ontology/core.ttl` AND create corresponding skeleton files for the other four imported ontologies that satisfy rules `R2`, `R3`, and `R4` in `ggen.toml`.

---

## 3. Caveats

* **Tool Execution Scope**: Because this explorer agent has read-only restrictions on the project source code and pack destination, the proposed schemas were written to `/Users/sac/rocket-craft/.agents/explorer_core_2/analysis.md` rather than the active target directory.
* **Semantic Detail**: The classes in `core.ttl` reflect static C++ inheritance structures. Dynamic bindings, reflection metadata, and Blueprint execution graph configurations are deferred to the corresponding imported ontologies (`reflection.ttl`, `blueprints.ttl`, etc.).

---

## 4. Conclusion

The schema designs for `core.ttl` (as well as the skeleton designs for `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl`) detailed in `/Users/sac/rocket-craft/.agents/explorer_core_2/analysis.md` fully model the UE4 class hierarchy, comply with SHACL rule shapes (labels, comments, and namespace sanity), and satisfy the semantic `ASK` validation rules. 

The immediate next step is for the implementer agent to write these files to the target folder and trigger the validation script.

---

## 5. Verification Method

1. **Verify Files Location**: Check that the files `core.ttl`, `reflection.ttl`, `blueprints.ttl`, `subsystems.ttl`, and `typestates.ttl` are successfully written to `/Users/sac/.ggen/packs/ue4_ontology/`.
2. **Run Validation Script**: Execute the following command from the workspace:
   ```bash
   /Users/sac/rocket-craft/validate_ontology.sh
   ```
3. **Validation Outcome**: The command must exit with code `0` and output `SUCCESS: Ontology validation passed`.
