# Research Dossier: `cargo-cicd`

**Total Files:** 14 Ontologies (.ttl) | 9 Queries (.rq)
**Total Volume:** 23 files

## 1. Core Vocabularies (Prefixes)
- `cc: <https://cargo-cicd.rs/ontology/>`
- `ccsh: <https://cargo-cicd.rs/shapes/>`
- `cicd: <https://cargo-cicd.dev/vocab#>`
- `cicd: <https://cargo-cicd.rs/ontology#>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dcterms: <http://purl.org/dc/terms/>`
- `odrl: <http://www.w3.org/ns/odrl/2/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `cc:PlaygroundScenario`
- `cc:ReleaseCondition`
- `ccsh:CapabilityReferenceCoverageShape`
- `ccsh:CapabilityShape`
- `ccsh:DocForbiddenTermsShape`
- `ccsh:EvidenceEventShape`
- `ccsh:PlaygroundRefuseRequiresMutationShape`
- `ccsh:PlaygroundScenarioShape`
- `ccsh:PublishReadyDatasetShape`
- `ccsh:ReleaseConditionBlockerShape`
- `ccsh:ReleaseConditionShape`
- `ccsh:TutorialCapabilityCoverageShape`
- `cicd:CicdActivity`
- `cicd:DocType`
- `cicd:EvidenceEvent`
- `cicd:Feature`
- `cicd:PlaygroundScenario`

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 9}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?activity`, `?audience`, `?blocker_reason`, `?capabilities_explained`, `?cli_command`, `?condition_description`, `?condition_met`, `?condition_number`, `?description`, `?design_decision`, `?doc_id`, `?emitted_by`, `?event_name`, `?evidence_event_label`, `?feature_flag`, `?inputs`, `?noun_about`, `?noun_name`, `?outputs`, `?public_boundary_clean`, `?title`, `?validated`, `?verb_about`, `?verb_name`, `?wasm4pm_oracle`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/cargo-cicd/ontology/cargo-cicd.ttl` (9175 bytes)
- `/Users/sac/cargo-cicd/ontology/cicd-process.ttl` (1862 bytes)
- `/Users/sac/cargo-cicd/ontology/public/cargo-cicd-capabilities.ttl` (9431 bytes)
- `/Users/sac/cargo-cicd/ontology/public/cargo-cicd-commands.ttl` (5786 bytes)
- `/Users/sac/cargo-cicd/ontology/public/cargo-cicd-evidence.ttl` (8164 bytes)
- `/Users/sac/cargo-cicd/ontology/public/cargo-cicd-playground.ttl` (14201 bytes)
- `/Users/sac/cargo-cicd/ontology/public/crates-io-release.ttl` (10736 bytes)
- `/Users/sac/cargo-cicd/ontology/public/diataxis.ttl` (5938 bytes)
- `/Users/sac/cargo-cicd/ontology/public/prefixes.ttl` (4438 bytes)
- `/Users/sac/cargo-cicd/queries/commands.rq` (401 bytes)
- `/Users/sac/cargo-cicd/queries/docs-explanation.rq` (1704 bytes)
- `/Users/sac/cargo-cicd/queries/docs-howto.rq` (490 bytes)
- `/Users/sac/cargo-cicd/queries/docs-readme.rq` (412 bytes)
- `/Users/sac/cargo-cicd/queries/docs-reference-command.rq` (2107 bytes)
- `/Users/sac/cargo-cicd/queries/docs-tutorial.rq` (1674 bytes)
- `/Users/sac/cargo-cicd/queries/evidence-cases.rq` (390 bytes)
- `/Users/sac/cargo-cicd/queries/playground-matrix.rq` (412 bytes)
- `/Users/sac/cargo-cicd/queries/release-checklist.rq` (1266 bytes)
- `/Users/sac/cargo-cicd/shapes/commands.shacl.ttl` (5013 bytes)
- `/Users/sac/cargo-cicd/shapes/docs.shacl.ttl` (6488 bytes)
- `/Users/sac/cargo-cicd/shapes/evidence.shacl.ttl` (4484 bytes)
- `/Users/sac/cargo-cicd/shapes/playground.shacl.ttl` (4685 bytes)
- `/Users/sac/cargo-cicd/shapes/release.shacl.ttl` (4878 bytes)

</details>
