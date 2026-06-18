# Ontology Synthesis & Custom Domain Design

**Phase 4: Design 6 Custom Rocket-Craft Domain-Specific Ontologies**

Date: 2026-06-18 | Status: Complete Research Foundation (12 agents) | Ready for Implementation

---

## Executive Summary

Based on 12+ completed research agents covering 50+ public ontologies across 10 distinct categories, this document synthesizes findings and presents 6 custom domain-specific ontologies (DSOs) for rocket-craft's semantic foundation.

**Key Finding:** No single public ontology covers rocket-craft's unique architecture (typestate machines, phantom-typed units, multi-game monorepo, Rust-first approach, affidavit receipts, Monte Carlo balancing). A layered custom ontology stack is required.

---

## Part 1: Consolidated Research Matrix

### Foundation Ontologies (Priority 1 — Deploy Now)

| Ontology | Namespace | W3C Status | Use in Rocket-Craft |
|----------|-----------|-----------|-------------------|
| **PROV-O** | http://www.w3.org/ns/prov# | W3C Recommendation | Test execution provenance, build artifact lineage |
| **QUDT** | http://qudt.org/schema/qudt/ | NASA Standard | Phantom-typed units (Hp, Gold, Damage, Mana, XP) |
| **SKOS** | http://www.w3.org/2004/02/skos/core# | W3C Recommendation | Test hierarchy (unit → integration → property-based) |
| **QB (Data Cube)** | http://purl.org/linked-data/cube# | W3C Recommendation | Multi-dimensional coverage metrics (workspace × branch × type × time) |
| **OWL Time** | http://www.w3.org/2006/time# | W3C Recommendation | Temporal constraints, test duration, coverage stability windows |
| **SHACL** | http://www.w3.org/ns/shacl# | W3C Recommendation | RDF graph validation, quality gates (92%+ coverage thresholds) |

### Secondary Integration Ontologies (Priority 2 — Phase 4.1)

| Ontology | Purpose | Adoption |
|----------|---------|----------|
| **OBO Relations** | Test dependencies, crate DAG modeling, fixture chains | High in biomedical; new for Rust |
| **ShEx** | Validation gates ("all nexus-combat ≥85% coverage") | Emerging in quality assurance |
| **DCTERMS** | Universal metadata (title, description, modified dates) | Very High |
| **DCAT** | Catalog test suites as searchable datasets | High |
| **FOAF** | Test ownership, maintainers, contributors | High |
| **SPDX** | Supply chain integrity, vulnerability correlation, SBOMs | Very High (mandated US federal) |

### Domain-Specific Frameworks

**OCEL 2.0 (Object-Centric Event Logging):** Process mining standard aligned with rocket-craft's event-driven architecture

**BPMN 2.0:** Workflow/pipeline representation ontology (extensible for CI/CD)

**Eiffel:** CI/CD event protocol (complementary to OCEL)

**OpenTelemetry Semantic Conventions:** Observability metrics standardization (for performance/coverage tracing)

---

## Part 2: Research Gaps Requiring Custom Ontologies

### Critical Gaps (No Public Standard Exists)

**Gap #1: Property-Based Testing Vocabulary**
- No RDF vocabulary for proptest invariants, shrinking strategies, hypothesis counts, coverage shrinking depth
- **Location:** nexus-tests (10-crate test harness)
- **Scope:** 50+ property-based tests across nexus-engine; 1000+ proptest invariants

**Gap #2: Phantom-Typed Units & Domain-Specific Types**
- No ontology for phantom-typed units (Hp, Gold, Damage, Mana, XP, Armor, ComboMultiplier)
- Bounded type constraints (e.g., Hp ∈ [0, 999999])
- Type relationships (Hp × DamageMultiplier → DeltaHp)
- **Location:** nexus-types (zero-dependency root crate)
- **Scope:** 20+ phantom types; 50+ type operations

**Gap #3: Typestate Machine Patterns**
- Machine<Law, Phase> pattern not formalized in RDF
- State transitions, compile-time safety guarantees, legal state transition rules
- **Locations:** Multiple (nexus-combat, connection, project-manifest)
- **Scope:** 7+ typestate machines across codebase

**Gap #4: Affidavit Receipt Provenance**
- Novel crypto-signed receipt mechanism combining PROV-O + QUDT + signing attestation
- Links actions (Activities) to cryptographic verification (Agents)
- **Location:** unify-rs/unify-receipts (6-crate subsystem)
- **Scope:** Receipt lineage, signature validation, audit trail

**Gap #5: Monte Carlo Balancing Metrics**
- Stochastic game balance metrics (convergence probability, expected value variance)
- Statistical distribution modeling for gameplay
- **Location:** nexus-economy (balancing subsystem)
- **Scope:** 50+ balancing rules expressed probabilistically

**Gap #6: Mutation Testing & Game Outcome Mutations**
- Game state mutation tracking (state → mutated-state transitions)
- Mutation kill detection (does changed rule affect outcome?)
- **Location:** nexus-tests (invariant validation)
- **Scope:** Orthogonal to traditional mutation testing; domain-specific

**Gap #7: Multi-Game Deployment & Platform Targeting**
- Ontology linking games (ShooterGame, SurvivalGame, etc.) to platforms (Win64, Android, iOS, HTML5)
- Platform constraints (Apex destruction not in HTML5; procedural meshes unsupported)
- **Location:** project-manifest.json + versions/
- **Scope:** 6 games × 4 platform families

**Gap #8: Unified CLI & Command Authority**
- Semantic modeling of CLI commands (./rocket setup, ./rocket audit, ./rocket wasm)
- Authority layers (who can run what; role-based access control)
- **Location:** rocket-cmd / rocket-sdk
- **Scope:** 12+ commands; 4 authority levels

**Gap #9: Rust Workspace Dependency Graph**
- Crate-level dependencies, feature flags, workspace resolver versions
- Transitive closure computation for impact analysis
- **Location:** tools/, nexus-engine/, blueprint-rs/, unify-rs/, ib4-mud/, chicago-tdd-tools/, asset-pipeline/
- **Scope:** 44 crates; 300+ direct dependencies

**Gap #10: Development Workflow & Branching Rules**
- Branch naming conventions, merge-to-main restrictions, semantic versioning alignment
- Which workspaces are in CI/CD and why
- **Location:** CLAUDE.md + .github/workflows/
- **Scope:** 7 workspaces; 2 CI jobs active

---

## Part 3: 6 Custom Rocket-Craft Domain-Specific Ontologies

### 1. rocket-craft-core.owl

**Purpose:** Foundation extending PROV-O, SHACL, Dublin Core, QUDT

**Namespace:** `http://rocket-craft.org/ontology/core#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_CORE_`

**Key Classes:**
```turtle
@prefix rc: <http://rocket-craft.org/ontology/core#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

rc:GameProject a owl:Class ;
  rdfs:subClassOf prov:Entity ;
  rdfs:label "Game Project" ;
  rdfs:comment "UE4 game project (ShooterGame, SurvivalGame, etc.)" .

rc:RustCrate a owl:Class ;
  rdfs:subClassOf prov:Entity ;
  rdfs:label "Rust Crate" ;
  rdfs:comment "Rust workspace crate (nexus-combat, unify-rdf, etc.)" .

rc:BuildArtifact a owl:Class ;
  rdfs:subClassOf prov:Entity ;
  rdfs:label "Build Artifact" ;
  rdfs:comment "Compiled output (.uasset, binary, .wasm)" ;
  prov:wasDerivedFrom rc:RustCrate .

rc:DeploymentTarget a owl:Class ;
  rdfs:label "Deployment Target" ;
  rdfs:comment "Platform + project pair (ShooterGame-Win64, SurvivalGame-Android)" .

# Object properties
rc:compiledFrom a owl:ObjectProperty ;
  rdfs:domain rc:BuildArtifact ;
  rdfs:range rc:RustCrate ;
  owl:inverseOf rc:compilesTo .

rc:targetsGame a owl:ObjectProperty ;
  rdfs:domain rc:DeploymentTarget ;
  rdfs:range rc:GameProject .

rc:deploysTo a owl:ObjectProperty ;
  rdfs:domain rc:BuildArtifact ;
  rdfs:range rc:DeploymentTarget .

# Data properties
rc:cargoTomlPath a owl:DatatypeProperty ;
  rdfs:domain rc:RustCrate ;
  rdfs:range xsd:string .

rc:linesOfCode a owl:DatatypeProperty ;
  rdfs:domain rc:RustCrate ;
  rdfs:range xsd:integer .
```

**Extends:**
- PROV-O for provenance tracking (who built what, when)
- QUDT for metrics quantification
- Dublin Core for metadata

**Integration Points:**
- Links to project-manifest.json via JSON-LD context
- Feeds into rocket-craft-manifest.owl for schema definitions
- SHACL shapes validate crate metadata consistency

---

### 2. rocket-craft-states.owl

**Purpose:** Formalize Machine<Law, Phase> typestate patterns as OWL restrictions

**Namespace:** `http://rocket-craft.org/ontology/states#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_STATES_`

**Key Classes:**
```turtle
@prefix rs: <http://rocket-craft.org/ontology/states#> .

rs:TypestateSystem a owl:Class ;
  rdfs:label "Typestate System" ;
  rdfs:comment "A zero-cost state machine with compile-time safety" .

rs:State a owl:Class ;
  rdfs:label "State" ;
  rdfs:comment "A phantom-typed state marker (Disconnected, Connected, etc.)" .

rs:Transition a owl:Class ;
  rdfs:label "Legal Transition" ;
  rdfs:comment "Allowed state change; undefined transitions are compile errors" .

# Concrete typestate systems
rs:ConnectionState a owl:Class ;
  rdfs:subClassOf rs:State ;
  rdfs:label "Connection State" ;
  rdfs:comment "nexus-net/src/connection.rs state machine" .

rs:Disconnected a rs:ConnectionState ;
  rdfs:label "Disconnected" .

rs:Handshaking a rs:ConnectionState ;
  rdfs:label "Handshaking" .

rs:Connected a rs:ConnectionState ;
  rdfs:label "Connected" .

rs:Authenticated a rs:ConnectionState ;
  rdfs:label "Authenticated" .

rs:InLobby a rs:ConnectionState ;
  rdfs:label "InLobby" .

rs:InMatch a rs:ConnectionState ;
  rdfs:label "InMatch" .

# Transitions
rs:Disconnected_to_Handshaking a rs:Transition ;
  rs:from rs:Disconnected ;
  rs:to rs:Handshaking ;
  rs:method "connect()" .

rs:Handshaking_to_Connected a rs:Transition ;
  rs:from rs:Handshaking ;
  rs:to rs:Connected ;
  rs:method "complete_handshake()" .

rs:Connected_to_Authenticated a rs:Transition ;
  rs:from rs:Connected ;
  rs:to rs:Authenticated ;
  rs:method "authenticate(token)" .

# CombatMachine states
rs:CombatState a owl:Class ;
  rdfs:subClassOf rs:State ;
  rdfs:label "Combat State" ;
  rdfs:comment "nexus-combat/src/machine.rs state machine" .

rs:Idle a rs:CombatState .
rs:Attacking a rs:CombatState .
rs:Parrying a rs:CombatState .
rs:Recovering a rs:CombatState .
rs:Stunned a rs:CombatState .

# ProjectManifest states
rs:ManifestState a owl:Class ;
  rdfs:subClassOf rs:State ;
  rdfs:label "Manifest State" ;
  rdfs:comment "unify-rdf/src/project_bridge.rs manifest validation" .

rs:Pending a rs:ManifestState .
rs:Ingested a rs:ManifestState .
rs:Validated a rs:ManifestState .

# Properties
rs:from a owl:ObjectProperty ;
  rdfs:domain rs:Transition ;
  rdfs:range rs:State .

rs:to a owl:ObjectProperty ;
  rdfs:domain rs:Transition ;
  rdfs:range rs:State .

rs:method a owl:DatatypeProperty ;
  rdfs:domain rs:Transition ;
  rdfs:range xsd:string ;
  rdfs:comment "Method name that enables this transition" .

rs:requires a owl:ObjectProperty ;
  rdfs:domain rs:Transition ;
  rdfs:range owl:Thing ;
  rdfs:comment "Precondition (e.g., valid token for authentication)" .

rs:IllegalTransition a owl:Class ;
  rdfs:label "Illegal Transition" ;
  rdfs:comment "A compile error; undefined in state machine impl" .
```

**Formalization Details:**
- Each state machine maps to a TypestateSystem instance
- Legal transitions are asserted; undefined transitions are owl:disjointWith
- OWL constraints verify no transition from State A leads to State A without explicit loop transition
- Inverse transitions are explicitly forbidden (e.g., once Connected → Authenticated, no path back to Disconnected)

**Integration Points:**
- Links to rocket-craft-core.owl for artifact state tracking
- Feeds into SHACL shape validation (e.g., "a Build in state Failed cannot transition to state Active")
- Enables SPARQL queries: "What states can be reached from Disconnected?"

---

### 3. rocket-craft-types.owl

**Purpose:** Phantom-typed units mapped to QUDT QuantityKind and bounded constraints

**Namespace:** `http://rocket-craft.org/ontology/types#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_TYPES_`

**Key Classes:**
```turtle
@prefix rt: <http://rocket-craft.org/ontology/types#> .
@prefix qudt: <http://qudt.org/schema/qudt/> .

rt:PhantomType a owl:Class ;
  rdfs:label "Phantom Type" ;
  rdfs:comment "Zero-cost type wrapping a primitive with semantic meaning" .

# Phantom-typed units (nexus-types/src/lib.rs)
rt:Hp a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Hit Points" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0 ;
  rt:upperBound 999999 ;
  rdfs:comment "Player health in game" .

rt:Gold a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Gold Currency" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0 ;
  rt:upperBound 99999999 ;
  rdfs:comment "In-game currency" .

rt:Damage a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Damage Output" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0 ;
  rt:upperBound 99999 ;
  rdfs:comment "Weapon or ability damage" .

rt:Mana a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Mana Points" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0 ;
  rt:upperBound 500000 ;
  rdfs:comment "Magical resource" .

rt:Xp a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Experience Points" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0 ;
  rt:upperBound 999999999 ;
  rdfs:comment "Progression metric" .

rt:Armor a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Armor Rating" ;
  qudt:hasUnit qudt:Percent ;
  rt:lowerBound 0 ;
  rt:upperBound 1.0 ;
  rdfs:comment "Damage reduction coefficient ∈ [0, 1.0]" .

rt:ComboMultiplier a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Combo Multiplier" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 1.0 ;
  rt:upperBound 10.0 ;
  rdfs:comment "Damage multiplier for consecutive hits" .

rt:TimeDilation a owl:Class ;
  rdfs:subClassOf rt:PhantomType, qudt:QuantityKind ;
  rdfs:label "Time Dilation Factor" ;
  qudt:hasUnit qudt:Number ;
  rt:lowerBound 0.1 ;
  rt:upperBound 2.0 ;
  rdfs:comment "Slow-motion multiplier; 1.0 = normal speed" .

# Type operations (relationships between types)
rt:TypeOperation a owl:Class ;
  rdfs:label "Type Operation" ;
  rdfs:comment "Describes how types combine (Hp × DamageMultiplier → DeltaHp)" .

rt:applies_to a owl:ObjectProperty ;
  rdfs:domain rt:PhantomType ;
  rdfs:range rt:TypeOperation ;
  rdfs:comment "Which operations consume this type" .

# Bounded constraints
rt:lowerBound a owl:DatatypeProperty ;
  rdfs:domain rt:PhantomType ;
  rdfs:range xsd:double ;
  rdfs:comment "Minimum valid value" .

rt:upperBound a owl:DatatypeProperty ;
  rdfs:domain rt:PhantomType ;
  rdfs:range xsd:double ;
  rdfs:comment "Maximum valid value" .

rt:invariant a owl:DatatypeProperty ;
  rdfs:domain rt:PhantomType ;
  rdfs:range xsd:string ;
  rdfs:comment "Logical constraint (e.g., 'Armor ∈ [0, 1.0]')" .

# SHACL shape for Hp validation
rt:HpShape a sh:NodeShape ;
  sh:targetClass rt:Hp ;
  sh:property [
    sh:path qudt:numericValue ;
    sh:datatype xsd:integer ;
    sh:minInclusive 0 ;
    sh:maxInclusive 999999 ;
    sh:message "Hp must be integer in range [0, 999999]"
  ] .
```

**Integration Points:**
- QUDT namespace for units of measurement
- SHACL shapes for value validation
- Links to rocket-craft-core.owl (types are part of game design)
- Feeds into nexus-economy.owl for balancing constraints

---

### 4. rocket-craft-manifest.owl

**Purpose:** project-manifest.json schema as RDF/OWL; semantic ingestion bridge

**Namespace:** `http://rocket-craft.org/ontology/manifest#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_MANIFEST_`

**Key Classes:**
```turtle
@prefix rm: <http://rocket-craft.org/ontology/manifest#> .

rm:ProjectManifest a owl:Class ;
  rdfs:label "Project Manifest" ;
  rdfs:comment "Root schema for project-manifest.json" .

rm:UE4Project a owl:Class ;
  rdfs:subClassOf prov:Entity ;
  rdfs:label "UE4 Game Project" .

rm:RustWorkspace a owl:Class ;
  rdfs:subClassOf prov:Entity ;
  rdfs:label "Rust Workspace" .

# UE4 Projects
rm:ShooterGame a rm:UE4Project ;
  rdfs:label "Shooter Game" ;
  rm:uprojectPath "versions/4.24-Shooter/ShooterGame/" ;
  rm:platform "Win64", "HTML5" ;
  rm:targets "ShooterGameEditor", "ShooterClient", "ShooterGame", "ShooterServer" .

rm:SurvivalGame a rm:UE4Project ;
  rdfs:label "Survival Game" ;
  rm:uprojectPath "versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/" ;
  rm:platform "Win64", "Android", "HTML5" ;
  rm:targets "SurvivalGameEditor", "SurvivalGameServer", "SurvivalGame" .

# Rust Workspaces
rm:ToolsWorkspace a rm:RustWorkspace ;
  rdfs:label "Tools Workspace" ;
  rm:path "tools/" ;
  rm:crateCount 5 ;
  rm:contains rm:RocketSdk, rm:RocketCmd, rm:Knhk, rm:Unrdf .

rm:NexusEngineWorkspace a rm:RustWorkspace ;
  rdfs:label "Nexus Engine" ;
  rm:path "nexus-engine/" ;
  rm:crateCount 10 ;
  rm:description "Gundam Nexus game engine" ;
  rm:contains rm:NexusTypes, rm:NexusCombat, rm:NexusNet, rm:NexusTests .

# Crates
rm:RocketSdk a prov:Entity ;
  rdfs:label "rocket-sdk" ;
  rm:path "tools/rocket-sdk/" ;
  rm:linesOfCode 3200 ;
  rm:testCoverage 78.5 .

rm:NexusTypes a prov:Entity ;
  rdfs:label "nexus-types" ;
  rm:path "nexus-engine/crates/nexus-types/" ;
  rm:linesOfCode 1800 ;
  rm:testCoverage 91.2 ;
  rm:comment "Zero-dependency root of monorepo; phantom-typed units and IDs" .

# Object properties
rm:contains a owl:ObjectProperty ;
  rdfs:domain rm:RustWorkspace ;
  rdfs:range prov:Entity ;
  rdfs:label "Contains Crate" .

rm:hasUE4Project a owl:ObjectProperty ;
  rdfs:domain rm:ProjectManifest ;
  rdfs:range rm:UE4Project .

rm:hasRustWorkspace a owl:ObjectProperty ;
  rdfs:domain rm:ProjectManifest ;
  rdfs:range rm:RustWorkspace .

# Data properties
rm:uprojectPath a owl:DatatypeProperty ;
  rdfs:range xsd:string .

rm:path a owl:DatatypeProperty ;
  rdfs:range xsd:string .

rm:platform a owl:DatatypeProperty ;
  rdfs:range xsd:string ;
  rdfs:comment "Target platform (Win64, Android, iOS, HTML5)" .

rm:targets a owl:DatatypeProperty ;
  rdfs:range xsd:string ;
  rdfs:comment "Build target names" .

rm:linesOfCode a owl:DatatypeProperty ;
  rdfs:range xsd:integer .

rm:testCoverage a owl:DatatypeProperty ;
  rdfs:range xsd:double ;
  rdfs:comment "Test coverage percentage (0.0-100.0)" .

# SHACL validation shape
rm:ProjectManifestShape a sh:NodeShape ;
  sh:targetClass rm:ProjectManifest ;
  sh:property [
    sh:path rm:hasUE4Project ;
    sh:minCount 1 ;
    sh:message "ProjectManifest must have at least one UE4Project"
  ] ;
  sh:property [
    sh:path rm:hasRustWorkspace ;
    sh:minCount 1 ;
    sh:message "ProjectManifest must have at least one RustWorkspace"
  ] .
```

**JSON-LD Context:**
```json
{
  "@context": {
    "rm": "http://rocket-craft.org/ontology/manifest#",
    "prov": "http://www.w3.org/ns/prov#",
    "sh": "http://www.w3.org/ns/shacl#",
    "ProjectManifest": { "@type": "@id", "@id": "rm:ProjectManifest" },
    "UE4Project": { "@type": "@id", "@id": "rm:UE4Project" },
    "uprojectPath": { "@id": "rm:uprojectPath" },
    "platform": { "@id": "rm:platform", "@type": "@set" },
    "targets": { "@id": "rm:targets", "@type": "@set" }
  }
}
```

**Integration Points:**
- Auto-generated from project-manifest.json via ./rocket sync
- Feeds into SHACL validation pipeline (./rocket audit)
- Links to rocket-craft-states.owl (manifest validation states: Pending → Ingested → Validated)
- Cross-references rocket-craft-core.owl for GameProject/RustCrate instances

---

### 5. rocket-craft-quality.owl

**Purpose:** Test coverage, error handling constraints, quality gates from gap audit

**Namespace:** `http://rocket-craft.org/ontology/quality#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_QUALITY_`

**Key Classes:**
```turtle
@prefix rq: <http://rocket-craft.org/ontology/quality#> .
@prefix dqv: <http://www.w3.org/ns/dqv#> .
@prefix qb: <http://purl.org/linked-data/cube#> .

rq:TestCoverage a dqv:Dimension ;
  rdfs:label "Test Coverage" ;
  rdfs:comment "Percentage of code exercised by tests" .

rq:CoverageMetric a dqv:Metric ;
  dqv:inDimension rq:TestCoverage ;
  rdfs:label "Coverage %" ;
  qudt:unit qudt:Percent .

rq:TestHierarchy a skos:ConceptScheme ;
  skos:prefLabel "Test Classification"@en ;
  rdfs:comment "SKOS taxonomy of test types" .

rq:UnitTest a skos:Concept ;
  skos:inScheme rq:TestHierarchy ;
  skos:prefLabel "Unit Test" ;
  skos:definition "Tests a single function/module in isolation" .

rq:IntegrationTest a skos:Concept ;
  skos:inScheme rq:TestHierarchy ;
  skos:prefLabel "Integration Test" ;
  skos:definition "Tests interaction between multiple components" ;
  skos:broader rq:Test .

rq:PropertyBasedTest a skos:Concept ;
  skos:inScheme rq:TestHierarchy ;
  skos:prefLabel "Property-Based Test" ;
  skos:definition "Tests invariants using proptest or quickcheck" ;
  skos:notation "PBT" .

rq:InvariantTest a skos:Concept ;
  skos:inScheme rq:TestHierarchy ;
  skos:prefLabel "Invariant Test" ;
  skos:definition "Tests that an assertion holds across transformations" ;
  skos:broader rq:PropertyBasedTest .

# Pandoc: use PROV-O for test execution tracking
rq:TestRun a prov:Activity ;
  rdfs:label "Test Run" ;
  rdfs:comment "Execution of a test suite at a specific time" ;
  prov:startedAtTime xsd:dateTime ;
  prov:endedAtTime xsd:dateTime .

rq:TestCase a prov:Entity ;
  rdfs:label "Test Case" ;
  rdfs:comment "Individual test function (e.g., test_hp_wraparound)" .

rq:TestFailure a prov:Entity ;
  rdfs:label "Test Failure" ;
  rdfs:comment "Failed test result with error message" ;
  prov:wasGeneratedBy rq:TestRun ;
  rq:errorMessage xsd:string ;
  rq:stackTrace xsd:string .

# Quality gates (SHACL constraints)
rq:CoverageGate92 a sh:NodeShape ;
  sh:targetClass rc:RustCrate ;
  sh:property [
    sh:path rc:testCoverage ;
    sh:minInclusive 92.0 ;
    sh:message "Coverage must be ≥92% for all production crates"
  ] .

rq:NoPanicSites a sh:NodeShape ;
  sh:targetClass rc:RustCrate ;
  sh:property [
    sh:path rq:panicSiteCount ;
    sh:maxInclusive 0 ;
    sh:message "Production crates must have zero unsafe panic sites"
  ] .

rq:ErrorHandling a sh:NodeShape ;
  sh:targetClass rc:RustCrate ;
  sh:property [
    sh:path rq:usesThiserror ;
    sh:hasValue true ;
    sh:message "Must use thiserror for typed domain errors"
  ] .

# Property-based testing vocabulary (Gap #1)
rq:PropTestInvariant a owl:Class ;
  rdfs:label "Proptest Invariant" ;
  rdfs:comment "Assertion that must hold across generated inputs" ;
  prov:wasGeneratedBy rq:PropertyBasedTest .

rq:ShrinkingStrategy a owl:Class ;
  rdfs:label "Shrinking Strategy" ;
  rdfs:comment "Algorithm to minimize failing test cases" .

rq:shrinkingDepth a owl:DatatypeProperty ;
  rdfs:domain rq:PropTestInvariant ;
  rdfs:range xsd:integer ;
  rdfs:comment "How many iterations of shrinking performed" .

rq:hypothesisCount a owl:DatatypeProperty ;
  rdfs:domain rq:PropTestInvariant ;
  rdfs:range xsd:integer ;
  rdfs:comment "Number of test cases generated (proptest default 256)" .

rq:coverageDirective a owl:ObjectProperty ;
  rdfs:domain rq:PropTestInvariant ;
  rdfs:range rq:ShrinkingStrategy ;
  rdfs:comment "Coverage-guided shrinking (e.g., focus on boundary values)" .

# Mutation testing vocabulary (Gap #6)
rq:Mutation a owl:Class ;
  rdfs:label "Code Mutation" ;
  rdfs:comment "Intentional code change to test robustness" .

rq:MutationOperator a owl:Class ;
  rdfs:label "Mutation Operator" ;
  rdfs:comment "Type of change (e.g., delete line, flip boolean, replace constant)" .

rq:mutationOperator a owl:ObjectProperty ;
  rdfs:domain rq:Mutation ;
  rdfs:range rq:MutationOperator .

rq:killedBy a owl:ObjectProperty ;
  rdfs:domain rq:Mutation ;
  rdfs:range rq:TestCase ;
  rdfs:comment "Which test(s) detected this mutation" .

rq:survives a owl:DatatypeProperty ;
  rdfs:domain rq:Mutation ;
  rdfs:range xsd:boolean ;
  rdfs:comment "True if no test caught the mutation (bad)" .

# Defect root-cause ontology (Gap #5)
rq:Defect a owl:Class ;
  rdfs:label "Defect" ;
  rdfs:comment "Root cause of a bug or test failure" .

rq:rootCause a owl:ObjectProperty ;
  rdfs:domain rq:Defect ;
  rdfs:range rq:Defect ;
  rdfs:comment "Chain of causation (e.g., null → panic → corruption)" .

rq:affectedCrates a owl:ObjectProperty ;
  rdfs:domain rq:Defect ;
  rdfs:range rc:RustCrate ;
  rdfs:comment "Which crates are impacted by this defect" .

rq:severity a owl:DatatypeProperty ;
  rdfs:domain rq:Defect ;
  rdfs:range xsd:string ;
  rdfs:comment "Critical, High, Medium, Low, Trivial" .

# Data properties
rq:errorMessage a owl:DatatypeProperty ;
  rdfs:range xsd:string .

rq:stackTrace a owl:DatatypeProperty ;
  rdfs:range xsd:string .

rq:panicSiteCount a owl:DatatypeProperty ;
  rdfs:range xsd:integer .

rq:usesThiserror a owl:DatatypeProperty ;
  rdfs:range xsd:boolean .
```

**Integration Points:**
- Extends DQV for quality measurement
- Uses SKOS for test taxonomy
- Links to PROV-O for test execution lineage
- SHACL shapes enforce 92% coverage gate
- Feeds into QB (Data Cube) for multi-dimensional metrics

---

### 6. rocket-craft-architecture.owl

**Purpose:** 7 workspaces, 44 crates, 6 games as ArchiMate/FOAF model + deployment topology

**Namespace:** `http://rocket-craft.org/ontology/architecture#`

**PURL:** `http://purl.obolibrary.org/obo/ROCKET_ARCH_`

**Key Classes:**
```turtle
@prefix ra: <http://rocket-craft.org/ontology/architecture#> .
@prefix archi: <http://purl.org/archimate/3.0/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

# Workspace structure
ra:Workspace a owl:Class ;
  rdfs:label "Rust Workspace" ;
  rdfs:comment "Independent Cargo workspace with shared resolver" .

ra:ToolsWorkspace a ra:Workspace ;
  rdfs:label "tools/" ;
  ra:resolver "3" ;
  ra:crateCount 5 ;
  ra:crates ra:RocketSdk, ra:RocketCmd, ra:Knhk, ra:Unrdf, ra:UnTestUtils .

ra:NexusEngineWorkspace a ra:Workspace ;
  rdfs:label "nexus-engine/" ;
  ra:resolver "2" ;
  ra:crateCount 10 ;
  ra:description "Gundam Nexus game engine" ;
  ra:dependencyOrder [
    rdf:_1 ra:NexusTypes ;
    rdf:_2 ra:NexusSession ;
    rdf:_3 ra:NexusCombat ;
    rdf:_4 ra:NexusNet ;
    rdf:_5 ra:NexusIntegration
  ] .

ra:BlueprintRsWorkspace a ra:Workspace ;
  rdfs:label "blueprint-rs/" ;
  ra:resolver "2" ;
  ra:crateCount 4 .

ra:UnifyRsWorkspace a ra:Workspace ;
  rdfs:label "unify-rs/" ;
  ra:resolver "2" ;
  ra:crateCount 17 ;
  ra:description "Semantic Web/MCP layer" .

ra:IB4Workspace a ra:Workspace ;
  rdfs:label "infinity-blade-4/mud/" ;
  ra:resolver "2" ;
  ra:crateCount 6 ;
  ra:description "MUD backend server" .

# Crates
ra:RocketSdk a prov:Entity ;
  rdfs:label "rocket-sdk" ;
  ra:workspace ra:ToolsWorkspace ;
  ra:dependsOn ra:UnTestUtils ;
  ra:linesOfCode 3200 ;
  ra:testCoverage 78.5 .

ra:NexusTypes a prov:Entity ;
  rdfs:label "nexus-types" ;
  ra:workspace ra:NexusEngineWorkspace ;
  ra:internalDeps 0 ;
  ra:linesOfCode 1800 ;
  ra:testCoverage 91.2 ;
  ra:comment "Zero-dependency root; phantom-typed units and IDs" .

ra:NexusCombat a prov:Entity ;
  rdfs:label "nexus-combat" ;
  ra:workspace ra:NexusEngineWorkspace ;
  ra:dependsOn ra:NexusTypes ;
  ra:linesOfCode 4500 ;
  ra:testCoverage 89.1 ;
  ra:comment "CombatMachine<S> typestate; combo system; parry/dodge" .

ra:NexusNet a prov:Entity ;
  rdfs:label "nexus-net" ;
  ra:workspace ra:NexusEngineWorkspace ;
  ra:dependsOn ra:NexusTypes ;
  ra:linesOfCode 3800 ;
  ra:testCoverage 85.7 ;
  ra:comment "Connection<S> typestate; duel matchmaking" .

ra:UnifyRdf a prov:Entity ;
  rdfs:label "unify-rdf" ;
  ra:workspace ra:UnifyRsWorkspace ;
  ra:linesOfCode 2600 ;
  ra:testCoverage 62.3 ;
  ra:criticality "HIGH" ;
  ra:comment "RDF triple store, SPARQL, SHACL validation" .

ra:UnifyMcp a prov:Entity ;
  rdfs:label "unify-mcp" ;
  ra:workspace ra:UnifyRsWorkspace ;
  ra:dependsOn ra:UnifyRdf ;
  ra:linesOfCode 1900 ;
  ra:testCoverage 71.2 ;
  ra:comment "JSON-RPC MCP server; tool/resource registries" .

ra:Knhk a prov:Entity ;
  rdfs:label "knhk" ;
  ra:workspace ra:ToolsWorkspace ;
  ra:linesOfCode 1400 ;
  ra:testCoverage 45.0 ;
  ra:criticality "MEDIUM" ;
  ra:comment "Semantic law enforcement via Wasmer WASM" .

# Game projects
ra:ShooterGame a prov:Entity ;
  rdfs:label "ShooterGame" ;
  ra:engineVersion "4.24.3" ;
  ra:platforms ra:Win64, ra:HTML5 ;
  ra:targets ra:ShooterGameEditor, ra:ShooterClient, ra:ShooterGame, ra:ShooterServer .

ra:SurvivalGame a prov:Entity ;
  rdfs:label "SurvivalGame" ;
  ra:engineVersion "4.24.3" ;
  ra:platforms ra:Win64, ra:Android, ra:HTML5 ;
  ra:targets ra:SurvivalGameEditor, ra:SurvivalGameServer, ra:SurvivalGame .

ra:InfinityBlade4 a prov:Entity ;
  rdfs:label "InfinityBlade4" ;
  ra:engineVersion "4.24.3" ;
  ra:platforms ra:iOS, ra:Android, ra:Win64 ;
  ra:targets ra:InfinityBlade4, ra:InfinityBlade4Editor .

# Platform constraints
ra:Win64 a ra:Platform ;
  rdfs:label "Windows 64-bit" ;
  ra:code "Win64" .

ra:Android a ra:Platform ;
  rdfs:label "Android" ;
  ra:code "Android" .

ra:iOS a ra:Platform ;
  rdfs:label "Apple iOS" ;
  ra:code "iOS" .

ra:HTML5 a ra:Platform ;
  rdfs:label "WebGL/HTML5" ;
  ra:code "HTML5" ;
  ra:incompatibleFeatures ra:ApexDestruction, ra:ProceduralMeshComponent ;
  ra:networkPort 8889 ;
  ra:transportProtocol "WebSocket" .

ra:PlatformConstraint a owl:Class ;
  rdfs:label "Platform Constraint" ;
  rdfs:comment "Feature not available on specific platform" .

ra:ApexDestruction a ra:PlatformConstraint ;
  rdfs:label "Apex Physics Destruction" ;
  ra:unsupportedOn ra:HTML5 .

ra:ProceduralMeshComponent a ra:PlatformConstraint ;
  rdfs:label "Procedural Mesh Component" ;
  ra:unsupportedOn ra:HTML5 .

# Dependency properties
ra:dependsOn a owl:ObjectProperty ;
  rdfs:domain prov:Entity ;
  rdfs:range prov:Entity ;
  rdfs:comment "Direct dependency relationship" ;
  owl:transitiveProperty true .

ra:workspace a owl:ObjectProperty ;
  rdfs:domain prov:Entity ;
  rdfs:range ra:Workspace .

ra:platforms a owl:ObjectProperty ;
  rdfs:domain prov:Entity ;
  rdfs:range ra:Platform .

ra:incompatibleFeatures a owl:ObjectProperty ;
  rdfs:domain ra:Platform ;
  rdfs:range ra:PlatformConstraint ;
  rdfs:comment "Features not supported on this platform" .

# Data properties
ra:resolver a owl:DatatypeProperty ;
  rdfs:range xsd:string ;
  rdfs:comment "Cargo workspace resolver version (2 or 3)" .

ra:crateCount a owl:DatatypeProperty ;
  rdfs:range xsd:integer .

ra:internalDeps a owl:DatatypeProperty ;
  rdfs:range xsd:integer ;
  rdfs:comment "Number of internal workspace dependencies" .

ra:linesOfCode a owl:DatatypeProperty ;
  rdfs:range xsd:integer .

ra:testCoverage a owl:DatatypeProperty ;
  rdfs:range xsd:double ;
  rdfs:comment "Percentage 0.0-100.0" .

ra:criticality a owl:DatatypeProperty ;
  rdfs:range xsd:string ;
  rdfs:comment "CRITICAL, HIGH, MEDIUM, LOW" .

ra:engineVersion a owl:DatatypeProperty ;
  rdfs:range xsd:string ;
  rdfs:comment "UE4 engine version (e.g., 4.24.3)" .

# ArchiMate views (organizational layers)
ra:ApplicationArchitecture a archi:ApplicationArchitecture ;
  rdfs:label "Application Architecture" ;
  ra:contains ra:NexusEngineWorkspace, ra:BlueprintRsWorkspace, ra:UnifyRsWorkspace .

ra:TechnologyArchitecture a archi:TechnologyArchitecture ;
  rdfs:label "Technology Architecture" ;
  ra:consists ra:ToolsWorkspace, ra:IB4Workspace .

# SHACL shape: Crate dependencies must be acyclic
ra:AcyclicDependencyGraph a sh:NodeShape ;
  sh:targetClass prov:Entity ;
  sh:property [
    sh:path ra:dependsOn ;
    sh:nodeKind sh:IRI ;
    sh:message "Dependencies must not form cycles"
  ] .

# SHACL shape: Platform constraints must be respected
ra:PlatformCompatibility a sh:NodeShape ;
  sh:targetClass prov:Entity ;
  sh:property [
    sh:path ra:platforms ;
    sh:not [
      sh:hasValue ra:HTML5 ;
      sh:qualifiedValueShape [
        sh:path ra:incompatibleFeatures ;
        sh:hasValue ra:ApexDestruction
      ]
    ] ;
    sh:message "Cannot target HTML5 if using Apex Destruction"
  ] .
```

**Integration Points:**
- ArchiMate 3.0 for enterprise architecture views
- FOAF for team/contributor tracking (extensible)
- Links to all 5 previous ontologies (core, states, types, manifest, quality)
- SHACL shapes verify: acyclic dependency graph, platform constraint consistency, resolver version alignment

---

## Part 4: Integration Strategy & Implementation Roadmap

### Phase 4.0: Foundation (Weeks 1-2)

**Deliverables:**
1. Publish 6 ontologies to PURL namespace (http://purl.obolibrary.org/obo/ROCKET_*)
2. Create JSON-LD contexts for each ontology
3. Generate SHACL shape definitions
4. Write SPARQL query library (50+ example queries)

**Code Generation (ggen):**
```bash
./rocket ggen --spec rocket-craft-manifest.owl --target nexus-types/src/lib.rs
# Generates phantom-type definitions with QUDT bounds validation
```

### Phase 4.1: Manifest Integration (Week 3)

**Deliverables:**
1. Auto-export project-manifest.json to rocket-craft-manifest.ttl (./rocket sync)
2. SHACL validation via ./rocket audit
3. SPARQL queries for dependency analysis

### Phase 4.2: Quality Gates (Weeks 4-5)

**Deliverables:**
1. CI/CD integration: GitHub Actions emits QB observations
2. Pre-merge SPARQL queries enforce 92% coverage
3. Dashboard: SPARQL endpoint (./rocket serve-rdf :7878)

### Phase 4.3: Typestate Formalization (Weeks 6-7)

**Deliverables:**
1. Auto-generate rocket-craft-states.ttl from Rust code
2. SPARQL verification: "No illegal state transitions"
3. Formal semantics proof (optional: Z3 SMT solver integration)

### Phase 4.4: Testing Vocabulary (Weeks 8-10)

**Deliverables:**
1. proptest ↔ RDF bridge (nexus-tests integration)
2. Mutation testing RDF export
3. Defect taxonomy RDF generation

### Phase 4.5: DevOps Integration (Weeks 11-15)

**Deliverables:**
1. GitHub Actions ↔ OCEL 2.0 event export
2. Artifact SBOM generation (SPDX)
3. Cross-workspace dependency visualization (SPARQL queries)

---

## Part 5: Critical Success Factors

1. **Namespace Registration:** Mint PURL `http://purl.obolibrary.org/obo/ROCKET` early (Week 1)
2. **Community Alignment:** OBO Foundry membership (optional but validates design)
3. **Rust Ecosystem Integration:** Ensure zero-cost abstractions (derive macros for RDF generation)
4. **Retroactive Compatibility:** All 6 ontologies must ingest existing project-manifest.json without code changes
5. **Query Performance:** SPARQL endpoint must return 92% coverage gate result in <100ms

---

## Conclusion

The 6 custom ontologies form a layered semantic foundation enabling:
- **Deterministic code generation** (ggen + RDF specification)
- **Compile-time state safety** formalized in RDF
- **Domain-specific type system** (phantom types + QUDT bounds)
- **Test-driven CI/CD** (SPARQL-enforced gates, OCEL event logs)
- **Supply chain transparency** (SPDX + PROV-O lineage)

**Next Step:** Implement Phase 4.0 (weeks 1-2) to publish ontologies and establish PURL namespace.
