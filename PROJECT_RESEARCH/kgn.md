# Research Dossier: `kgn`

**Total Files:** 51 Ontologies (.ttl) | 3 Queries (.rq)
**Total Volume:** 54 files

## 1. Core Vocabularies (Prefixes)
- `attest: <http://kgen.dfllss.org/ontology/attest/>`
- `cli: <http://kgen.dfllss.org/ontology/cli/>`
- `crypto: <http://www.w3.org/ns/auth/acl#>`
- `dc: <http://purl.org/dc/elements/1.1/>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `ex: <http://example.org/>`
- `ex: <http://example.org/sales/>`
- `ex: <http://example.org/shapes#>`
- `ex: <https://kgen.dev/example#>`
- `fin: <https://kgen.dev/ontologies/finance#>`
- `financial: <http://example.com/financial/>`
- `foaf: <http://www.foaf.org/0.1/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `fr: <http://example.org/financial-reporting#>`
- `gaap: <https://kgen.dev/ontology/gaap#>`
- `inject: <https://kgen.dev/inject#>`
- `kgen: <http://example.com/kgen/>`
- `kgen: <http://example.org/kgen#>`
- `kgen: <http://kgen.ai/ontology#>`
- `kgen: <http://kgen.dfllss.org/ontology/>`
- `kgen: <https://kgen.dev/ontology#>`
- `office: <http://example.com/office/>`
- `org: <http://www.w3.org/ns/org#>`
- `org: <https://kgen.dev/ontologies/organization#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `schema: <http://schema.org/>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `sox: <https://kgen.dev/ontology/sox#>`
- `time: <http://www.w3.org/2006/time#>`
- `tmpl: <https://kgen.dev/template#>`
- `vcard: <http://www.w3.org/2006/vcard/ns#>`
- `xml: <http://www.w3.org/XML/1998/namespace>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:Anomaly`
- `:ArchitectureShape`
- `:Award`
- `:AwardConsistency`
- `:BoxOfficeValidation`
- `:Building`
- `:CTQMetricShape`
- `:CharterShape`
- `:ComponentShape`
- `:Condition`
- `:Disease`
- `:GeneratedArtifactShape`
- `:Genre`
- `:Location`
- `:Medication`
- `:MedicationShape`
- `:MilestoneShape`
- `:Movie`
- `:MovieShape`
- `:Patient`
- `:PatientShape`
- `:Person`
- `:PersonShape`
- `:Post`
- `:Procedure`
- `:ProjectShape`
- `:Reading`
- `:ReadingShape`
- `:SelfHostingCapabilityShape`
- `:SelfHostingQualityGate`
- `:Sensor`
- `:StakeholderShape`
- `:Studio`
- `:StudioShape`
- `:Symptom`
- `:TestShape`
- `:TestSuiteShape`
- `:Treatment`
- `:TreatmentSafetyShape`
- `attest:Attestation`
- `attest:AttestationShape`
- `attest:Environment`
- `attest:EnvironmentShape`
- `attest:GenerationRecord`
- `attest:GenerationRecordShape`
- `attest:Hash`
- `attest:HashShape`
- `attest:Input`
- `attest:InputShape`
- `attest:Output`
- *...and 175 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 2, 'CONSTRUCT': 1}

### Projected Variables (SELECT ?var)
- *No specific projection variables identified.*

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/kgn/examples/ontologies/sales-ontology.ttl` (5078 bytes)
- `/Users/sac/kgn/examples/shapes/sales-shapes.shacl.ttl` (4904 bytes)
- `/Users/sac/kgn/examples/workshop/knowledge/sample.ttl` (6399 bytes)
- `/Users/sac/kgn/examples/workshop/knowledge/shapes.ttl` (8412 bytes)
- `/Users/sac/kgn/examples/workshop/rdf/kgen-shapes.ttl` (10757 bytes)
- `/Users/sac/kgn/examples/workshop/rdf/workshop.ttl` (9482 bytes)
- `/Users/sac/kgn/examples/workshop/validation/validation-report.ttl` (224 bytes)
- `/Users/sac/kgn/features/fixtures/office/data/sample-rdf-data.ttl` (9583 bytes)
- `/Users/sac/kgn/features/fixtures/project-data.ttl` (510 bytes)
- `/Users/sac/kgn/features/fixtures/simple-component.ttl` (691 bytes)
- `/Users/sac/kgn/features/fixtures/structured-users.ttl` (958 bytes)
- `/Users/sac/kgn/ontologies/attest.ttl` (7491 bytes)
- `/Users/sac/kgn/ontologies/cli.ttl` (6321 bytes)
- `/Users/sac/kgn/ontologies/shacl/artifact-shapes.ttl` (17839 bytes)
- `/Users/sac/kgn/ontologies/shacl/attest-shapes.ttl` (8517 bytes)
- `/Users/sac/kgn/ontologies/shacl/cli-shapes.ttl` (5624 bytes)
- `/Users/sac/kgn/ontologies/shacl/injection-shapes.ttl` (23507 bytes)
- `/Users/sac/kgn/ontologies/shacl/template-inheritance-shapes.ttl` (24111 bytes)
- `/Users/sac/kgn/packages/assets/facts.ttl` (1089 bytes)
- `/Users/sac/kgn/packages/assets/ontology.ttl` (1062 bytes)
- `/Users/sac/kgn/packages/assets/shapes.ttl` (1202 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/assets/facts-enhanced.ttl` (6447 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/assets/facts.ttl` (2593 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/assets/ontology.ttl` (1528 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/assets/shapes.ttl` (763 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/healthcare/data.ttl` (1588 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/healthcare/ontology.ttl` (1196 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/healthcare/shapes.ttl` (1461 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/iot/ontology.ttl` (1004 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/iot/shapes.ttl` (1028 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/movies/data.ttl` (5013 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/movies/ontology.ttl` (3729 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/movies/shapes-basic.ttl` (2173 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/movies/shapes.ttl` (3313 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/examples/social/social-graph.ttl` (2261 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/job-hunt-captures/job-1757980990918/provenance.ttl` (1147 bytes)
- `/Users/sac/kgn/packages/kgen-adapters/resume-demo-output/sean-chatman-resume.ttl` (4946 bytes)
- `/Users/sac/kgn/packages/kgen-core/test-data/sample1.ttl` (550 bytes)
- `/Users/sac/kgn/packages/kgen-core/test-data/sample2.ttl` (666 bytes)
- `/Users/sac/kgn/packages/test-utils/test-data/sample-project.ttl` (2959 bytes)
- `/Users/sac/kgn/packages/test-utils/test-data/sample-shacl.ttl` (4635 bytes)
- `/Users/sac/kgn/packs/finance-gaap-sox/IR/gaap-ontology.ttl` (6461 bytes)
- `/Users/sac/kgn/packs/finance-gaap-sox/IR/sox-compliance.ttl` (8148 bytes)
- `/Users/sac/kgn/tests/e2e/consolidated-reporting/dist/consolidated-graph.ttl` (3622 bytes)
- `/Users/sac/kgn/tests/e2e/consolidated-reporting/ontologies/global-finance-canon.ttl` (2553 bytes)
- `/Users/sac/kgn/tests/e2e/consolidated-reporting/queries/consolidated-financials.rq` (3528 bytes)
- `/Users/sac/kgn/tests/e2e/financial-reporting/dist/ledger-graph.ttl` (1746 bytes)
- `/Users/sac/kgn/tests/e2e/financial-reporting/mappers/ledger-to-rdf.rq` (1159 bytes)
- `/Users/sac/kgn/tests/e2e/financial-reporting/ontologies/finance-canon.ttl` (1477 bytes)
- `/Users/sac/kgn/tests/e2e/financial-reporting/queries/calculate-financials.rq` (860 bytes)
- `/Users/sac/kgn/tests/e2e/financial-reporting/shapes/accounting-principles.ttl` (1031 bytes)
- `/Users/sac/kgn/tests/e2e/workflow-test/input/financial-reporting.ttl` (3378 bytes)
- `/Users/sac/kgn/tests/test-data/sample-project.ttl` (2959 bytes)
- `/Users/sac/kgn/tests/test-data/sample-shacl.ttl` (4635 bytes)

</details>
