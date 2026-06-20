# Research Dossier: `java-maven-template`

**Total Files:** 64 Ontologies (.ttl) | 11 Queries (.rq)
**Total Volume:** 75 files

## 1. Core Vocabularies (Prefixes)
- `a2a: <http://yawl.io/a2a#>`
- `a2a: <http://yawlfoundation.org/a2a#>`
- `agent: <http://yawl.io/agent/2030#>`
- `app: <https://jgen.dev/ontology/java/enterprise#>`
- `code: <http://yawl.org/code#>`
- `data: <http://yawl.org/data#>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `dim: <http://yawl.org/dimension/>`
- `dm-jtbd: <http://yawlfoundation.org/yawl/datamodelling/jtbd#>`
- `dm: <http://yawlfoundation.org/yawl/datamodelling/bridge#>`
- `eip: <https://jgen.dev/ontology/java/messaging#>`
- `ex: <http://example.org/yawl-shapes#>`
- `ex: <http://yawl.org/agent/>`
- `ex: <https://example.org/telemetry#>`
- `exec: <http://yawl.org/execution#>`
- `extract: <http://yawl.io/extraction#>`
- `fibo-fbc: <https://spec.edmcouncil.org/fibo/ontology/FBC/>`
- `fibo-fnd: <https://spec.edmcouncil.org/fibo/ontology/FND/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `forensic: <http://yawl.io/forensic#>`
- `gdpr: <http://www.yawlfoundation.org/gdpr#>`
- `godspeed: <http://yawl.io/godspeed#>`
- `hipaa: <http://www.yawlfoundation.org/hipaa#>`
- `inst: <https://jgen.dev/instance/>`
- `inverse: <http://yawl.io/godspeed/inverse#>`
- `j11: <http://yawl.io/java/pattern/java11#>`
- `j25: <http://yawl.io/java/pattern/java25#>`
- `java: <http://yawl.io/java#>`
- `java: <https://jgen.dev/ontology/java#>`
- `jconc: <https://jgen.dev/ontology/java/concurrency#>`
- `jcore: <https://jgen.dev/ontology/java/core#>`
- `jmig: <https://jgen.dev/ontology/java/migration#>`
- `jmod: <https://jgen.dev/ontology/java/modules#>`
- `jpat: <https://jgen.dev/ontology/java/patterns#>`
- `jproj: <https://jgen.dev/ontology/java/project#>`
- `lineage: <http://yawl.org/lineage#>`
- `market: <http://yawl.io/marketplace/2030#>`
- `mcp: <http://yawl.io/mcp#>`
- `mcp: <http://yawlfoundation.org/mcp#>`
- `ns1: <https://yawl.io/sim#>`
- `opt: <http://yawl.org/optimization/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `pm: <http://yawlfoundation.org/yawl/processmining/bridge#>`
- `product: <http://yawl.io/product/2030#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `rule: <http://yawl.io/java/migration/rule#>`
- *...and 17 more.*

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:ANDGatewayShape`
- `:ANDJoinShipmentSync`
- `:AlignmentNotes`
- `:ArchitecturePattern`
- `:AtomicTaskShape`
- `:BackorderIDVariable`
- `:BackorderStatus`
- `:BuildPlugin`
- `:BuildProfile`
- `:CancellationRegionShape`
- `:CaseExecutionBundle`
- `:CompositeTaskShape`
- `:ConditionShape`
- `:Coverage`
- `:DataShape`
- `:DecompositionShape`
- `:Dependency`
- `:FinancialComplianceFramework`
- `:FlowShape`
- `:GatewayShape`
- `:InputTaskShape`
- `:IntegrationPoint`
- `:InventoryAgent`
- `:InventoryStatus`
- `:InventoryStatusVariable`
- `:InvoiceDocument`
- `:MarkingShape`
- `:Module`
- `:ORGatewayShape`
- `:Order`
- `:OrderAmountVariable`
- `:OrderFulfillmentProcess`
- `:OrderIDVariable`
- `:OutputTaskShape`
- `:ParameterShape`
- `:Payment`
- `:PaymentMethodVariable`
- `:PaymentProcessingTask`
- `:PaymentProcessorAgent`
- `:PaymentStatus`
- `:PaymentStatusVariable`
- `:ProcessTaskShape`
- `:Quality`
- `:TaskAssignmentDelegation`
- `:Test`
- `:TrackingNumberVariable`
- `:TransactionIDVariable`
- `:WorkflowExecutionTimeline`
- `:XORGatewayShape`
- `:XORSplitPaymentVsBackorder`
- *...and 291 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 11}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?annotation`, `?artifactId`, `?category`, `?class`, `?comment`, `?complexity`, `?component`, `?componentName`, `?componentType`, `?element`, `?elementType`, `?feature`, `?field`, `?fieldName`, `?fieldType`, `?groupId`, `?hasCompactConstructor`, `?intent`, `?interface`, `?isBreaking`, `?label`, `?method`, `?methodName`, `?modifier`, `?name`, `?package`, `?packageName`, `?pattern`, `?patternName`, `?permittedKind`, `?permittedName`, `?permittedType`, `?primitive`, `?priority`, `?record`, `?replacesLegacy`, `?requiredName`, `?requiredPattern`, `?returnType`, `?rule`, `?sealedType`, `?source`, `?superclass`, `?target`, `?targetVersion`, `?template`, `?typeKind`, `?typeName`, `?typeParam`, `?value`, `?version`, `?versionName`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/java-maven-template/examples/f1-telemetry-application.ttl` (21321 bytes)
- `/Users/sac/java-maven-template/examples/race-telemetry-project.ttl` (4509 bytes)
- `/Users/sac/java-maven-template/examples/telemetry-app.ttl` (3212 bytes)
- `/Users/sac/java-maven-template/queries/extract-classes.rq` (938 bytes)
- `/Users/sac/java-maven-template/queries/extract-migrations.rq` (782 bytes)
- `/Users/sac/java-maven-template/queries/extract-patterns.rq` (1197 bytes)
- `/Users/sac/java-maven-template/queries/extract-records.rq` (779 bytes)
- `/Users/sac/java-maven-template/queries/extract-sealed-types.rq` (1336 bytes)
- `/Users/sac/java-maven-template/queries/pattern-dependencies.rq` (877 bytes)
- `/Users/sac/java-maven-template/queries/project-extract.rq` (3466 bytes)
- `/Users/sac/java-maven-template/queries/v5/select-records.rq` (316 bytes)
- `/Users/sac/java-maven-template/queries/v5/select-sealed-interfaces.rq` (317 bytes)
- `/Users/sac/java-maven-template/queries/v5/select-services.rq` (305 bytes)
- `/Users/sac/java-maven-template/queries/v5/select-testable.rq` (298 bytes)
- `/Users/sac/java-maven-template/schema/java-concurrency.ttl` (7647 bytes)
- `/Users/sac/java-maven-template/schema/java-core.ttl` (15018 bytes)
- `/Users/sac/java-maven-template/schema/java-enterprise.ttl` (13293 bytes)
- `/Users/sac/java-maven-template/schema/java-instances.ttl` (3637 bytes)
- `/Users/sac/java-maven-template/schema/java-messaging.ttl` (12520 bytes)
- `/Users/sac/java-maven-template/schema/java-migration.ttl` (13854 bytes)
- `/Users/sac/java-maven-template/schema/java-modules.ttl` (5156 bytes)
- `/Users/sac/java-maven-template/schema/java-patterns.ttl` (11290 bytes)
- `/Users/sac/java-maven-template/schema/java-project.ttl` (6489 bytes)
- `/Users/sac/java-maven-template/yawl/native-bridge-registry-extensions.ttl` (2512 bytes)
- `/Users/sac/java-maven-template/yawl/native-bridge-with-registry.ttl` (19518 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/analysis/dimensions.ttl` (902 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/godspeed-mcp-tools.ttl` (23979 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/godspeed-protocol.ttl` (18588 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/inverse-godspeed-mcp-tools.ttl` (26704 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/inverse-godspeed-protocol.ttl` (22667 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/marketplace-2030.ttl` (25712 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/vision-2030-collaboration.ttl` (22287 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/godspeed/vision-2030-inverse-extraction.ttl` (30353 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/migration/java-code.ttl` (15119 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/migration/java11-patterns.ttl` (15554 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/migration/java25-patterns.ttl` (16928 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/migration/migration-rules.ttl` (17632 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/migration/yawl-migration-workflow.ttl` (19191 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/optimization/cycle-1-improvements.ttl` (565 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/optimization/cycle-2-improvements.ttl` (185 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/optimization/cycle-3-improvements.ttl` (185 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/process-mining/pm-bridge-generated.ttl` (29376 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/process-mining/pm-bridge.ttl` (21006 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/process.ttl` (19848 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/profiles/fibo-alignment.ttl` (19849 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/profiles/prov-alignment.ttl` (17940 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/profiles/schema-org-alignment.ttl` (11773 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/safe/safe-core.ttl` (6284 bytes)
- `/Users/sac/java-maven-template/yawl/ontology/simulation/yawl-sim.ttl` (6574 bytes)
- `/Users/sac/java-maven-template/yawl/optimal_pipeline_1772516280.ttl` (304 bytes)
- `/Users/sac/java-maven-template/yawl/optimal_pipeline_1772516314.ttl` (304 bytes)
- `/Users/sac/java-maven-template/yawl/optimal_pipeline_1772516363.ttl` (304 bytes)
- `/Users/sac/java-maven-template/yawl/pm-bridge-ggen/gen-ttl/test-output.ttl` (6452 bytes)
- `/Users/sac/java-maven-template/yawl/schema/shacl/yawl-compliance-gdpr-shapes.ttl` (13762 bytes)
- `/Users/sac/java-maven-template/yawl/schema/shacl/yawl-compliance-hipaa-shapes.ttl` (16617 bytes)
- `/Users/sac/java-maven-template/yawl/schema/shacl/yawl-compliance-sox-shapes.ttl` (972 bytes)
- `/Users/sac/java-maven-template/yawl/schema/yawl-lineage-ontology.ttl` (28053 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/ontology/ggen-observation.ttl` (12549 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/ontology/yawl-public-roots.ttl` (24328 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/shacl/yawl-core-shapes.ttl` (4817 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/shacl/yawl-element-shapes.ttl` (8282 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/shacl/yawl-net-shapes.ttl` (7970 bytes)
- `/Users/sac/java-maven-template/yawl/src/main/resources/shacl/yawl-workflow-shapes.ttl` (5477 bytes)
- `/Users/sac/java-maven-template/yawl/stress-test/src/main/resources/shacl/yawl-core-shapes.ttl` (3645 bytes)
- `/Users/sac/java-maven-template/yawl/stress-test/src/main/resources/shacl/yawl-element-shapes.ttl` (8700 bytes)
- `/Users/sac/java-maven-template/yawl/stress-test/src/main/resources/shacl/yawl-net-shapes.ttl` (4888 bytes)
- `/Users/sac/java-maven-template/yawl/stress-test/src/main/resources/shacl/yawl-workflow-shapes.ttl` (3974 bytes)
- `/Users/sac/java-maven-template/yawl/tests/orderfulfillment.ttl` (19848 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-data-modelling/dm-bridge-ggen/ontology/dm-bridge.ttl` (79937 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-data-modelling/dm-bridge-ggen/ontology/dm-jtbd.ttl` (12586 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-elements/schema/shacl/yawl-compliance-gdpr-shapes.ttl` (13762 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-elements/schema/shacl/yawl-compliance-hipaa-shapes.ttl` (16617 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-elements/schema/shacl/yawl-compliance-sox-shapes.ttl` (972 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-elements/schema/yawl-lineage-ontology.ttl` (28053 bytes)
- `/Users/sac/java-maven-template/yawl/yawl-ggen/src/main/resources/yawl-mined-ontology.ttl` (6118 bytes)

</details>
