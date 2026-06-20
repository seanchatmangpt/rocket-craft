# Challenger Report

## Challenge Summary

**Overall risk assessment**: MEDIUM

While the remediated C++ Backbone ontology compiles successfully and passes all configured GGen quality gates, our adversarial review identified three specific structural vulnerabilities that could lead to silent failures, validation bypasses, or malformed C++ class generation in downstream projects.

---

## Challenges

### [Medium] Challenge 1: Subproperty Inference Failure

- **Assumption challenged**: The assumption that standard SPARQL CONSTRUCT inference rules (like `infer-is-component-of` and `infer-is-level-of`) automatically cover subproperties of the primary relations.
- **Attack scenario**: A downstream project defines level actors using specific subproperties such as `ue4:hasRootComponent` (subproperty of `ue4:hasComponent`) or `ue4:persistentLevel` (subproperty of `ue4:hasLevel`). Since the query in `ggen.toml` only checks the exact predicate (`ue4:hasComponent` / `ue4:hasLevel`), the inverse relations (`ue4:isComponentOf` / `ue4:isLevelOf`) are never inferred.
- **Blast radius**: Components and levels will lack inverse relation triples, causing generated C++ headers or reflection wrappers to miss owner pointers and level references.
- **Mitigation**: Update the SPARQL CONSTRUCT query in `ggen.toml` to be subproperty-aware using property paths:
  ```sparql
  CONSTRUCT {
    ?component ue4:isComponentOf ?actor .
  } WHERE {
    ?prop rdfs:subPropertyOf* ue4:hasComponent .
    ?actor ?prop ?component .
  }
  ```

### [Medium] Challenge 2: SHACL Namespace Sanity Bypass

- **Assumption challenged**: The assumption that the namespace sanity shape (`ue4:NamespaceSanityShape`) prevents all opaque/private IRIs (such as `urn:private:`) from leaking into the ontology class/property definitions.
- **Attack scenario**: An opaque class URI (e.g. `urn:private:opaqueClass`) is introduced as a subclass (e.g. `urn:private:opaqueClass rdfs:subClassOf ue4:AActor`), but it is not explicitly declared as an `owl:Class`. Since the SHACL shape uses `sh:targetClass rdfs:Class , owl:Class`, it fails to target the undeclared subject, allowing it to bypass the sanity check.
- **Blast radius**: Malformed private URIs will bypass the validation gates and corrupt generated C++ headers.
- **Mitigation**: Expand the SHACL shape target or use a SHACL SPARQL constraint to check any subject/object used within class or property hierarchies, regardless of whether it is explicitly typed as a class.

### [Low] Challenge 3: Unvalidated Circular Inheritance

- **Assumption challenged**: The assumption that C++ inheritance rules (non-circularity) are enforced in the ontology representation.
- **Attack scenario**: A user accidently creates a cycle in class definitions (e.g. `ex:ClassA rdfs:subClassOf ex:ClassB` and `ex:ClassB rdfs:subClassOf ex:ClassA`). Neither SHACL rules nor `ggen.toml` custom validations check for this.
- **Blast radius**: Downstream code generation tools will loop infinitely or produce uncompilable C++ code (C++ does not allow circular class inheritance).
- **Mitigation**: Add a validation rule to `ggen.toml` to detect circular subclass relationships:
  ```sparql
  PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
  ASK {
    ?cls rdfs:subClassOf+ ?cls .
  }
  ```
  If this returns true, the validation should fail.

---

## Stress Test Results

- **Scenario 1: Component & Level Inference validation** → Verify standard and inferred relationships → Standard instance generation produces `isComponentOf` and `isLevelOf` → **PASS**
- **Scenario 2: Subproperty Inference validation** → Verify if subproperty relationships trigger standard inference → Standard inference fails to match subproperties; subproperty-aware queries correctly retrieve them → **FAIL** (on standard queries) / **PASS** (on subproperty-aware queries)
- **Scenario 3: SHACL Namespace Sanity bypass** → Verify if an undeclared opaque URI class bypasses SHACL validation → Opaque subclass bypasses targetClass filter since it's not declared as `owl:Class` → **FAIL** (Sanity gate bypassed)
- **Scenario 4: Circular inheritance detection** → Introduce cycle and verify if current validation detects it → Current gates do not detect cycle; custom SPARQL query successfully detects it → **FAIL** (Validation gap)

---

## Unchallenged Areas

- **C++ Compiler WASM/HTML5 packaging** — Out of scope. We focused strictly on C++ Backbone ontology and its immediate graph validations.
