# Rocket-Craft x ggen Marketplace Integration Strategy

**Vision:** Rocket-Craft as the Premier Game Generation Platform in the ggen Marketplace

**Date:** 2026-06-18  
**Status:** Strategic Plan — Ready for Implementation  

---

## Executive Summary

Rocket-Craft transforms from a monorepo of games into a **deterministic game generation platform** integrated with ggen's marketplace infrastructure.

**The Model:**

```
Game Specification (RDF Ontology)
    ↓
ggen μ₁-μ₅ Pipeline (Load → Extract → Generate → Merge → Emit)
    ↓
8 Canonical Proof Gates (Schema, Ontology, Projection, Compilation, Receipt, Ethos, Observability, Causality)
    ↓
Production Game Code (UE4 Backend + HTML5 Frontend + Servers)
    ↓
Cryptographic Receipt (Proof of Deterministic Generation)
```

**Market Opportunity:**

- **No-code game development** for designers, educators, indie studios
- **Marketplace of 1000+ games** generated from verified specifications
- **Revenue:** 30% take of F2P monetization, premium ontology subscriptions
- **First-mover advantage:** Only deterministic game generation platform

---

## Part 1: ggen Architecture Overview

### Five-Stage Pipeline (μ₁-μ₅)

| Stage | Name | Input | Output | Role |
|-------|------|-------|--------|------|
| μ₁ | Load Ontology | RDF/Turtle file | Oxigraph triple store | Parse and normalize domain model |
| μ₂ | Extract | SPARQL CONSTRUCT | Derived RDF graph | Infer domain patterns via queries |
| μ₃ | Generate | Tera templates | Source code files | Render code from extracted data |
| μ₄ | Merge & Validate | Generated files + SHACL shapes | Validated artifact set | Quality gates & consistency checks |
| μ₅ | Emit | Validated artifacts | Cryptographic receipt | Proof of deterministic generation |

### 8 Canonical Proof Gates

1. **Schema Gate** — RDF conforms to SHACL shape definitions
2. **Ontology Gate** — Semantic consistency (no contradictions)
3. **Projection Gate** — Generated code is faithful to ontology (completeness check)
4. **Compilation Gate** — Code compiles without errors
5. **Receipt Gate** — Cryptographic signature validates provenance
6. **Ethos Gate** — Code follows style/quality conventions
7. **Observability Gate** — Traces and metrics exported correctly
8. **Causality Gate** — Dependency graphs acyclic, no circular imports

### ggen Marketplace Architecture

**Key Components:**
- **ggen-marketplace crate** — Storefront, ontology registry, payment processing
- **ggen-config** — Manifest-driven configuration (ggen.toml)
- **ggen-graph** — Dependency graphs, impact analysis
- **ggen-a2a-mcp** — Agent-to-Agent protocol for multi-agent orchestration
- **ggen-lsp** — Language Server Protocol for IDE integration
- **genesis-core-v2** — Pure Rust kernel (IO-free, wasm-compatible)

### ggen Philosophy

1. **Specification-First (Big Bang 80/20)** — 80% of code generation comes from specifying the domain correctly
2. **Deterministic Validation** — Same spec → Same code, always
3. **RDF-First** — Ontologies are the single source of truth
4. **Evidence Replaces Narrative** — Cryptographic receipts prove generation, not documentation

---

## Part 2: Rocket-Craft as ggen Marketplace Application

### Current State

Rocket-Craft has:
- 6 custom domain-specific ontologies (rocket-craft-{core,states,types,manifest,quality,architecture}.owl)
- 44 Rust crates across 7 workspaces
- 6 UE4 game projects (proof-of-concept implementations)
- 50+ SPARQL queries for validation
- 92%+ test coverage enforcement via quality gates

### Integration Strategy

**Layer 1: Ontology Specifications**
```
Game Spec (RDF Ontology)
├── game-mechanics.ttl (Combat, economy, progression)
├── game-balance.ttl (Monte Carlo metrics, damage/healing)
├── game-platform.ttl (Target platforms, HTML5 constraints)
└── game-assets.ttl (3D models, textures, audio)
```

**Layer 2: ggen μ₁-μ₅ Pipeline**
```
μ₁: Load game ontology into Oxigraph
μ₂: Extract via SPARQL queries:
    - SELECT game mechanics properties
    - CONSTRUCT derived game systems
    - CONSTRUCT economy balancing rules
μ₃: Generate via Tera templates:
    - Rust backend (nexus-engine systems)
    - TypeScript frontend (HTML5 client)
    - Server orchestration (Docker Compose)
μ₄: Validate via 8 proof gates:
    - SHACL: Game mechanics shape compliance
    - Compilation: Rust code compiles, TS type-checks
    - Receipt: Cryptographic signature
μ₅: Emit receipt + generated artifacts
```

**Layer 3: Marketplace Storefront**
```
Game Registry
├── Published Games (vetted, tested, deployed)
├── Draft Specifications (under development)
├── Leaderboards (SPARQL-queryable via federated endpoints)
└── Analytics (coverage metrics, player engagement)
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     ggen Marketplace                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Rocket-Craft Game Generation                   │  │
│  │                                                           │  │
│  │  Spec Editor          Pipeline           Deployment      │  │
│  │  (Web UI)          (ggen-core)         (CI/CD)           │  │
│  │                                                           │  │
│  │  Game.ttl    →  μ₁ Load  →  Oxigraph                    │  │
│  │  (Ontology)     μ₂ Extract → SPARQL                      │  │
│  │                 μ₃ Generate → Tera                       │  │
│  │  Templates      μ₄ Validate → SHACL + Proof Gates       │  │
│  │  (Tera)         μ₅ Emit     → Receipt + Code            │  │
│  │                                                           │  │
│  │  Output:        UE4 Backend                              │  │
│  │  - Rust code    HTML5 Client                             │  │
│  │  - TypeScript   Servers                                  │  │
│  │  - SPDX SBOM    Docker Compose                           │  │
│  │  - Receipt      (deployed to game.rocket-craft.io)       │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Payment Processing   Analytics   Community                     │
│  (Stripe)            (SPARQL)     (Discord)                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Part 3: Implementation Roadmap (16 Weeks)

### Phase 5.0: ggen Integration Foundation (Weeks 1-2)

**Deliverables:**
- [ ] Link ggen as git dependency in rocket-craft Cargo.toml
- [ ] Create rocket-craft-games crate (marketplace application layer)
- [ ] Port 6 custom ontologies into ggen-compatible format
- [ ] Implement game.ttl ontology template

**Code Changes:**
```toml
# rocket-craft/Cargo.toml
[dependencies]
ggen-core = { git = "https://github.com/seanchatmangpt/ggen.git", rev = "..." }
ggen-marketplace = { git = "https://github.com/seanchatmangpt/ggen.git" }

[dev-dependencies]
ggen-cli-lib = { git = "https://github.com/seanchatmangpt/ggen.git" }
```

**Effort:** 40 hours (architecture, dependency alignment, type compatibility)

### Phase 5.1: Tera Template Pipeline (Weeks 3-5)

**Deliverables:**
- [ ] Tera templates for Rust backend generation (nexus-engine systems)
- [ ] Tera templates for TypeScript frontend (HTML5 client)
- [ ] Tera templates for server orchestration (Kubernetes/Docker)
- [ ] Template testing via ggen's template validator

**Example Template:**
```tera
# templates/nexus-combat.rs.tera
// Generated from {{ game_name }} game spec
use {{ workspace }}::nexus_combat::{CombatMachine, Idle, Attacking};

#[derive(Debug)]
pub struct {{ game_name }}Combat {
    machine: CombatMachine<Idle>,
    {% for ability in game_abilities %}
    {{ ability.field_name }}: {{ ability.rust_type }},
    {% endfor %}
}
```

**Effort:** 80 hours (template design, validation, edge cases)

### Phase 5.2: SPARQL Extraction Queries (Week 6)

**Deliverables:**
- [ ] 30+ SPARQL CONSTRUCT queries for game pattern extraction
- [ ] Query library for:
  - Combat systems (attacks, damage calculation, combo chains)
  - Economy systems (currency, trading, inflation controls)
  - Progression systems (leveling, unlocks, achievements)
  - Platform constraints (HTML5 incompatibilities)

**Example Query:**
```sparql
PREFIX game: <http://rocket-craft.org/ontology/game#>
PREFIX rc: <http://rocket-craft.org/ontology/core#>

CONSTRUCT {
  ?ability game:damageFormula ?formula ;
           game:cooldownMs ?cooldown ;
           game:scalingFactor ?scaling .
}
WHERE {
  ?game a game:GameSpecification ;
        game:hasAbility ?ability .
  ?ability rc:usesPhantomType rc:Damage ;
           game:formula ?formula ;
           game:cooldown ?cooldown .
  OPTIONAL { ?ability game:scalingFactor ?scaling . }
}
```

**Effort:** 60 hours (SPARQL expertise, ontology alignment)

### Phase 5.3: 8 Proof Gates Implementation (Weeks 7-8)

**Deliverables:**
- [ ] Implement 8 canonical proof gates for game generation
- [ ] Gate 1: Schema (SHACL shapes for game specifications)
- [ ] Gate 2: Ontology (semantic consistency checks)
- [ ] Gate 3: Projection (generated code completeness)
- [ ] Gate 4: Compilation (Rust compile + TS type-check)
- [ ] Gate 5: Receipt (cryptographic signature generation)
- [ ] Gate 6: Ethos (code style, naming conventions)
- [ ] Gate 7: Observability (OTEL span validation)
- [ ] Gate 8: Causality (dependency acyclicity)

**Effort:** 100 hours (complex validation logic, edge case handling)

### Phase 5.4: Marketplace Registry Integration (Weeks 9-10)

**Deliverables:**
- [ ] Register rocket-craft-games as ggen marketplace app
- [ ] Implement game spec upload/parsing API
- [ ] Create game listing page with ontology metadata
- [ ] Hook into ggen-marketplace payment processor

**Effort:** 80 hours (API design, payment integration, UI)

### Phase 5.5: Game Deployment Automation (Weeks 11-13)

**Deliverables:**
- [ ] CI/CD pipeline for generated games
- [ ] Docker containerization of game backends
- [ ] Kubernetes deployment manifests (auto-generated)
- [ ] HTML5 CDN deployment (game.rocket-craft.io/{game-id})
- [ ] SPDX SBOM generation for each game

**Effort:** 100 hours (DevOps, automation, security)

### Phase 5.6: Production Marketplace Launch (Weeks 14-16)

**Deliverables:**
- [ ] Spec editor web UI
- [ ] Game showcase gallery
- [ ] Leaderboards (SPARQL-backed)
- [ ] Payment processing (Stripe integration)
- [ ] Community features (forums, Discord bot)
- [ ] Analytics dashboard (player engagement, revenue)

**Effort:** 120 hours (frontend, backend APIs, third-party integrations)

**Total Phase 5 Effort:** 580 hours (14.5 weeks @ 40h/week)

---

## Part 4: Game Generation Examples

### Example 1: Puzzle Game (Bloons-style)

**Input:** `bloons-spec.ttl` (Game ontology)
```turtle
@prefix game: <http://rocket-craft.org/ontology/game#> .

:BloonsGame a game:GameSpecification ;
  game:title "Bloons Tower Defense" ;
  game:genre game:TowerDefense ;
  game:platforms game:HTML5, game:Android, game:iOS ;
  game:mechanics [
    game:hasUnit :Dart ;
    game:hasUnit :Bomb ;
    game:hasUnit :Glue ;
    game:hasUnit :MonkeyEngineer ;
  ] ;
  game:economy [
    game:currency :BananaPoints ;
    game:incomePerWave 100 ;
    game:costPerUnit :Dart 100 ;
  ] ;
  game:progression [
    game:maxLevel 100 ;
    game:unlockMechanic game:ProgressiveUnlock ;
  ] .
```

**ggen μ₁-μ₅ Output:**
- **μ₁:** RDF parsed into Oxigraph
- **μ₂:** SPARQL extracts tower stats, wave progression, economy rules
- **μ₃:** Tera renders nexus-economy balance rules, nexus-ecs tower systems
- **μ₄:** SHACL validates tower damage/cost ratios (e.g., expensive towers do more damage)
- **μ₅:** Receipt emitted: `receipt.json` with cryptographic signature

**Result:** Fully playable Bloons clone, deployed to `bloons-2026.rocket-craft.io`

### Example 2: RPG Game

**Input:** `rpg-spec.ttl`
```turtle
:MyRPGGame a game:GameSpecification ;
  game:title "Adventure Quest" ;
  game:mechanics [
    game:hasCombatSystem game:TurnBased ;
    game:hasCharacterClass :Warrior ;
    game:hasCharacterClass :Mage ;
    game:hasCharacterClass :Rogue ;
  ] ;
  game:progression [
    game:experienceFormula "log(enemy_level * 100)" ;
    game:skillTree [
      game:hasSkill :FireBolt ;
      game:hasSkill :IceWall ;
    ] ;
  ] .
```

**Output:** Full RPG backend with:
- Character class systems (nexus-types phantom types)
- Combat state machine (nexus-combat CombatMachine<S>)
- Experience curves (nexus-economy Monte Carlo balancing)
- Skill progression (nexus-session state tracking)
- Multiplayer support (nexus-net WebSocket connection state)

---

## Part 5: Revenue Model & Unit Economics

### Monetization

| Channel | Rate | Volume (Y1) | Revenue |
|---------|------|-----------|---------|
| Game Sales (F2P take) | 30% | 1,000 games × $50k avg | $15M |
| Premium Ontologies | $99/mo | 500 creators | $594k |
| White-Label SaaS | $5k/mo | 50 studios | $3M |
| LLM Training Data | $1M+ | Game specs as ML corpus | $1M+ |
| **Total Y1** | | | **$19.6M+** |

### CAC/LTV

- **Creator Acquisition:** Dev blog, Game Dev conferences, Reddit (low CAC)
- **Player Acquisition:** Per-game marketing (game devs handle)
- **LTV:** 30% of game revenue (recurring, 5-year horizon)

---

## Part 6: Critical Success Factors

1. **Deterministic Pipeline** ✅
   - Same spec always generates same code
   - Proof gates ensure quality
   - Cryptographic receipts prove provenance

2. **Template Quality** 🔑
   - Tera templates must cover 80+ game genres
   - Edge cases for platform constraints (HTML5, mobile)
   - Performance optimization for multiplayer

3. **Marketplace UX** 🔑
   - Spec editor must be accessible to non-programmers
   - One-click game deployment
   - Integrated leaderboards/analytics

4. **Community Network Effects** 🔑
   - Reusable ontology modules (published as packages)
   - Open-source game templates
   - Creator marketplace (custom assets, music, etc.)

5. **Regulatory Alignment** ⚠️
   - Game rating systems (ESRB, PEGI)
   - Child safety (COPPA compliance)
   - Payment processing (PCI DSS)

---

## Part 7: Competitive Advantages

| Aspect | Rocket-Craft + ggen | Traditional Game Engines | No-Code Platforms |
|--------|-------------------|--------------------------|-------------------|
| Code Generation | Deterministic, RDF-based | Manual coding | Limited, template-based |
| Quality Gates | 8 proof gates, cryptographic receipts | Dev discipline | None |
| Marketplace | Native (ggen-marketplace) | Asset stores only | Walled garden |
| Multiplatform | UE4 + HTML5 + native (automatic) | Manual porting | Web-only |
| Monetization | 30% F2P + subscriptions | 100% engine cut | 30-50% platform cut |
| **Differentiation** | **Only spec-driven, verifiable game generation platform** | | |

---

## Part 8: Immediate Next Steps

**This Week:**
1. ✅ Understand ggen architecture (DONE)
2. ⚡ Fork ggen-marketplace, rename to rocket-craft-games
3. ⚡ Link ggen in rocket-craft Cargo.toml
4. ⚡ Port 6 custom ontologies into Turtle files

**Next 2 Weeks:**
1. Implement first Tera template (Rust backend for combat)
2. Write 5 SPARQL extraction queries (game mechanics)
3. Design proof gate #1 (SHACL schema validation)
4. Create game spec format (ggen.toml + game.ttl)

**Next Month:**
1. Implement all 8 proof gates
2. Full μ₁-μ₅ pipeline working end-to-end
3. Generate one complete game (proof-of-concept)
4. Publish to GitHub with README + examples

---

## Conclusion

Rocket-Craft + ggen is positioned to become the **first deterministic, verifiable game generation platform**. The combination of:

✅ **RDF-first specifications** (6 custom ontologies)  
✅ **ggen's 5-stage pipeline** (deterministic generation)  
✅ **8 canonical proof gates** (quality assurance)  
✅ **ggen marketplace infrastructure** (storefront, payment, registry)  
✅ **Cryptographic receipts** (proof of provenance)  

...creates a defensible moat and first-mover advantage in a nascent market.

**Market timing:** 2026 = perfect window before major game engines release their own AI generation tools.

---

**Document:** GGEN_MARKETPLACE_INTEGRATION_STRATEGY.md  
**Status:** Ready for Architecture Review  
**Next:** Create Phase 5.0 implementation branch  
