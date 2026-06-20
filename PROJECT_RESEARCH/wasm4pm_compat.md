# Research Dossier: `wasm4pm-compat`

**Total Files:** 21 Ontologies (.ttl) | 52 Queries (.rq)
**Total Volume:** 73 files

## 1. Core Vocabularies (Prefixes)
- `DECLARATIONS
# ============================================================================

@prefix compat: <https://wasm4pm-compat.rs/ontology#>`
- `audit: <https://wasm4pm-compat.rs/audit#>`
- `bpmn: <https://open-ontologies.org/bpmn#>`
- `bpmn: <https://www.omg.org/spec/BPMN/20100524/MODEL#>`
- `compat: <https://wasm4pm-compat.rs/ontology#>`
- `compat: <https://wasm4pm.dev/ns#>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `domain: <https://wasm4pm-compat.rs/domain#>`
- `ex: <http://example.org/>`
- `metrics: <https://open-ontologies.org/metrics#>`
- `ocel: <https://ocel-standard.org/ontology#>`
- `ocel: <https://open-ontologies.org/ocel#>`
- `ocpn: <https://open-ontologies.org/ocpn#>`
- `open: <https://open-ontologies.github.io/>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `paper: <https://wasm4pm-compat.rs/paper#>`
- `petri: <https://open-ontologies.org/petri#>`
- `pi: <https://wasm4pm.org/process-intelligence#>`
- `pm4py: <https://open-ontologies.org/pm4py#>`
- `pm: <http://www.purl.org/pm/ontology/2023/05#>`
- `prov: <http://www.w3.org/ns/prov#>`
- `ptree: <https://open-ontologies.org/processtree#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-syntax-ns#>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `substrate: <https://wasm4pm-compat.rs/substrate#>`
- `wasm4pm: <http://wasm4pm-compat.org/>`
- `wfnet: <https://open-ontologies.org/wfnet#>`
- `xes: <http://www.xes-standard.org/ontology#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`
- `zod: <https://wasm4pm-compat.rs/zod#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `audit:AllowedContext`
- `audit:AuditExecutable`
- `audit:AuditGate`
- `audit:AuditSpec`
- `audit:AuditTemplate`
- `audit:AuxiliaryCommit`
- `audit:CheckpointClaim`
- `audit:ClosureClaim`
- `audit:CommitEvidence`
- `audit:ForbiddenPattern`
- `audit:Gap`
- `audit:HostileAssumption`
- `compat:CognitionBreed`
- `compat:CompileFailLaw`
- `compat:CompilePassSurface`
- `compat:ConformanceAuthority`
- `compat:EvidenceState`
- `compat:GraduationBoundary`
- `compat:GraduationSurface`
- `compat:LifecycleAuthority`
- `compat:LossReportShape`
- `compat:MiningAuthority`
- `compat:NamedLossShape`
- `compat:PaperCoverage`
- `compat:ProcessForm`
- `compat:ProcessTreeOperator_LoopShape`
- `compat:ProcessTreeOperator_OrShape`
- `compat:ProcessTreeOperator_ParallelShape`
- `compat:ProcessTreeOperator_SequenceShape`
- `compat:ProcessTreeOperator_SilentShape`
- `compat:ProcessTreeOperator_XorShape`
- `compat:ProcessTreeRefusalShape`
- `compat:ProcessTreeStructureShape`
- `compat:ProjectionNameShape`
- `compat:ReplayAuthority`
- `compat:SourceModule`
- `compat:StateTransition`
- `compat:TreeProjectableShape`
- `compat:TypeConstraint`
- `compat:TypedLoopNodeShape`
- `compat:WitnessFamily`
- `compat:WitnessMarker`
- `domain:Admission`
- `domain:AdmissionBoundary`
- `domain:AdmissionVerdict`
- `domain:AdmittedEvidence`
- `domain:ApiGrammarWitness`
- `domain:Arc`
- `domain:BpmnEvent`
- `domain:BpmnGateway`
- *...and 94 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'CONSTRUCT': 9, 'SELECT': 40, 'ASK': 3}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?acceptedAsDebt`, `?alignmentBasis`, `?alignmentClass`, `?alignmentDistance`, `?auditScope`, `?authority`, `?bpmnElement`, `?bpmnElementLabel`, `?bpmnElementType`, `?breed_doc`, `?breed_id`, `?breed_label`, `?breed_status`, `?carrierType`, `?categoryId`, `?categoryLabel`, `?citation`, `?citeKey`, `?class`, `?classification`, `?closingCommit`, `?closureClaim`, `?closureMethod`, `?commitAuthor`, `?commitHash`, `?commitMessage`, `?commitTimestamp`, `?compatDenomRestriction`, `?compatDxSurface`, `?compatForm`, `?compatMaxBound`, `?compatMinBound`, `?compatNumRestriction`, `?compatRustType`, `?compatSourceFile`, `?compat_class`, `?contextComment`, `?contextName`, `?contextScope`, `?decoy_reason`, `?description`, `?errorCode`, `?exceptionReason`, `?externalDef`, `?externalFormulaOrReference`, `?externalLabel`, `?externalMetricDef`, `?externalMetricLabel`, `?externalNamespace`, `?externalNamespaceLabel`, `?externalPrefix`, `?externalURI`, `?externalValidationRule`, `?fieldName`, `?fieldType`, `?forbiddenByUri`, `?forbiddenPatternName`, `?form`, `?fresh_name`, `?gapCategory`, `?gapId`, `?gapSeverity`, `?gate`, `?gateComment`, `?gateCondition`, `?gateDescription`, `?gateLabel`, `?gateName`, `?gateSeverity`, `?gatewayType`, `?graduatesToWasm4pm`, `?hardcode_lockable`, `?hasClosureClaim`, `?isAligned`, `?isArray`, `?isEnum`, `?isOptional`, `?isSoundWfNet`, `?isUnion`, `?key`, `?law`, `?lawName`, `?mapping_reason`, `?marker`, `?metricForm`, `?metricKind`, `?module`, `?moduleName`, `?module_file`, `?module_path`, `?objectCentricVariant`, `?ocelComponent`, `?ocelComponentType`, `?output_json_path`, `?paper`, `?paperCitation`, `?paperReference`, `?parentCommit`, `?patternComment`, `?patternDescription`, `?patternName`, `?petriComponent`, `?petriComponentLabel`, `?petriComponentType`, `?pointer_derivation`, `?pointer_kind`, `?pointer_locus`, `?pointer_value`, `?reason`, `?receipt`, `?relatedCarrierType`, `?relatedLawName`, `?relatedSpecName`, `?relatedSpecScope`, `?remediationPlan`, `?rustType`, `?rust_variant`, `?scopeTarget`, `?soundnessConstraint`, `?sourceFile`, `?sourceTtl`, `?spec`, `?specComment`, `?specName`, `?standard`, `?state`, `?stateRequired`, `?status`, `?successor`, `?templateComment`, `?templateKind`, `?templateName`, `?treeComponent`, `?treeComponentLabel`, `?treeComponentType`, `?typeName`, `?verdictType`, `?witnessFamily`, `?witnessKey`, `?witnessRequired`, `?witnessTitle`, `?witnessYear`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/wasm4pm-compat/ggen/ontology-breeds/breed-vocabulary.ttl` (30285 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology-breeds/paper-pointers.ttl` (58473 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/audit-machinery.ttl` (46511 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/domain-evidence-structure.ttl` (16179 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/domain-graduation-boundaries.ttl` (18671 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/domain-process-forms.ttl` (21990 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/domain-type-constraints.ttl` (19856 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/ggen-substrate.ttl` (13070 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/papers.ttl` (22198 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/process-intelligence.ttl` (10270 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/wasm4pm-compat-integrated.ttl` (13880 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/wasm4pm-compat.ttl` (77177 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/witnesses-ai-llm.ttl` (49253 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/witnesses-cognition.ttl` (26604 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/witnesses-domain.ttl` (63312 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/witnesses-rdf.ttl` (41061 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/witnesses-workflow.ttl` (10557 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/zod-merged.ttl` (266238 bytes)
- `/Users/sac/wasm4pm-compat/ggen/ontology/zod-types.ttl` (13598 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries-breeds/extract-breed-witnesses.rq` (1372 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries-breeds/extract-breeds.rq` (677 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries-breeds/extract-fresh-names.rq` (325 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries-breeds/extract-paper-pointers.rq` (952 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/audit-critical-gaps-have-remediation.rq` (3409 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/audit-gap-closure-claims-have-gap-id.rq` (1434 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/audit-no-commit-count-gate.rq` (2080 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/audit-no-file-count-alive.rq` (1538 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-alive-gate.rq` (431 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-echo.rq` (99 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-graph.rq` (41 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-integrated-ontology.rq` (4403 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-minimal.rq` (50 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/construct-pass-through.rq` (188 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/derive-module-docs.rq` (261 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/derive-paper-coverage.rq` (248 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/derive-witness-metadata.rq` (258 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-all-witness-keys.rq` (736 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-blocking-audits.rq` (3852 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-bpmn-compat-join.rq` (2958 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-compile-fail-laws.rq` (679 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-conformance-authority.rq` (1159 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-conformance-metrics-bridge.rq` (3365 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-graduation-candidates.rq` (738 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-lifecycle-authority.rq` (1141 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-mining-authority.rq` (1078 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-ocel-compat-join.rq` (2544 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-paper-coverage.rq` (693 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-petri-compat-join.rq` (3382 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-process-forms.rq` (740 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-process-tree-compat-join.rq` (2719 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-replay-authority.rq` (1077 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-source-modules.rq` (989 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-states.rq` (627 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witness-to-external-mapping.rq` (4226 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-ai-llm.rq` (835 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-cognition.rq` (846 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-domain.rq` (861 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-full.rq` (1067 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-rdf.rq` (826 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses-workflow.rq` (851 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-witnesses.rq` (685 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/extract-zod-schemas.rq` (798 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/integrate-open-ontologies.rq` (2917 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/open-ontologies-bridge-federation.rq` (4041 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/open-ontologies-bridge.rq` (4369 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/select-allowed-contexts.rq` (752 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/select-audit-specs.rq` (3676 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/select-commit-gap-map.rq` (4686 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/select-forbidden-patterns.rq` (828 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/select-gap-ledger.rq` (2262 bytes)
- `/Users/sac/wasm4pm-compat/ggen/queries/validate-bridge-alignment.rq` (5503 bytes)
- `/Users/sac/wasm4pm-compat/ggen/shapes/loss-accounting.shacl.ttl` (8012 bytes)
- `/Users/sac/wasm4pm-compat/ggen/shapes/process-tree.shacl.ttl` (12239 bytes)

</details>
