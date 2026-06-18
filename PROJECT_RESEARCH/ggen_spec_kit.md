# Research Dossier: `ggen-spec-kit`

**Total Files:** 25 Ontologies (.ttl) | 27 Queries (.rq)
**Total Volume:** 52 files

## 1. Core Vocabularies (Prefixes)
- `cli: <http://github.com/github/spec-kit/cli#>`
- `dcat: <http://www.w3.org/ns/dcat#>`
- `dcterms: <http://purl.org/dc/terms/>`
- `foaf: <http://xmlns.com/foaf/0.1/>`
- `ggen: <https://ggen.io/ns#>`
- `jtbd: <http://github.com/github/spec-kit/jtbd#>`
- `owl: <http://www.w3.org/2001/XMLSchema#>`
- `owl: <http://www.w3.org/2002/07/owl#>`
- `rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>`
- `rdfs: <http://www.w3.org/2000/01/rdf-schema#>`
- `sh: <http://www.w3.org/ns/shacl#>`
- `shacl: <http://www.w3.org/ns/shacl#>`
- `sk: <http://github.com/github/spec-kit#>`
- `spec: <https://example.com/spec-kit#>`
- `specify: <https://github.com/github/spec-kit#>`
- `xsd: <http://www.w3.org/2001/XMLSchema#>`

## 2. Domain Taxonomy & Entities
### Base Classes & Shapes
- `:Command`
- `:CommandShape`
- `:Option`
- `:OptionShape`
- `:Parameter`
- `:ParameterShape`
- `:Subcommand`
- `cli:Argument`
- `cli:ArgumentShape`
- `cli:CheckCommand`
- `cli:Command`
- `cli:CommandGroup`
- `cli:CommandGroupShape`
- `cli:CommandShape`
- `cli:Example`
- `cli:ExampleShape`
- `cli:GgenCommand`
- `cli:GgenSyncCommand`
- `cli:InitCommand`
- `cli:Option`
- `cli:OptionShape`
- `cli:PmCommand`
- `cli:PmConformCommand`
- `cli:PmDiscoverCommand`
- `cli:PmStatsCommand`
- `cli:SpiffCommand`
- `cli:SpiffRunWorkflowCommand`
- `cli:SpiffValidateCommand`
- `cli:TypeEnum`
- `cli:ValidationRule`
- `cli:ValidationRuleShape`
- `cli:VersionCommand`
- `jtbd:Anxiety`
- `jtbd:AnxietyShape`
- `jtbd:CompetitiveSolution`
- `jtbd:ContextClue`
- `jtbd:CurrentSolution`
- `jtbd:CustomerSegment`
- `jtbd:CustomerSegmentShape`
- `jtbd:DesiredOutcome`
- `jtbd:DesiredOutcomeShape`
- `jtbd:EmotionalJob`
- `jtbd:EnhancedPersonaShape`
- `jtbd:FeatureImpactAnalysis`
- `jtbd:FeatureImpactAnalysisShape`
- `jtbd:Force`
- `jtbd:ForceType`
- `jtbd:FunctionalJob`
- `jtbd:Habit`
- `jtbd:HabitShape`
- *...and 126 more.*

## 3. Extraction Layer (SPARQL)
- **Query Types Executed:** {'SELECT': 27}

### Projected Variables (SELECT ?var)
This project actively projects the following variables into code/templates:
> `?activeUsers`, `?adoptionRate`, `?attemptCount`, `?audience`, `?breakingChanges`, `?category`, `?changeDescription`, `?changeId`, `?changeType`, `?circumstanceDescription`, `?command`, `?commandAlias`, `?commandDeprecated`, `?commandDescription`, `?commandGroup`, `?commandName`, `?completedCount`, `?config`, `?configDefault`, `?configDescription`, `?configName`, `?configRequired`, `?configType`, `?contextDescription`, `?currentSatisfaction`, `?defaultValue`, `?deprecatedFeatures`, `?description`, `?developmentCost`, `?dueDate`, `?effectivenessScore`, `?emotionalBenefit`, `?entityDescription`, `?entityName`, `?entityType`, `?examples`, `?feature`, `?featureBranch`, `?featureDescription`, `?featureName`, `?featureStatus`, `?flag`, `?forceDescription`, `?forceStrength`, `?forceType`, `?guide`, `?isVariadic`, `?jobDescription`, `?jobFrequency`, `?jobImportance`, `?jobName`, `?jobSatisfaction`, `?jobTitle`, `?jobType`, `?journeyStage`, `?journeyStageOrder`, `?lastUpdated`, `?maintenanceCost`, `?module`, `?name`, `?occurrenceFrequency`, `?occurrenceFrequencyUnit`, `?optionName`, `?optionType`, `?outcome`, `?outcomeAchieved`, `?outcomeClarifier`, `?outcomeDescription`, `?outcomeDirection`, `?outcomeImportance`, `?outcomeMetric`, `?outcomeName`, `?outcomeObject`, `?outcomeOpportunityScore`, `?outcomePriority`, `?outcomeSatisfaction`, `?outcomeSatisfactionImprovement`, `?outcomeStatement`, `?outcomeTarget`, `?painpoint`, `?painpointCategory`, `?painpointDescription`, `?painpointFrequency`, `?painpointImpact`, `?painpointResolved`, `?painpointSeverity`, `?paramName`, `?paramType`, `?personaChallenges`, `?personaFrustrations`, `?personaGoals`, `?personaName`, `?personaRole`, `?phase`, `?phaseDescription`, `?phaseId`, `?phaseName`, `?prerequisites`, `?principle`, `?principleId`, `?principleIndex`, `?priority`, `?purpose`, `?rationale`, `?relatedJob`, `?relatedOutcome`, `?release`, `?releaseDate`, `?required`, `?satisfaction30dAgo`, `?satisfaction60dAgo`, `?satisfaction90dAgo`, `?satisfactionAfter`, `?satisfactionBefore`, `?sections`, `?segmentName`, `?shortFlag`, `?solutionName`, `?solutionSatisfaction`, `?solutionType`, `?stageCompletionRate`, `?stageTimeline`, `?stageTimelineUnit`, `?status`, `?step`, `?stepDescription`, `?stepId`, `?stepIndex`, `?subcommandDesc`, `?subcommandName`, `?successCriteria`, `?targetValue`, `?telemetryName`, `?timeToFirstValue`, `?timeToFirstValueUnit`, `?timeToFullValue`, `?timeToFullValueUnit`, `?title`, `?totalCost`, `?transitionToNextStage`, `?triggerFrequency`, `?triggerType`, `?usageFrequency`, `?userSatisfaction`, `?versionNumber`, `?violations`

## 4. File Inventory
<details>
<summary>Click to expand all files</summary>

- `/Users/sac/ggen-spec-kit/docs/examples/cli-command-spec-example.ttl` (10001 bytes)
- `/Users/sac/ggen-spec-kit/docs/examples/jtbd-example-feature.ttl` (17674 bytes)
- `/Users/sac/ggen-spec-kit/docs/ggen-examples/feature-query.rq` (740 bytes)
- `/Users/sac/ggen-spec-kit/docs/ggen-examples/feature.ttl` (1764 bytes)
- `/Users/sac/ggen-spec-kit/memory/documentation.ttl` (13002 bytes)
- `/Users/sac/ggen-spec-kit/memory/jtbd-customer-jobs.ttl` (40883 bytes)
- `/Users/sac/ggen-spec-kit/memory/jtbd-example.ttl` (16367 bytes)
- `/Users/sac/ggen-spec-kit/memory/jtbd-forces-analysis.ttl` (44177 bytes)
- `/Users/sac/ggen-spec-kit/memory/philosophy.ttl` (20730 bytes)
- `/Users/sac/ggen-spec-kit/ontology/cli-command-shapes.ttl` (13623 bytes)
- `/Users/sac/ggen-spec-kit/ontology/cli-commands-uvmgr-full.ttl` (15777 bytes)
- `/Users/sac/ggen-spec-kit/ontology/cli-commands-uvmgr.ttl` (11123 bytes)
- `/Users/sac/ggen-spec-kit/ontology/cli-commands.ttl` (14535 bytes)
- `/Users/sac/ggen-spec-kit/ontology/cli-schema.ttl` (25009 bytes)
- `/Users/sac/ggen-spec-kit/ontology/jtbd-schema.ttl` (45940 bytes)
- `/Users/sac/ggen-spec-kit/ontology/jtbd-shapes.ttl` (25818 bytes)
- `/Users/sac/ggen-spec-kit/ontology/spec-kit-docs-extension.ttl` (22604 bytes)
- `/Users/sac/ggen-spec-kit/ontology/spec-kit-jtbd-extension.ttl` (15121 bytes)
- `/Users/sac/ggen-spec-kit/ontology/spec-kit-schema.ttl` (25456 bytes)
- `/Users/sac/ggen-spec-kit/schema/cli-commands-uvmgr-full.ttl` (15777 bytes)
- `/Users/sac/ggen-spec-kit/schema/specify-domain.ttl` (22346 bytes)
- `/Users/sac/ggen-spec-kit/sparql/changelog-query.rq` (876 bytes)
- `/Users/sac/ggen-spec-kit/sparql/command-query.rq` (2374 bytes)
- `/Users/sac/ggen-spec-kit/sparql/command-test-query.rq` (4803 bytes)
- `/Users/sac/ggen-spec-kit/sparql/config-query.rq` (763 bytes)
- `/Users/sac/ggen-spec-kit/sparql/extract-commands.rq` (680 bytes)
- `/Users/sac/ggen-spec-kit/sparql/extract-options.rq` (859 bytes)
- `/Users/sac/ggen-spec-kit/sparql/extract-parameters.rq` (701 bytes)
- `/Users/sac/ggen-spec-kit/sparql/extract-runtime.rq` (851 bytes)
- `/Users/sac/ggen-spec-kit/sparql/guide-query.rq` (1111 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-completion-rate.rq` (4020 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-extract-jobs.rq` (3179 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-feature-effectiveness.rq` (5176 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-feature-impact.rq` (2584 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-job-feature-mapping.rq` (2234 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-outcome-achievement.rq` (4344 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-outcome-metrics.rq` (2336 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-painpoint-analysis.rq` (4736 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-painpoint-coverage.rq` (2535 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-persona-analysis.rq` (2904 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-persona-journey.rq` (5838 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-roi-calculation.rq` (5866 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-satisfaction-trends.rq` (5405 bytes)
- `/Users/sac/ggen-spec-kit/sparql/jtbd-time-to-value.rq` (5605 bytes)
- `/Users/sac/ggen-spec-kit/sparql/principle-query.rq` (687 bytes)
- `/Users/sac/ggen-spec-kit/sparql/workflow-query.rq` (647 bytes)
- `/Users/sac/ggen-spec-kit/templates/jtbd/example-feature-jtbd.ttl` (4292 bytes)
- `/Users/sac/ggen-spec-kit/templates/jtbd/example-roadmap-jtbd.ttl` (8112 bytes)
- `/Users/sac/ggen-spec-kit/templates/jtbd/example-sparql-queries.rq` (7201 bytes)
- `/Users/sac/ggen-spec-kit/templates/schema/example-domain.ttl` (5031 bytes)
- `/Users/sac/ggen-spec-kit/templates/schema/inference-rules.ttl` (3480 bytes)
- `/Users/sac/ggen-spec-kit/tests/integration/fixtures/feature-content.ttl` (1694 bytes)

</details>
