# Research Dossier: `jotp`

**Total Files:** 12 Ontologies (.ttl) | 11 Queries (.rq)
**Total Volume:** 23 files

## 1. Core Vocabularies (Prefixes)
- `app: <https://jgen.dev/ontology/java/enterprise#>`
- `eip: <https://jgen.dev/ontology/java/messaging#>`
- `ex: <https://example.org/telemetry#>`
- `inst: <https://jgen.dev/instance/>`
- `java: <https://jgen.dev/ontology/java#>`
- `jconc: <https://jgen.dev/ontology/java/concurrency#>`
- `jcore: <https://jgen.dev/ontology/java/core#>`
- `jmig: <https://jgen.dev/ontology/java/migration#>`
- `jmod: <https://jgen.dev/ontology/java/modules#>`
- `jpat: <https://jgen.dev/ontology/java/patterns#>`
- `jproj: <https://jgen.dev/ontology/java/project#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `saga: <https://jgen.dev/ontology/java/saga#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `app:ApiGatewayPattern`
- `app:ApplicationPattern`
- `app:CQRSPattern`
- `app:EventSourcingPattern`
- `app:Infrastructure`
- `app:MessagingInfrastructurePattern`
- `app:ObservabilityPattern`
- `app:Service`
- `app:ServiceMeshPattern`
- `eip:FoundationPattern`
- `eip:JOTPPrimitive`
- `eip:MessagingPattern`
- `eip:OrchestrationPattern`
- `eip:ResiliencePattern`
- `eip:RoutingPattern`
- `java:AbstractClass`
- `java:AnnotationType`
- `java:ClassType`
- `java:Constructor`
- `java:Enum`
- `java:Feature`
- `java:Field`
- `java:FinalClass`
- `java:FunctionalInterface`
- `java:InterfaceType`
- `java:JavaVersion`
- `java:Member`
- `java:Method`
- `java:Modifier`
- `java:Parameter`
- `java:PrimitiveType`
- `java:Record`
- `java:RecordComponent`
- `java:ReferenceType`
- `java:SealedClass`
- `java:SealedInterface`
- `java:Service`
- `java:Testable`
- `java:Type`
- `java:TypeParameter`
- `java:VoidType`
- `jconc:ConcurrencyPattern`
- `jconc:ExecutorPattern`
- `jconc:ScopedValuePattern`
- `jconc:StructuredConcurrencyPattern`
- `jconc:ThreadPattern`
- `jmig:MigrationRule`
- `jmod:AutomaticModule`
- `jmod:Directive`
- `jmod:ExportsDirective`
- *...and 19 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 11}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?annotation`, `?artifactId`, `?category`, `?class`, `?comment`, `?complexity`, `?component`, `?componentName`, `?componentType`, `?element`, `?elementType`, `?feature`, `?field`, `?fieldName`, `?fieldType`, `?groupId`, `?hasCompactConstructor`, `?intent`, `?interface`, `?isBreaking`, `?label`, `?method`, `?methodName`, `?modifier`, `?name`, `?package`, `?packageName`, `?pattern`, `?patternName`, `?permittedKind`, `?permittedName`, `?permittedType`, `?primitive`, `?priority`, `?record`, `?replacesLegacy`, `?requiredName`, `?requiredPattern`, `?returnType`, `?rule`, `?sealedType`, `?source`, `?superclass`, `?target`, `?targetVersion`, `?template`, `?typeKind`, `?typeName`, `?typeParam`, `?value`, `?version`, `?versionName`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/jotp/examples/f1-telemetry-application.ttl` (21321 bytes)
- `/Users/sac/jotp/examples/race-telemetry-project.ttl` (4509 bytes)
- `/Users/sac/jotp/examples/telemetry-app.ttl` (3212 bytes)
- `/Users/sac/jotp/queries/extract-classes.rq` (938 bytes)
- `/Users/sac/jotp/queries/extract-migrations.rq` (782 bytes)
- `/Users/sac/jotp/queries/extract-patterns.rq` (1197 bytes)
- `/Users/sac/jotp/queries/extract-records.rq` (779 bytes)
- `/Users/sac/jotp/queries/extract-sealed-types.rq` (1336 bytes)
- `/Users/sac/jotp/queries/pattern-dependencies.rq` (877 bytes)
- `/Users/sac/jotp/queries/project-extract.rq` (3466 bytes)
- `/Users/sac/jotp/queries/v5/select-records.rq` (316 bytes)
- `/Users/sac/jotp/queries/v5/select-sealed-interfaces.rq` (317 bytes)
- `/Users/sac/jotp/queries/v5/select-services.rq` (305 bytes)
- `/Users/sac/jotp/queries/v5/select-testable.rq` (298 bytes)
- `/Users/sac/jotp/schema/java-concurrency.ttl` (7647 bytes)
- `/Users/sac/jotp/schema/java-core.ttl` (15018 bytes)
- `/Users/sac/jotp/schema/java-enterprise.ttl` (13293 bytes)
- `/Users/sac/jotp/schema/java-instances.ttl` (3637 bytes)
- `/Users/sac/jotp/schema/java-messaging.ttl` (12520 bytes)
- `/Users/sac/jotp/schema/java-migration.ttl` (13854 bytes)
- `/Users/sac/jotp/schema/java-modules.ttl` (5156 bytes)
- `/Users/sac/jotp/schema/java-patterns.ttl` (11290 bytes)
- `/Users/sac/jotp/schema/java-project.ttl` (6489 bytes)

</details>
