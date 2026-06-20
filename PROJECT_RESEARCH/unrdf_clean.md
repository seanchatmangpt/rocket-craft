# Research Dossier: `unrdf-clean`

**Total Files:** 53 Ontologies (.ttl) | 3 Queries (.rq)
**Total Volume:** 56 files

## 1. Core Vocabularies (Prefixes)
- `api: <https://example.org/blog-api/ontology#>`
- `auth: <urn:unrdf:doc:auth:>`
- `blog: <https://example.org/blog#>`
- `bu: <http://example.disney.com/bu#>`
- `cap: <urn:unrdf:doc:capability:>`
- `ce: <http://unrdf.org/chatman-equation#>`
- `ce: <urn:chatman:equation:>`
- `chatman-cmp: <http://unrdf.org/chatman/comparison/>`
- `chatman-eq: <http://unrdf.org/chatman/equation/>`
- `chatman-event: <http://unrdf.org/chatman/event/>`
- `chatman-rel: <http://unrdf.org/chatman/relationship/>`
- `chatman: <http://unrdf.org/chatman/>`
- `chatman: <urn:chatman:>`
- `cost: <urn:unrdf:doc:cost:>`
- `data: <http://example.org/data/>`
- `dc: <http://purl.org/dc/terms/>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dcterms: <http://purl.org/dc/terms/>`
- `disney: <http://example.disney.com/canon#>`
- `doc: <urn:unrdf:doc:>`
- `ex: <http://example.org/>`
- `ex: <http://example.org/instances#>`
- `ex: <http://example.org/schema#>`
- `ex: <http://papers-thesis.org/examples#>`
- `ex: <urn:chatman:examples:>`
- `ex: <urn:unrdf:doc:example:>`
- `fibo: <https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/FinancialInstruments/>`
- `foaf: <http://www.foaf-project.org/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `invalid: <http://example.org/invalid#>`
- `kgc: <http://kgc.unrdf.dev/ns#>`
- `kgc: <http://unrdf.org/kgc#>`
- `kgc: <https://unrdf.org/kgc/probe#>`
- `lineage: <urn:chatman:lineage:>`
- `nfo: <http://www.semanticdesktop.org/ontologies/2007/03/22/nfo#>`
- `nie: <http://www.semanticdesktop.org/ontologies/2007/01/19/nie#>`
- `oa: <http://www.w3.org/ns/oa#>`
- `odrl: <http://www.w3.org/ns/odrl/2/>`
- `org: <http://www.w3.org/ns/org#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `paper: <http://example.org/ontology/paper#>`
- `parliamentary: <urn:parliamentary:>`
- `playground: <http://example.org/playground#>`
- `policy: <http://example.org/policy#>`
- `proto: <urn:unrdf:doc:protocol:>`
- `prov: <http://www.w3.org/ns/prov#>`
- `pt: <http://papers-thesis.org/ontology#>`
- `quality: <urn:quality:>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- *...and 23 more.*

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `auth:AuthorizationLevel`
- `auth:Permission`
- `auth:Principal`
- `auth:Role`
- `cap:AtomicCapability`
- `cap:Capability`
- `cap:CompositeCapability`
- `cap:Condition`
- `cap:Effect`
- `cap:Invariant`
- `cap:Postcondition`
- `cap:Precondition`
- `cap:Resource`
- `ce:Artifact`
- `ce:BlueOceanOperator`
- `ce:ClosureOperator`
- `ce:Component`
- `ce:ComponentShape`
- `ce:ConfigurationFormat`
- `ce:DisruptionOperator`
- `ce:Methodology`
- `ce:MethodologyShape`
- `ce:Observation`
- `ce:OntologyFormat`
- `ce:StrategicPivotOperator`
- `ce:TemplateEngine`
- `ce:UnificationDomain`
- `ce:UnificationOperator`
- `chatman:AchievementShape`
- `chatman:FamilyRelationshipShape`
- `chatman:IntellectualLineageShape`
- `chatman:MathematicalConstantShape`
- `chatman:PersonShape`
- `chatman:ProvenanceShape`
- `chatman:ScientificComparisonShape`
- `chatman:ScientificEquationShape`
- `chatman:SoftwareProjectShape`
- `chatman:TimelineEventShape`
- `cost:CostEstimate`
- `cost:CostModel`
- `cost:ResourceCost`
- `entity`
- `ex:Comment`
- `ex:Document`
- `ex:DocumentShape`
- `ex:Organization`
- `ex:OrganizationShape`
- `ex:Person`
- `ex:PersonShape`
- `ex:Post`
- *...and 220 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 1, 'ASK': 2}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?amount`, `?currency`, `?initiator`, `?recipient`, `?timestamp`, `?transaction`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/unrdf-clean/docs/ontology/minimal-n3-routing.ttl` (12340 bytes)
- `/Users/sac/unrdf-clean/docs/schema/documentation-ontology.ttl` (48480 bytes)
- `/Users/sac/unrdf-clean/docs/schema/examples.ttl` (39490 bytes)
- `/Users/sac/unrdf-clean/docs/schema/shacl-validation.ttl` (40309 bytes)
- `/Users/sac/unrdf-clean/examples/hooks/financial/large-transaction.select.rq` (1222 bytes)
- `/Users/sac/unrdf-clean/examples/hooks/parliamentary/motion-compliance.ask.rq` (875 bytes)
- `/Users/sac/unrdf-clean/examples/hooks/quality/data-quality.shacl.ttl` (2140 bytes)
- `/Users/sac/unrdf-clean/examples/openapi/ontology/blog-api.ttl` (2777 bytes)
- `/Users/sac/unrdf-clean/examples/rdf-kgn/data/sample-data.ttl` (858 bytes)
- `/Users/sac/unrdf-clean/examples/rdf-kgn/data/sample-ontology.ttl` (1193 bytes)
- `/Users/sac/unrdf-clean/examples/rdf-kgn/data/sample-shapes.ttl` (1609 bytes)
- `/Users/sac/unrdf-clean/hooks/health-check.ask.rq` (196 bytes)
- `/Users/sac/unrdf-clean/ontologies/disney-governed-universe.ttl` (14631 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/data/achievements.ttl` (13353 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/data/lineage.ttl` (12982 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/examples/turtle/ontology.ttl` (2388 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/examples/turtle/shapes.ttl` (727 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/ontology/chatman.ttl` (21640 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/ontology/examples.ttl` (15514 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/ontology/shapes.ttl` (13192 bytes)
- `/Users/sac/unrdf-clean/packages/chatman-equation/shapes/chatman-shapes.ttl` (11278 bytes)
- `/Users/sac/unrdf-clean/packages/cli/examples/sync/ontology/schema.ttl` (2841 bytes)
- `/Users/sac/unrdf-clean/packages/cli/examples/sync/schema.ttl` (7446 bytes)
- `/Users/sac/unrdf-clean/packages/cli/test-data-cli-1766303513888-0bbdg8463e1f/graph1.ttl` (57 bytes)
- `/Users/sac/unrdf-clean/packages/cli/test-data-cli-1766303513888-0bbdg8463e1f/graph2.ttl` (57 bytes)
- `/Users/sac/unrdf-clean/packages/cli/test/fixtures/person.ttl` (588 bytes)
- `/Users/sac/unrdf-clean/packages/core/src/ontologies/unfs-ontology.ttl` (2991 bytes)
- `/Users/sac/unrdf-clean/packages/core/src/ontologies/unmetric-ontology.ttl` (3179 bytes)
- `/Users/sac/unrdf-clean/packages/core/src/ontologies/unproj-ontology.ttl` (5266 bytes)
- `/Users/sac/unrdf-clean/packages/kgc-probe/examples/example-output.ttl` (7891 bytes)
- `/Users/sac/unrdf-clean/packages/kgc-probe/final-test/observations.ttl` (699 bytes)
- `/Users/sac/unrdf-clean/packages/kgc-probe/src/vocabulary.ttl` (7165 bytes)
- `/Users/sac/unrdf-clean/packages/kgc-probe/test-output/observations.ttl` (699 bytes)
- `/Users/sac/unrdf-clean/playground/ontologies/examples.ttl` (24195 bytes)
- `/Users/sac/unrdf-clean/playground/ontologies/papers-thesis.ttl` (32197 bytes)
- `/Users/sac/unrdf-clean/playground/ontologies/playground-shapes.ttl` (1855 bytes)
- `/Users/sac/unrdf-clean/playground/ontologies/playground.ttl` (2466 bytes)
- `/Users/sac/unrdf-clean/playground/papers-thesis-cli/ontologies/examples.ttl` (8992 bytes)
- `/Users/sac/unrdf-clean/playground/papers-thesis-cli/ontologies/papers-thesis.ttl` (18370 bytes)
- `/Users/sac/unrdf-clean/playground/smoke-test/data.ttl` (652 bytes)
- `/Users/sac/unrdf-clean/schema/domain.ttl` (8374 bytes)
- `/Users/sac/unrdf-clean/schema/packages-discovered.ttl` (24808 bytes)
- `/Users/sac/unrdf-clean/schema/project-structure.ttl` (8264 bytes)
- `/Users/sac/unrdf-clean/schemas/unrdf-packages.ttl` (45271 bytes)
- `/Users/sac/unrdf-clean/src/ontologies/unfs-ontology.ttl` (2991 bytes)
- `/Users/sac/unrdf-clean/src/ontologies/unproj-ontology.ttl` (5266 bytes)
- `/Users/sac/unrdf-clean/test-data/persons.ttl` (467 bytes)
- `/Users/sac/unrdf-clean/test-data/rules.ttl` (221 bytes)
- `/Users/sac/unrdf-clean/test-data/shapes.ttl` (591 bytes)
- `/Users/sac/unrdf-clean/test-governance-cli/ontologies/registry.ttl` (1189 bytes)
- `/Users/sac/unrdf-clean/test-governance-cli/overlays/bu/invalid.delta.ttl` (743 bytes)
- `/Users/sac/unrdf-clean/test-governance-cli/overlays/bu/studios.delta.ttl` (1041 bytes)
- `/Users/sac/unrdf-clean/test-governance-cli/policies/system-policy.ttl` (1547 bytes)
- `/Users/sac/unrdf-clean/test/fixtures/test-universe.ttl` (1876 bytes)
- `/Users/sac/unrdf-clean/tools/test-detailed.ttl` (123 bytes)
- `/Users/sac/unrdf-clean/tools/test-warnings.ttl` (93 bytes)

</details>
