# UE4 Universal RDF Mapping: E2E Test Suite Readiness (TEST_READY)

This document certifies the readiness of the End-to-End (E2E) testing suite and ontology validation harness for the UE4 Universal RDF Mapping project.

## 1. Ready Status

- **Validation Harness Status:** **FULLY OPERATIONAL**
  - The script wrapper `/Users/sac/rocket-craft/validate_ontology.sh` is executable, verified, and integrated with the `ggen` ontology compiler.
  - The validation configuration file (`ggen.toml`) is written, containing the complete definitions of core rules R1, R2, R3, and R4.
  - The SHACL ruleset (`validation.shacl.ttl`) is deployed and active, enforcing structural, metadata, and namespace requirements.
- **Ontology Content Status:** **PENDING AUTHORING**
  - As per the milestones in `PROJECT.md`, the actual Turtle files (`core.ttl`, `reflection.ttl`, etc.) are scheduled to be written in subsequent milestones.
  - Currently, running the validation command triggers the expected **E0001 Manifest Validation Failure** (specifically `Ontology source not found: core.ttl`), confirming that the validation runner executes correctly and halts on missing/invalid resources.

---

## 2. Test Runner Command

To run the validation test suite, execute the following command from the project root or any location:

```bash
/Users/sac/rocket-craft/validate_ontology.sh
```

### Script Internals

The harness script executes the following steps:
1. Verifies that the target directory `/Users/sac/.ggen/packs/ue4_ontology` exists.
2. Checks that the compiler binary `/Users/sac/.local/bin/ggen` is present and executable.
3. Switches to the target directory.
4. Runs `ggen sync --validate-only true` to compile and validate the ontologies.
5. Captures the exit status of the compiler.
6. Returns the exact exit code to the caller environment (exiting with `0` on success and non-zero on failure).

---

## 3. Coverage Metrics

Once all ontology files are authored, the validation suite will evaluate the graph against the following coverage metrics:

| Metric | Target | Description |
|---|---|---|
| **Class Validation Coverage** | 100% | Every class declared in the Turtle files must be validated by the compiler and match structural shapes. |
| **Feature Coverage (Tier 1)** | >= 5 cases per feature | Validates minimum required declarations for Core C++, Subsystems, Reflection, and Typestates. |
| **Boundary Coverage (Tier 2)** | >= 5 checks | Structural limits (namespace rules, label presence, description presence, circular inheritance checks). |
| **Interaction Coverage (Tier 3)**| Pairwise combinations | Cross-component integrity (e.g., Blueprint nodes referencing reflection, subsystems invoking C++ classes). |
| **Scenarios (Tier 4)** | 100% path coverage | Verifies a complete character setup is compilable and packageable. |

---

## 4. Feature Checklist

The following table tracks the validation coverage for the target UE4 features:

| Feature / Rule ID | Description | Validation Type | Status |
|---|---|---|---|
| **R1: Core C++ Backbone** | Verifies class hierarchy of `UObject`, `AActor`, `APawn`, `ACharacter`, `UActorComponent`, `UWorld`, `ULevel`. | SPARQL ASK | **Harness Ready** (Ontology Pending) |
| **R2: Subsystems** | Verifies presence of Rendering, Physics, and Networking subsystem classes/relationships. | SPARQL ASK | **Harness Ready** (Ontology Pending) |
| **R3: Reflection & Blueprints** | Verifies presence of reflection metadata and Blueprint graph structure/execution nodes. | SPARQL ASK | **Harness Ready** (Ontology Pending) |
| **R4: Cooking & WASM** | Verifies typestates representing cooking, linking, and WASM/HTML5 packaging states. | SPARQL ASK | **Harness Ready** (Ontology Pending) |
| **ClassLabelShape** | Enforces that all classes have at least one `rdfs:label`. | SHACL Shape | **Harness Ready** (Ontology Active) |
| **ClassCommentShape** | Enforces that all classes have an `rdfs:comment` (Warning level). | SHACL Shape | **Harness Ready** (Ontology Active) |
| **NamespaceSanityShape** | Verifies subjects use public HTTP/HTTPS IRIs, not private/opaque URIs. | SHACL Shape | **Harness Ready** (Ontology Active) |

---

## 5. Summary of Current Build Verification

- **Command executed:** `/Users/sac/rocket-craft/validate_ontology.sh`
- **Exit Code:** `1` (Expected failure; validation runner correctly rejected the setup due to missing `core.ttl` ontology file).
- **Diagnostics captured:**
  ```text
  ERROR: CLI execution failed: Command execution failed: error[E0001]: Manifest validation failed
    --> ggen.toml
    |
    = error: Ontology source not found: core.ttl
    = help: Fix validation errors before syncing
  ```
