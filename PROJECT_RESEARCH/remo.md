# Research Dossier: `remo`

**Total Files:** 23 Ontologies (.ttl) | 0 Queries (.rq)
**Total Volume:** 23 files

## 1. Core Vocabularies (Prefixes)
- `capability: <https://weaver.opentelemetry.io/capability#>`
- `comp: <http://weaver.dev/ontologies/compliance#>`
- `dc: <http://purl.org/dc/elements/1.1/>`
- `dc: <http://purl.org/dc/terms/>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `ex: <http://example.org/workflow#>`
- `example: <http://rflow.dev/example/parallel-gateway#>`
- `example: <http://rflow.dev/example/simple-sequence#>`
- `example: <http://rflow.dev/example/timer-workflow#>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `forge: <https://remo.cli/ontology/forge/>`
- `job: <http://rflow.dev/ontology/job-model#>`
- `job: <http://rflow.io/ontology/job-model#>`
- `obs: <https://remo.cli/ontology/observatory/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `remo: <http://example.com/remo/ontology#>`
- `remo: <http://remo.cli/ontology#>`
- `remo: <http://remo.dev/ontology#>`
- `remo: <https://remo.cli/ontology/>`
- `remo: <https://remo.dev/ontology#>`
- `sec: <http://weaver.dev/ontologies/security#>`
- `security: <https://weaver.opentelemetry.io/domain/security#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `stack: <https://weaver.dev/ontology/stack#>`
- `tel: <http://weaver.dev/ontologies/telemetry#>`
- `telemetry: <https://remo.cli/ontology/telemetry/>`
- `time: <http://www.w3.org/2006/time#>`
- `weaver: <https://weaver.dev/ontology#>`
- `weaver: <https://weaver.opentelemetry.io/ontology#>`
- `wvr: <http://weaver.dev/ontologies/capabilities#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`
- `yawl: <http://rflow.dev/ontology/yawl-core#>`
- `yawl: <http://rflow.io/ontology/yawl-core#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `forge:Artifact`
- `forge:BuildCommand`
- `forge:BuildOperation`
- `forge:BuildPlan`
- `forge:BuildStep`
- `forge:DeployCommand`
- `forge:DeployOperation`
- `forge:Forge`
- `forge:PlanCommand`
- `forge:PlanOperation`
- `forge:Target`
- `obs:AnalyzeOperation`
- `obs:CodeFile`
- `obs:Dependency`
- `obs:IndexOperation`
- `obs:Observatory`
- `obs:QueryCommand`
- `obs:ScanCommand`
- `obs:ScanOperation`
- `obs:ScanResult`
- `obs:Symbol`
- `obs:WatchCommand`
- `remo:Action`
- `remo:Anomaly`
- `remo:Command`
- `remo:CommandOperation`
- `remo:Component`
- `remo:ComponentEntry`
- `remo:Config`
- `remo:ConfigCommand`
- `remo:ConfigOperation`
- `remo:ConfigValue`
- `remo:DataType`
- `remo:DatabaseEffect`
- `remo:Extension`
- `remo:FeedbackLoop`
- `remo:FileSystemEffect`
- `remo:Generator`
- `remo:Group`
- `remo:HealthStatus`
- `remo:Inference`
- `remo:Interface`
- `remo:Library`
- `remo:Metric`
- `remo:Model`
- `remo:MutationOperation`
- `remo:NetworkEffect`
- `remo:OntologyService`
- `remo:Operation`
- `remo:OperationStatus`
- *...and 102 more.*

## 3. Extraction Layer (SPARQL)
- *No queries executed in this project.*

### Projected Variables (SELECT ?var)
- *No specific projection variables identified.*

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/remo/dashboard/docs/weaver/ontologies/artifacts.ttl` (18089 bytes)
- `/Users/sac/remo/dashboard/docs/weaver/ontologies/capabilities.ttl` (44927 bytes)
- `/Users/sac/remo/dashboard/docs/weaver/ontologies/entities.ttl` (13665 bytes)
- `/Users/sac/remo/dashboard/docs/weaver/ontologies/stack-bindings.ttl` (18082 bytes)
- `/Users/sac/remo/dashboard/docs/weaver/ontologies/test-ontology.ttl` (5365 bytes)
- `/Users/sac/remo/dashboard/scripts/weaver/examples/test-ontology.ttl` (5365 bytes)
- `/Users/sac/remo/dashboard/tests/fixtures/test-ontology.ttl` (15404 bytes)
- `/Users/sac/remo/ontology/remo.ttl` (14703 bytes)
- `/Users/sac/remo/rflow/ontology/abox/exclusive-choice.ttl` (2698 bytes)
- `/Users/sac/remo/rflow/ontology/abox/parallel-gateway.ttl` (7675 bytes)
- `/Users/sac/remo/rflow/ontology/abox/parallel-split.ttl` (2644 bytes)
- `/Users/sac/remo/rflow/ontology/abox/simple-sequence.ttl` (4989 bytes)
- `/Users/sac/remo/rflow/ontology/abox/synchronization.ttl` (3833 bytes)
- `/Users/sac/remo/rflow/ontology/abox/timer-based.ttl` (2424 bytes)
- `/Users/sac/remo/rflow/ontology/abox/timer-workflow.ttl` (7200 bytes)
- `/Users/sac/remo/rflow/ontology/tbox/job-model.owl.ttl` (15093 bytes)
- `/Users/sac/remo/rflow/ontology/tbox/yawl-core.owl.ttl` (15855 bytes)
- `/Users/sac/remo/rforge/generated/ontology.ttl` (844 bytes)
- `/Users/sac/remo/src/architecture/ontology/remo.ttl` (12443 bytes)
- `/Users/sac/remo/src/ontology/remo-ontology.ttl` (9999 bytes)
- `/Users/sac/remo/src/ontology/remo.ttl` (8335 bytes)
- `/Users/sac/remo/src/remo/pkg/weaver/schema.ttl` (11317 bytes)
- `/Users/sac/remo/src/remogen/test/fixtures/sample_ontology.ttl` (8873 bytes)

</details>
