# Research Dossier: `test-unjucks-nuxt4`

**Total Files:** 53 Ontologies (.ttl) | 0 Queries (.rq)
**Total Volume:** 53 files

## 1. Core Vocabularies (Prefixes)
- `access: <http://unjucks.dev/access/>`
- `api: <http://example.org/api#>`
- `api: <http://example.org/api/>`
- `api: <http://unjucks.dev/api/>`
- `audit: <http://enterprise.example.com/audit#>`
- `audit: <http://unjucks.dev/audit/>`
- `basel: <http://www.bis.org/basel3/ontology/>`
- `blockchain: <http://blockchain.org/voc/>`
- `cbv: <http://gs1.org/cbv/>`
- `compliance: <http://example.org/compliance#>`
- `compliance: <http://standards.org/compliance/>`
- `compliance: <http://unjucks.dev/compliance/>`
- `consent: <http://unjucks.dev/consent/>`
- `data: <http://unjucks.dev/data/>`
- `dc: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `declaration
@prefix : <ht://invalid-scheme>`
- `doap: <http://usefulinc.com/ns/doap#>`
- `documentation: <http://unjucks.dev/documentation/>`
- `enterprise: <http://example.org/enterprise#>`
- `enterprise: <http://unjucks.dev/enterprise/>`
- `epcis: <http://gs1.org/epcis/>`
- `evil: <javascript:alert('XSS')>`
- `ex: <http://example.org/>`
- `ex: <http://example.org/ontology#>`
- `ex: <http://example.org/schema/>`
- `ex: <https://example.org/>`
- `fhir: <http://hl7.org/fhir/>`
- `fibo-be-le-lp: <https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/LegalPersons/>`
- `fibo-be-le: <https://spec.edmcouncil.org/fibo/ontology/BE/LegalEntities/>`
- `fibo-der-drc-bsc: <https://spec.edmcouncil.org/fibo/ontology/DER/DerivativesContracts/DerivativesBasics/>`
- `fibo-fbc-de: <https://spec.edmcouncil.org/fibo/ontology/FBC/DebtAndEquities/>`
- `fibo-fbc-fi: <https://spec.edmcouncil.org/fibo/ontology/FBC/FinancialInstruments/FinancialInstruments/>`
- `fibo-fnd-rel-rel: <https://spec.edmcouncil.org/fibo/ontology/FND/Relations/Relations/>`
- `fibo-sec-sec-bsk: <https://spec.edmcouncil.org/fibo/ontology/SEC/Securities/Baskets/>`
- `fibo: <https://spec.edmcouncil.org/fibo/ontology/>`
- `file: <file:///etc/>`
- `financial: <http://unjucks.dev/financial/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `gdpr: <http://compliance.enterprise.org/gdpr/>`
- `generator: <http://unjucks.dev/generator/>`
- `governance: <http://example.org/governance#>`
- `governance: <http://unjucks.dev/governance/>`
- `gs1: <http://gs1.org/voc/>`
- `http: <http://www.w3.org/2011/http#>`
- `hydra: <http://www.w3.org/ns/hydra/core#>`
- `loinc: <http://loinc.org/rdf#>`
- `monitoring: <http://unjucks.dev/monitoring/>`
- `onto: <http://example.org/ontology/>`
- `openapi: <http://openapi.org/spec#>`
- *...and 23 more.*

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:DataModelShape`
- `:DeleteEndpointRule`
- `:FieldShape`
- `:ParameterShape`
- `:PostEndpointRule`
- `:PutEndpointRule`
- `:RestAPIShape`
- `:RestEndpointShape`
- `:User`
- `:UserShape`
- `GeneratorShape`
- `OrganizationShape`
- `PersonShape`
- `TemplateShape`
- `VariableShape`
- `api:API`
- `api:ApiEndpoint`
- `api:Deprecated`
- `api:Development`
- `api:GraphQlApi`
- `api:GrpcApi`
- `api:LifecycleState`
- `api:PartnerApi`
- `api:PrivateApi`
- `api:PublicApi`
- `api:Published`
- `api:RateLimiting`
- `api:RestApi`
- `api:Retired`
- `api:Service`
- `api:Testing`
- `api:Throttling`
- `api:WebhookApi`
- `audit:AccessLog`
- `audit:AuditTrail`
- `compliance:ComplianceFramework`
- `compliance:GDPRConstraints`
- `compliance:PCIConstraints`
- `compliance:SOXConstraints`
- `data:BiometricData`
- `data:Collection`
- `data:GeneticData`
- `data:HealthData`
- `data:PersonalData`
- `data:Processing`
- `data:SensitivePersonalData`
- `data:Storage`
- `data:Transfer`
- `enterprise:AuditTemplate`
- `enterprise:ComplianceTemplate`
- *...and 79 more.*

## 3. Extraction Layer (SPARQL)
- *No queries executed in this project.*

### Projected Variables (SELECT ?var)
- *No specific projection variables identified.*

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/test-unjucks-nuxt4/unjucks-source/_templates/enterprise/data/schemas/api-standards.ttl` (1497 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/_templates/enterprise/data/schemas/compliance-requirements.ttl` (1779 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/_templates/nuxt-openapi/config/openapi-schema.ttl` (13366 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/01-basic-generation/data/api-schema.ttl` (3616 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/02-validation/data/invalid-data.ttl` (3587 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/02-validation/data/valid-data.ttl` (3217 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/02-validation/data/validation-rules.ttl` (6974 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/03-enterprise/data/enterprise-ontology.ttl` (27347 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/generated-hr-types/reverse.ttl` (1338 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/generated/reverse-generated.ttl` (2468 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/examples/sample-ontology.ttl` (4124 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/schema/user.ttl` (1029 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/src/semantic/ontologies/enterprise-template-ontology.ttl` (16043 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/src/semantic/schemas/api-governance.ttl` (10092 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/src/semantic/schemas/gdpr-compliance.ttl` (7648 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/src/semantic/schemas/sox-compliance.ttl` (4471 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/templates/_templates/fortune5/registry.ttl` (6839 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/test-ontology.ttl` (844 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/api-data.ttl` (4883 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/fortune5/cvs-health/patient-records.ttl` (6551 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/fortune5/jpmorgan/financial-instruments.ttl` (7631 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/fortune5/walmart/product-catalog.ttl` (9023 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/fortune5/walmart/supply-chain-events.ttl` (11311 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/performance/large-api-ontology.ttl` (467907 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/performance/massive-enterprise-graph.ttl` (2700161 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/rdf/validation/complex-schema.ttl` (2809 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/sample-ontology.ttl` (1243 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/semantic/financial/fibo-instruments.ttl` (5439 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/semantic/healthcare/fhir-patient-data.ttl` (4293 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/semantic/supply-chain/gs1-product-catalog.ttl` (7436 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/api-ontology.ttl` (235 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/basic-person.ttl` (599 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/complex-project.ttl` (1728 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/complex-schema.ttl` (1961 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/compliance-ontology.ttl` (290 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/edge-cases.ttl` (1090 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/enterprise-schema.ttl` (18318 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/fortune5-compliance.ttl` (487 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/invalid-syntax.ttl` (193 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/large-dataset.ttl` (6437 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/malicious.ttl` (1921 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/ontology.ttl` (3954 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/performance/complex-schema.ttl` (19077 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/performance/large-10000.ttl` (378255 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/performance/medium-1000.ttl` (36255 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/performance/small-100.ttl` (3630 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/sample-ontology.ttl` (1538 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/sample.ttl` (2671 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/security-ontology.ttl` (346 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/fixtures/turtle/shacl-validation.ttl` (2764 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/sample-data/test.ttl` (623 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/sample-ontology.ttl` (1463 bytes)
- `/Users/sac/test-unjucks-nuxt4/unjucks-source/tests/security/fixtures/malicious-patterns.ttl` (1334 bytes)

</details>
