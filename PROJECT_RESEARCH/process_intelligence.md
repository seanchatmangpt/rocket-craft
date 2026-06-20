# Research Dossier: `process-intelligence`

**Total Files:** 27 Ontologies (.ttl) | 77 Queries (.rq)
**Total Volume:** 104 files

## 1. Core Vocabularies (Prefixes)
- `aka: <http://process-intelligence.local/autonomic/>`
- `api: <http://process-intelligence.org/api/>`
- `art: <https://process.intelligence/artifact/>`
- `audit: <https://process.intelligence/audit-law/>`
- `bibo: <http://purl.org/ontology/bibo/>`
- `br: <http://process-intelligence.local/blue-river/>`
- `chk: <https://process.intelligence/checkpoint/>`
- `compat: <https://process.intelligence/compat/>`
- `conf: <https://process.intelligence/conformance/>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dct: <http://purl.org/dc/terms/>`
- `dcterms: <http://purl.org/dc/terms/>`
- `fcl: <https://process.intelligence/forbidden-collapse/>`
- `gen: <https://process.intelligence/generation-rule/>`
- `ggen: <https://codegen.org/ggen/>`
- `gov: <http://process-intelligence.local/governance/>`
- `grad: <https://process.intelligence/graduation/>`
- `grun: <https://process.intelligence/ggen-unified-run/>`
- `inv: <https://process.intelligence/invalid-ggen/>`
- `lifecycle: <https://process.intelligence/lifecycle/>`
- `ma: <https://process.intelligence/ma/>`
- `mape: <http://process-intelligence.local/mape-k/>`
- `ostar: <urn:ostar:ontology#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `pi: <http://process-intelligence.org/pi/>`
- `pi: <https://process.intelligence/ontology/>`
- `pm: <https://pi-research.dev/ontology/prompt-manufactory#>`
- `proj: <https://process.intelligence/project/>`
- `prov: <http://www.w3.org/ns/prov#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `schema: <https://schema.org/>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `skos: <http://www.w3.org/2004/02/skos/core#>`
- `src: <https://process.intelligence/source/>`
- `wasm4pm: <https://process.intelligence/wasm4pm/>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `aka:AutonomicPolicy`
- `aka:ComplianceTransition`
- `aka:DeclareConstraint`
- `aka:ElasticTransition`
- `aka:SandboxIsolation`
- `aka:TypestateCompilation`
- `audit:SHAPE_001`
- `audit:SHAPE_002`
- `audit:SHAPE_003`
- `audit:SHAPE_004`
- `audit:SHAPE_005`
- `audit:SHAPE_006`
- `audit:SHAPE_007`
- `audit:SHAPE_008`
- `audit:SHAPE_009`
- `audit:SHAPE_010`
- `audit:SHAPE_011`
- `audit:SHAPE_012`
- `audit:SHAPE_013`
- `audit:SHAPE_014`
- `audit:SHAPE_015`
- `compat:Evidence`
- `gov:Authority`
- `gov:ComplianceBoundary`
- `gov:ComplianceOverride`
- `gov:ConformanceDeviation`
- `gov:GovernancePolicy`
- `gov:LTLSafetyInvariant`
- `gov:PolicyMutation`
- `gov:Procedure`
- `lifecycle:AnalyzeRule`
- `lifecycle:AutonomicPolicy`
- `lifecycle:ConformancePattern`
- `lifecycle:DecommissionState`
- `lifecycle:DesignState`
- `lifecycle:ExecuteAction`
- `lifecycle:KnowledgeAsset`
- `lifecycle:MonitorRule`
- `lifecycle:MonitoringState`
- `lifecycle:OptimizationState`
- `lifecycle:PlanRule`
- `lifecycle:PredictiveModel`
- `lifecycle:ProcessModel`
- `lifecycle:ProcessState`
- `lifecycle:RemediationStrategy`
- `lifecycle:RepairState`
- `lifecycle:SimulationState`
- `lifecycle:StateTransition`
- `lifecycle:TransitionGuard`
- `lifecycle:ValidationState`
- *...and 132 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 63, 'ASK': 14}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?activityBottleneck`, `?activityName`, `?admissible_states`, `?agent`, `?agentLabel`, `?agentMission`, `?algorithm`, `?aliveClaim`, `?analyzeExpression`, `?analyzeRule`, `?analyzeThreshold`, `?artifact`, `?artifactType`, `?audience`, `?authRequired`, `?authScheme`, `?authority`, `?authorizedDownstream`, `?authorizedProject`, `?backedByLog`, `?bearerFormat`, `?blockingGap`, `?blockingIssue`, `?burst`, `?cell`, `?checkpoint`, `?checkpointLabel`, `?checkpointMission`, `?checkpointTitle`, `?claim`, `?claimCategory`, `?claimDescription`, `?claimLabel`, `?claimType`, `?classification`, `?collapse`, `?compat`, `?count`, `?courtType`, `?created`, `?crossing_name`, `?definition`, `?description`, `?endpoint`, `?estimatedFixHours`, `?evidence`, `?evidenceLink`, `?executeAction`, `?executeActionName`, `?executeAuditLog`, `?failedGates`, `?failingGate`, `?failureImpact`, `?failureIssue`, `?failureLocation`, `?feedstock`, `?file`, `?fileSize`, `?filename`, `?forbiddenCollapse`, `?forbiddenSurface`, `?form_name`, `?formalism`, `?gateCriteria`, `?gatesCriteriaMet`, `?gatesMet`, `?gatesTotal`, `?hookComment`, `?hookId`, `?hookLabel`, `?hookMission`, `?issued`, `?knowledgeAsset`, `?knowledgeAssetType`, `?knowledgeAssetValue`, `?label`, `?lattice_position`, `?logFormat`, `?method`, `?metric`, `?metricThreshold`, `?metricUnit`, `?metricValue`, `?mission`, `?monitorExpression`, `?monitorMetric`, `?monitorRule`, `?name`, `?operationId`, `?operationalDebtIfApplicable`, `?orchestrator`, `?outputContract`, `?outputFile`, `?outputMode`, `?outputPath`, `?ownedSurface`, `?ownerProgram`, `?partialClaim`, `?path`, `?phase`, `?phaseLabel`, `?phaseMission`, `?planOutputShape`, `?planPolicyExpression`, `?planRule`, `?position`, `?program`, `?programId`, `?project`, `?projectId`, `?projectLabel`, `?promptClass`, `?promptId`, `?promptType`, `?proofCell`, `?purpose`, `?quantifiedMetric`, `?query`, `?queryType`, `?rateLimitTier`, `?reason`, `?receipt`, `?receiptHash`, `?receiptTimestamp`, `?referenced`, `?relatedActivity`, `?remediationEffortHours`, `?remediationNote`, `?remediationPath`, `?replayTrace`, `?requestBodySchema`, `?responseBodySchema`, `?riskSeverity`, `?role`, `?roleId`, `?roleLabel`, `?roleMission`, `?rph`, `?rpm`, `?rps`, `?rule`, `?ruleCount`, `?ruleTitle`, `?rustified_marker`, `?rustified_signature`, `?scheme`, `?skillId`, `?skillLabel`, `?skillMission`, `?skillRefusal`, `?sound_marking`, `?sourceInstance`, `?sourceType`, `?source_type`, `?state`, `?stateDescription`, `?stateName`, `?statePhase`, `?status`, `?statusCode`, `?structure`, `?structure_constraint`, `?substrate`, `?summary`, `?surface`, `?surfaceType`, `?synergyCategoryIfApplicable`, `?tags`, `?targetFormat`, `?target_type`, `?template`, `?templateRole`, `?templateTitle`, `?timestamp`, `?title`, `?traceDeviations`, `?traceGasToReturn`, `?transitionActionOnFire`, `?transitionGuardCondition`, `?transitionGuardExpression`, `?transitionTarget`, `?transitionTargetName`, `?type`, `?verdict`, `?verdictFitness`, `?verdictPrecision`, `?verdictType`, `?version`, `?violation`, `?violationType`, `?witness_label`, `?witness_name`, `?workflow`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/process-intelligence/ggen/ontology-extensions.ttl` (18588 bytes)
- `/Users/sac/process-intelligence/ggen/queries/extract-board-claims.rq` (2218 bytes)
- `/Users/sac/process-intelligence/ggen/queries/extract-diligence-claims.rq` (3073 bytes)
- `/Users/sac/process-intelligence/ggen/queries/extract-lifecycle-governance.rq` (3710 bytes)
- `/Users/sac/process-intelligence/ggen/queries/extract-visualizer-data.rq` (1737 bytes)
- `/Users/sac/process-intelligence/ggen/queries/select-boundary-crossings.rq` (830 bytes)
- `/Users/sac/process-intelligence/ggen/queries/select-process-forms.rq` (794 bytes)
- `/Users/sac/process-intelligence/ggen/queries/select-witness-projections.rq` (1008 bytes)
- `/Users/sac/process-intelligence/ggen/wasm4pm-compat.ttl` (11449 bytes)
- `/Users/sac/process-intelligence/queries/extract-pcp-types.rq` (199 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/api-specification.ttl` (13550 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/autonomic-law.ttl` (6201 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/checkpoint-ledger.ttl` (16953 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/conformance-ledger.ttl` (14116 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/forbidden-collapse-law.ttl` (18228 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/governance-policy.ttl` (4101 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/graduation-boundary.ttl` (13804 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-audit-law.ttl` (17584 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-checkpoint-ledger.ttl` (23100 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-generation-ledger.ttl` (13752 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-invalid-extension-ledger.ttl` (13775 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-project-registry.ttl` (15415 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-source-ledger.ttl` (18687 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-ggen-unified-run.ttl` (9359 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/pi-program.ttl` (13562 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/project-registry.ttl` (21904 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/ontology/research-artifact-ledger.ttl` (18841 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-all-legacy-ggen-classified.rq` (542 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-checkpoint-has-receipts.rq` (558 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-checkpoints-have-receipts-or-explicit-missing.rq` (718 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-closure-invariant.rq` (637 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-commitment-integrity.rq` (538 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-compliance-ledger.rq` (669 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-every-generation-rule-has-query-template-output.rq` (726 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-every-rendered-artifact-has-source-trace.rq` (671 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-evidence-traceability.rq` (591 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-gates-complete.rq` (529 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-client-only-auth.rq` (708 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-commit-count-alive.rq` (656 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-dashboard-truth.rq` (677 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-dto-flattening.rq` (509 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-file-count-alive.rq` (726 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-forced-alive.rq` (528 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-hand-written-research-warrant.rq` (504 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-invalid-ggen-extension.rq` (487 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-invalid-new-ggen-source.rq` (477 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-manual-prompt-writing.rq` (556 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-realtime-as-evidence.rq` (674 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-telemetry-as-receipt.rq` (614 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-tool-smuggling.rq` (657 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-no-unsigned-verdicts.rq` (470 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-partial-checkpoint-possible.rq` (612 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-partial-has-gaps.rq` (452 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-source-court-citations.rq` (449 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-validator-passes-valid-tera.rq` (537 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-validator-rejects-invalid-tera.rq` (544 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/audit-warrant-path-exists.rq` (654 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-alive-claims.rq` (462 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-checkpoints.rq` (832 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-generation-rules.rq` (747 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-ggen-manifests.rq` (753 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-ontology-graphs.rq` (677 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-projects.rq` (506 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-query-surfaces.rq` (677 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-rendered-artifacts.rq` (720 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-all-template-surfaces.rq` (720 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-andon-gates.rq` (3302 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-api-auth-rules.rq` (927 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-api-endpoints.rq` (1019 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-checkpoints.rq` (546 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-compatibility-surfaces.rq` (593 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-drift-detection-rules.rq` (4447 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-engine-surfaces.rq` (589 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-failed-gates.rq` (942 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-forbidden-collapses.rq` (470 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-invalid-ggen-files.rq` (588 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-manufacturing-surfaces.rq` (429 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-mape-monitors.rq` (2317 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-mobile-substrate-surfaces.rq` (455 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-next-workflows.rq` (438 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-partial-claims.rq` (501 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-proof-cells.rq` (443 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-rate-limit-rules.rq` (858 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-remediation-candidates.rq` (731 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-telemetry-feedstock-surfaces.rq` (461 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-unified-run-plan.rq` (591 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-warrant-paths.rq` (922 bytes)
- `/Users/sac/process-intelligence/research/pi-program/ggen/queries/select-workflow-substrate-surfaces.rq` (453 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/checkpoint-law.ttl` (5549 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/forbidden-collapse-law.ttl` (11226 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/hook-law.ttl` (2262 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/prompt-manufactory.ttl` (7471 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/research-program-law.ttl` (6967 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/skill-law.ttl` (1915 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/subagent-role-law.ttl` (8311 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/ontology/workflow-law.ttl` (4659 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-checkpoint-prompts.rq` (526 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-hook-policies.rq` (514 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-legacy-ggen-files.rq` (410 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-rendered-prompts.rq` (485 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-research-programs.rq` (517 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-skill-prompts.rq` (509 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-subagent-prompts.rq` (658 bytes)
- `/Users/sac/process-intelligence/research/prompt-manufactory/ggen/queries/select-workflow-prompts.rq` (935 bytes)

</details>
