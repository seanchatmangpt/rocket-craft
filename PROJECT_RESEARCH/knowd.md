# Research Dossier: `knowd`

**Total Files:** 13 Ontologies (.ttl) | 3 Queries (.rq)
**Total Volume:** 16 files

## 1. Core Vocabularies (Prefixes)
- `api: <http://knowd.io/api#>`
- `config: <http://knowd.io/config#>`
- `dc: <http://purl.org/dc/terms/>`
- `ex: <http://example.org/>`
- `ex: <https://coach.local/#>`
- `feat: <http://knowd.io/features#>`
- `feature: <http://knowd.io/features#>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `http: <http://www.w3.org/2011/http#>`
- `hydra: <http://www.w3.org/ns/hydra/core#>`
- `knowd: <http://knowd.io/ontology#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `schema: <http://schema.org/>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `api:Endpoint`
- `api:EndpointShape`
- `api:HTTPMethodShape`
- `api:OpenAPIFragmentShape`
- `api:PathFormatShape`
- `api:RateLimitShape`
- `api:RequestBody`
- `api:RequestBodyShape`
- `api:ResponseBody`
- `api:ResponseBodyShape`
- `config:AnalyzeSampleShape`
- `config:BooleanValueShape`
- `config:ClusterModeShape`
- `config:DirectoryPathShape`
- `config:EncryptionConsistencyShape`
- `config:EncryptionKeySourceShape`
- `config:IntegerValueShape`
- `config:PlanCacheSizeShape`
- `config:PortShape`
- `config:ServerSetting`
- `config:ServerSettingShape`
- `config:StoreTypeShape`
- `ex:AuditTrailShape`
- `ex:ConsentShape`
- `ex:HighValueTransactionShape`
- `ex:PersonShape`
- `ex:TransactionShape`
- `feat:ComplianceFeatureShape`
- `feat:DependencyShape`
- `feat:LLMDependencyShape`
- `feat:PerformanceFeatureShape`
- `feat:ProductionFeature`
- `feat:ProductionFeatureShape`
- `feat:ProductionReadinessReport`
- `feat:ProductionReadyShape`
- `feat:SecurityFeatureShape`
- `feat:Status`
- `feat:StatusProgressionShape`
- `feat:TestCoverageShape`
- `knowd:APIEndpoint`
- `knowd:APIEndpointShape`
- `knowd:APIFeature`
- `knowd:ComplianceFeature`
- `knowd:ConfigurationResource`
- `knowd:ConfigurationResourceShape`
- `knowd:DependencyConstraint`
- `knowd:Deployment`
- `knowd:ExperimentalFeature`
- `knowd:Feature`
- `knowd:Hook`
- *...and 20 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 3}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?completeness`, `?coverage`, `?depCompleteness`, `?depName`, `?depStatus`, `?dependency`, `?feature`, `?issue`, `?name`, `?status`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/knowd/apps/interview-coach/data/interview-knowledge.ttl` (7687 bytes)
- `/Users/sac/knowd/examples/policy-packs/enterprise-governance-v1/shapes/financial_shape.ttl` (2157 bytes)
- `/Users/sac/knowd/examples/policy-packs/enterprise-governance-v1/shapes/pii_shape.ttl` (1546 bytes)
- `/Users/sac/knowd/ontology/api-shapes.ttl` (3288 bytes)
- `/Users/sac/knowd/ontology/api.ttl` (9211 bytes)
- `/Users/sac/knowd/ontology/config-shapes.ttl` (8153 bytes)
- `/Users/sac/knowd/ontology/config.ttl` (7289 bytes)
- `/Users/sac/knowd/ontology/features-shapes.ttl` (6346 bytes)
- `/Users/sac/knowd/ontology/features.ttl` (9401 bytes)
- `/Users/sac/knowd/ontology/knowd.ttl` (11157 bytes)
- `/Users/sac/knowd/ontology/queries/dependency-queries.rq` (4516 bytes)
- `/Users/sac/knowd/ontology/queries/release-queries.rq` (4927 bytes)
- `/Users/sac/knowd/ontology/queries/status-queries.rq` (3789 bytes)
- `/Users/sac/knowd/ontology/shapes.ttl` (15882 bytes)
- `/Users/sac/knowd/testdata/example.ttl` (289 bytes)
- `/Users/sac/knowd/testdata/sample.ttl` (107 bytes)

</details>
