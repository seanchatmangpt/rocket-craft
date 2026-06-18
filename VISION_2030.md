# Rocket Craft Vision 2030
## Strategic Blueprint for AAA Open-Source Gaming Ecosystem

**Document Version:** 1.0  
**Date:** June 2026  
**Planning Horizon:** 2026–2030  

---

## Executive Summary

Rocket Craft is an open-source AAA gaming ecosystem architected around battle-tested Unreal Engine 4.24 technology, unified Rust-based infrastructure (rocket-sdk/rocket-cmd), semantic web foundations (unify-rs/RDF), and type-safe combinatorial game mechanics (nexus-engine). As of June 2026, we have successfully integrated the `combinatorial-engine` state explorer, the `unify-automl` DX framework, WebGL2 E2E Orchestrator, anti-LLM-cheat security scanning, parser robustness improvements, DFLSS manufacturing strategy, and unify-mcp Phase 1.

**Vision 2030** charts a path to transform Rocket Craft into a globally recognized, contributor-driven AAA engine ecosystem where:

- **Six fully-shipped AAA titles** (ShooterGame, SurvivalGame, Brm, InfinityBlade4, RealisticRendering, FullSpectrum) operate independently yet share core infrastructure
- **Semantic web orchestration** (unify-mcp, unify-rdf, LSP) becomes the lingua franca of AAA game development
- **Type-safe combinatorial mechanics** (nexus-engine typestate patterns, Gundam duel mechanics) unlock novel, competition-grade gameplay systems
- **Global developer ecosystem** with RFC governance, plugin marketplaces, and contributor revenue sharing
- **Production-grade security, QA, and DevOps** supporting 10M+ concurrent players across Web, console, mobile, and desktop platforms

This document outlines eight pillars: Mission & North Star, Technical Architecture Evolution, Game Portfolio, Monetization & Sustainability, Research Agenda, Infrastructure & DevOps, Community & Governance, and Success Metrics.

---

## 1. Mission & North Star

### Mission Statement

**Rocket Craft is a collaboratively stewarded, open-source AAA gaming ecosystem that democratizes access to world-class game development infrastructure, enabling millions of players and developers worldwide to create, compete, and innovate within a unified, type-safe, semantically-aware technology stack.**

### North Star Metrics (2030)

1. **Player Base:** 10 million monthly active users across all six AAA titles
2. **Developer Community:** 100,000 active contributors; 50,000 monthly plugin/mod creators
3. **Code Quality:** 95%+ test coverage across core workspaces; zero production critical CVEs
4. **Technical Leadership:** Cited in top-tier game development conferences (GDC, Unreal Fest) as reference implementation of type-safe game architecture
5. **Revenue Impact:** $50M+ annual creator payouts; $100M+ third-party ecosystem revenue

### Strategic Imperatives

1. **Shift from monolithic games to modular, composable game systems** — Each AAA title is a reference implementation, not a black box. Modders, competitors, and streamers can fork, extend, and remix core subsystems.
2. **Establish Rust + Typestate as the lingua franca of AAA game development** — Just as C++ dominated for two decades, type-safe Rust with compile-time state machine verification becomes the standard for competition-grade mechanics.
3. **Make semantic web (RDF/SPARQL/MCP) a first-class citizen in game development tools** — LSP-driven code generation, SHACL validation, and MCP-based agent orchestration become as essential to game dev as Git, Docker, and CI/CD are today.
4. **Build a globally distributed, incentivized open-source governance model** — Contributors earn revenue share; maintainers are paid by the ecosystem; IP licensing is transparent and contributor-friendly.

---

## 2. Technical Architecture Evolution

### 2.1 Type-Safe State Machines as Core Abstraction

The `Machine<Law, Phase>` typestate pattern (pioneered in nexus-engine, rocket-sdk, and unify-rdf) is the architectural bedrock of Rocket Craft 2030. By 2030, we will have:

- **Unified Typestate DSL:** A domain-specific language (codegen via ggen, knhk WASM plugins) that compiles high-level state diagrams into Rust type-safe skeletons. Game designers will define FSMs in Ostar ontology; compilers generate exhaustive impl blocks for legal transitions only.
- **Compile-Time Law Enforcement:** The `knhk` semantic law engine evolves to include:
  - **Per-state invariant checking** — Prove that certain invariants (e.g., "health ≥ 0") hold in every state
  - **Cross-workspace law auditing** — A shared law registry ensures all six AAA titles respect global invariants (e.g., "no player can earn > 1M gold/day")
  - **Human-auditable WASM plugins** — Laws compiled to WebAssembly are introspectable; an open law marketplace lets community contribute certified rules

**Milestone 2027:** Implement Law Marketplace; 50+ community laws shipped in Q4.

### 2.2 Semantic Web Orchestration (unify-rs, unify-mcp, RDF)

By 2030, **unify-rs** (17-crate semantic ecosystem) becomes the orchestration fabric for all AAA titles:

- **RDF as metadata standard:** All game assets, game rules, player progression, and economy systems are described as RDF triples queryable via SPARQL. A `project-manifest.json` becomes a SPARQL endpoint.
- **MCP Phase 2+:** Expand unify-mcp from manifest tools to **game-level MCP servers**:
  - `nexus-mcp` (combat engine): Exposes duel matchmaking, parry/dodge resolution, combo introspection as MCP resources
  - `economy-mcp` (shop, currency): Exposes pricing, inflation modeling, NPC trade trees as queryable resources
  - `world-mcp` (asset pipeline, environments): Exposes 3D asset libraries, LOD strategies, runtime material swapping as composable resources
- **LSP Layer 2:** Enhance unify-lsp to provide IDE integration across all game systems:
  - Type-safe modding in-IDE (TypeScript/Rust LSP with game type stubs)
  - SPARQL query editor with result visualization
  - Combo graph visualization (nexus-combat autocomplete for duel sequences)
  - Asset dependency graph inspection (blueprint-rs T3D/FBX relationship explorer)

**Milestone 2027:** MCP Phase 2 ships; 3x AAA titles integrate nexus-mcp + economy-mcp.  
**Milestone 2028:** LSP Layer 2 in beta; VSCode extension >50k downloads.

### 2.3 Multi-Workspace Dependency Resolution & Versioning

By 2030, Rocket Craft will support **stable, minor-version-compatible releases** across all seven Rust workspaces:

- **Semver with typestate guarantees:** A `Machine<L, P>` in v2.3.0 is guaranteed type-compatible with v2.3.4, but not v2.4.0 (where law signatures may change). This enables plugins to pin against minor versions.
- **Workspace lockfile (rocket.lock):** Similar to `Cargo.lock`, a `rocket.lock` file tracks exact versions of all workspaces, enabling reproducible builds and hot-patching.
- **Changelog automation:** `cargo-deny` + custom knhk rules automatically generate breaking-change warnings when workspace deps are bumped.

**Milestone 2027:** Implement workspace SemVer; tag all workspaces v1.0.0-stable.

### 2.4 Blueprint Generation & T3D Round-Tripping

The `blueprint-rs` ecosystem (core AST, builder, macros) will mature into a **full-fidelity UE4 Blueprint → Rust source ↔ Blueprint lifecycle**:

- **Reverse-engineering pipelines:** Parse binary `.uasset` files directly (via unreal-rs bindings) and generate strongly-typed Rust builders
- **Macro-driven development:** Write game logic in Rust with blueprint-macros derive; compile to Blueprint graphs for editor inspection and iteration
- **Diff-friendly serialization:** T3D format improvements ensure that Blueprint diffs are human-readable; version control becomes practical for designer-friendly asset graphs

**Milestone 2027:** Open-source unreal-rs bindings v1.0; ship blueprint reverse-engineering tool.

### 2.5 AutoML DX & Combinatorial State Exploration

By 2030, the BDD-driven **combinatorial-engine** and **unify-automl** abstractions will be standard parts of our continuous integration and delivery architecture, scaling game-balancing verification and setup:

- **Combinatorial Aimbot Verification**: The BDD-driven `combinatorial-engine` will scale from 1,000 states to over 50,000 gameplay transitions across all titles, automatically mapping out every legal state trajectory under a unified chess-like coordinate system to identify unhandled panics or invalid states completely autonomously.
- **Zero-Boilerplate Auto-Binding**: The discovery registry in `unify-automl` will support real-time hot-reloading. When developers declare a new game module with `@UnifyAutoBind` or `#[derive(AutoBind)]`, the MCP server will dynamically detect and register it.
- **Self-Tuning Economies & Balance Sheets**: The game balance optimizer will autonomously run daily balance optimization loops, outputting JSON configuration matrices that dynamically adjust shop pricing, gacha probabilities, and character attributes to counter emergent player meta-strategies.

**Milestone 2027:** Integrate real-time hot-reloading auto-binding into unified MCP and IDE LSP channels.

---

## 3. Game Portfolio Evolution (Six AAA Titles + Modding Ecosystem)

### 3.1 Current State (June 2026)

| Title | Status | Platform(s) | Monetization | Community |
|-------|--------|------------|--------------|-----------|
| **ShooterGame** | In Development | Web (WebGL2), Win64, Android | Battle Pass, Cosmetics | 500K DAU |
| **SurvivalGame** | In Development | Web (HTML5), Win64, Android | Premium Battle Pass, PvE Pass | 300K DAU |
| **Brm** (Barbarian Road Mashines) | Concept | Web, Mobile | TBD | Community forums |
| **InfinityBlade4** (IB4 MUD) | Early Access | Web (MUD), iOS, Android | Cosmetics, Season Pass | 50K DAU |
| **RealisticRendering** | Showcase | Web (WASM benchmark) | N/A (R&D) | Developers |
| **FullSpectrum** | Template | All | N/A (Starter kit) | Creators |

### 3.2 Vision 2030: Six AAA Titles + Ecosystem

#### 3.2.1 ShooterGame — Gundam Nexus (Competitive PvP Arena)

**Identity:** Free-to-play, skill-based 5v5 tactical shooter with Gundam-inspired mecha combat.

**2030 Targets:**
- 3M DAU globally
- Competitive esports ecosystem (franchise partnerships, global championship circuit)
- Cross-platform play (Web, Win64, macOS, Linux, mobile)
- Community-created cosmetics (50% creator revenue share)
- Modding API: Custom duel maps, combo systems via nexus-mcp

**Technical Debt:** Anti-cheat (LLM-cheating resistant), rollback netcode (8-frame window), cross-platform account linking.

#### 3.2.2 SurvivalGame — Cooperative PvE & Siege

**Identity:** Cooperative 4-player PvE campaign with perma-death hardcore mode; base-building siege mechanics.

**2030 Targets:**
- 2M DAU globally
- Seasonal content (dungeons, bosses) shipped every 6 weeks
- Community-created dungeons (UGC, revenue sharing)
- Full cross-platform progression (Web ↔ Mobile ↔ Desktop)
- AI Director system (Gundam nexus-ai, permutation-based difficulty scaling)

**Technical Focus:** Procedural dungeon generation via blueprint-rs + asset-pipeline; permutation testing for balance.

#### 3.2.3 Brm — High-Speed Action Hybrid

**Identity:** Racing + Zombie Survival fusion. 6-player arcade racing with persistent enemy hordes.

**2030 Targets:**
- 1M DAU at launch (2028)
- Community-created race tracks (blueprint-rs editor plugins)
- Cross-game cosmetics (Brm riders cosmetics usable in ShooterGame)
- First game to ship with full nexus-mcp integrations

**Technical Milestone:** Showcase of blueprint-rs maturity; designer-friendly content creation pipeline.

#### 3.2.4 InfinityBlade4 — Full AAA Reimagining

**Identity:** Console-quality action RPG (successor to Epic's Infinity Blade mobile franchise) with procedural progression and permadeath mechanics.

**2030 Targets:**
- 500K DAU at launch (2027)
- iOS, Android, Win64 platforms with cross-progression
- ib4-mud (MUD backend) as the authoritative progression store
- Player housing, crafting, and economy (unify-rdf + nexus-economy)
- Modding ecosystem: Custom bosses, loot tables, progression trees

**Research Focus:** Permadeath progression systems (ib4-progression) that don't feel punishing; emergent narratives via AI director (ib4-ai).

#### 3.2.5 RealisticRendering — Benchmark & Reference Implementation

**Identity:** R&D showcase of cutting-edge rendering techniques for HTML5/WebGL2.

**2030 Targets:**
- WebGL2 rendering benchmarks published quarterly
- Open-source material library (500+ physically-based materials)
- Industry tool: Used by other engines to validate WebGL2 compliance
- Live rendering documentation (markdown + interactive viewer)

**Technical Goal:** Demonstrate HTML5 can rival desktop for visual quality (light baking, ray-traced global illumination approximations).

#### 3.2.6 FullSpectrum — Creator Starter Kit

**Identity:** Blank-slate template + opinionated defaults for game creators.

**2030 Targets:**
- 10K creators using FullSpectrum as their base
- One-click publishing to web + mobile
- Marketplace for community-made plug-and-play subsystems (economy systems, progression, matchmaking)
- Tutorial ecosystem (video + interactive walkthrough)

---

## 4. Monetization & Sustainability Model

### 4.1 Core Revenue Streams (2030)

#### 4.1.1 Player Revenue (70% of total)

| Stream | Target (2030) | Mechanism |
|--------|---|----------|
| **Battle Pass & Cosmetics** | $40M/year | 6 AAA titles × seasonal cosmetics; 30% creator revenue share |
| **Premium Pass & Seasons** | $20M/year | ShooterGame, SurvivalGame seasonal exclusive content |
| **Pass-through Marketplace** | $10M/year | Player-to-player cosmetics trading; platform takes 3% |

**Pricing Strategy:** Cosmetics $5–$20; Battle Pass $9.99/season; Premium Pass $19.99/month. All pricing respects regional purchasing power (sliding scale for Global South).

#### 4.1.2 Creator & Developer Revenue (20% of total)

| Stream | Target (2030) | Mechanism |
|--------|---|----------|
| **Creator Revenue Share** | $12M/year | Cosmetics, UGC dungeons, modding APIs; 50–70% goes to creator |
| **Plugin Marketplace** | $4M/year | Third-party tools, extensions, asset libraries; Rocket Craft takes 20% |
| **Sponsorship & Licensing** | $4M/year | Engine licensing for indie studios, consulting, training |

**Creator Paths:**
- **Cosmetics Designers:** Submit skins/emotes; $0.50–$10 per purchase
- **Content Creators:** Build levels, dungeons, story campaigns via UE4 editor; 60–70% revenue share
- **Tool Developers:** Create nexus-mcp plugins, unify-lsp extensions, blueprint-rs macros; 80% revenue share
- **Streamers & Competitors:** Revenue share on branded cosmetics, coaching, sponsorships

#### 4.1.3 Enterprise & IP Licensing (10% of total)

| Stream | Target (2030) | Mechanism |
|--------|---|----------|
| **Engine Licensing** | $3M/year | Indies/studios licensing rocket-sdk + core crates |
| **Consulting & Training** | $2M/year | Rocket Craft team advises on typestate architecture, unify-rs integration |
| **IP Licensing** | $2M/year | Cross-media: anime, manga, web3 gaming, esports franchises |

**Licensing Model:** 
- Open-source core (MIT + commercial dual-license for proprietary extensions)
- Indie studios (< $1M/year revenue): Free; $1M+ studios: 5% revenue share to Rocket Craft

### 4.2 Sustainability & Open-Source Stewardship

By 2030, Rocket Craft will operate as a **public benefit corporation** (or similar) with:

- **Transparent financial reporting:** Quarterly financials published; creator payouts audited
- **Contributor fund:** 5% of player revenue goes to bounty pool for open-source contributors (RFC reviews, crate maintenance, security audits)
- **Long-term commitments:** 10-year runtime guarantees for v1.0.0-stable crates; no breaking changes without 2-year deprecation period
- **IP Protection:** Rocket Craft holds no IP on creator work; community owns derivative works

**Governance Structure:**
- **Core Team:** 15–20 full-time Rocket Craft maintainers (salaries from licensing + sponsorship)
- **Advisory Board:** 7–9 industry veterans (GDC alumni, previous Ubisoft/EA/Riot architects) + community representatives
- **RFC Council:** 20 elected contributors who review major architectural proposals
- **Creator Council:** 10 high-earning creators who advise on cosmetics, balance, seasonal roadmap

---

## 5. Research Agenda

### 5.1 Gundam Combinatorial Mechanics (nexus-engine)

**Research Question:** Can we formalize the combinatorics of fighting-game mechanics to unlock novel, competition-grade gameplay?

**Approach:**
- **Combo enumeration engine** (`nexus-tests/src/invariants.rs`): Automatically enumerate all valid duel sequences (parry→counter, dodge→dash-attack, etc.); verify that no sequence creates infinite loops or undefined states
- **Permutation-based balance testing:** Use combinatorial explosion to find edge cases in combo damage scaling
- **AI opponent generation:** Generate combos via proptest strategies; train neural nets to predict player strategies

**Milestones:**
- **2027 Q2:** Combo enumeration engine ships; proves ShooterGame duel system has zero infinite-loop bugs
- **2028 Q1:** AI opponent training pipeline deployed; bot difficulty tunable via invariant relaxation
- **2029 Q1:** Publish "Combinatorial Mechanics in AAA Fighting Games" white paper; GDC talk

**Ownership:** nexus-engine team + academic partnerships (UC Berkeley, CMU)

### 5.2 Permadeath Progression Systems (ib4-progression)

**Research Question:** How can permadeath progression feel rewarding rather than punishing?

**Approach:**
- **Spiral progression model:** Each permadeath run unlocks permanent upgrades (rune shards, skill trees) for next run; players feel constant forward momentum
- **Asymptotic difficulty:** Boss difficulty scales sublinearly with permadeath runs; players approach win condition exponentially (first run: 1% winrate; run 50: 50% winrate)
- **Narrative emergence:** ib4-ai generates procedural boss dialogue that references player's previous runs; creates sense of continuity

**Milestones:**
- **2027 Q3:** ib4-progression v1 ships with InfinityBlade4 alpha; player retention metrics > 60% at day 30
- **2028 Q2:** Publish permadeath design patterns in GDC, IGDA
- **2029 Q2:** Permadeath progression becomes industry best practice; adopted by 10+ AAA titles

**Ownership:** ib4-progression team + game design research groups

### 5.3 Semantic Web for Game Design (unify-rs)

**Research Question:** Can RDF + SPARQL become the standard for game balance sheets, progression systems, and economy design?

**Approach:**
- **Balance sheet as RDF graph:** All game numbers (health points, damage, cooldowns, loot drop rates) stored as RDF; SPARQL queries auto-generate balance documentation
- **Invariant querying:** "Find all weapons where ATK > 100 AND cooldown < 2s" via SPARQL; auto-flag balance outliers
- **Cross-game analytics:** Query across all six AAA titles ("aggregate winrate by character across ShooterGame + SurvivalGame") via federated SPARQL endpoints
- **Designer-friendly UI:** Unify-lsp plugin for game designers (non-programmers) to write SPARQL, see results in real-time

**Milestones:**
- **2027 Q4:** SurvivalGame balance sheet as RDF; prove 20% faster balance iteration vs. spreadsheets
- **2028 Q3:** Cross-game SPARQL federation working; enable "cosmetic portability" (cosmetics from one game worn in another)
- **2029 Q4:** Semantic game design becomes industry standard; published in TOCHI, ACM Games Transactions

**Ownership:** unify-rs team + game design + research

### 5.4 Anti-LLM Cheat Detection

**Research Question:** As LLMs become ubiquitous, how can we detect and prevent AI-assisted cheating in competitive games?

**Approach:**
- **Behavioral profiling:** Record keystroke timing, aim smoothness, decision latency; detect anomalous LLM-assisted patterns
- **Input entropy analysis:** LLMs produce lower-entropy inputs (predictable decision trees); humans produce higher-entropy patterns
- **Adversarial testing:** Train LLMs to cheat; use adversarial examples to understand detection surface
- **Open research:** Publish findings; enable community to contribute detectors

**Milestones:**
- **2026 Q4:** Anti-LLM-cheat scanner v1 deployed in ShooterGame; 99.5% false-positive threshold
- **2027 Q3:** Publish "Anti-AI-Cheating in Multiplayer Games" research; open-source detector library
- **2028 Q2:** Adopt as industry standard; integrated into Unreal Engine 5.2+

**Ownership:** Security team + game integrity

### 5.5 Parser Robustness & DFLSS Manufacturing

**Research Question:** How can we automate parsing, validation, and error recovery in game asset pipelines?

**Approach:**
- **DFLSS (Define, Find, Look, See, Standardize):** Formalize asset parsing as a manufacturing process; auto-generate error recovery
- **Parser fuzzing:** Fuzz all parsing (T3D, FBX, GLTF, PMX) with libfuzzer + quickcheck; find edge cases automatically
- **Standardized error messages:** Every parse error includes a SPARQL-friendly error graph (error location, recovery suggestion, example fix)

**Milestones:**
- **2027 Q1:** DFLSS manufacturing framework published; asset-pipeline robustness improves 50%
- **2027 Q4:** Zero parser crashes in asset-pipeline; all errors recoverable
- **2029 Q1:** DFLSS becomes standard in Unreal asset tools; adopted by other studios

**Ownership:** Asset-pipeline + blueprint-rs teams

---

## 6. Infrastructure & DevOps Maturity

### 6.1 CI/CD Evolution

**Current (2026):** Basic GitHub Actions: pwa-staff tests + chicago-tdd-tools build.

**2030 Vision:**

| Component | 2026 | 2030 |
|-----------|------|------|
| **Workspace Coverage** | 20% | 100% (all 7 Rust workspaces + PWA + 6 UE4 projects) |
| **Test Speed** | 15min | <2min (sharded across 50 runners) |
| **Artifact Caching** | No | Hermetic builds; incremental compilation via sccache |
| **Cross-Platform Builds** | Manual | Automated (Win64, Linux, macOS, Android, iOS, Web) |
| **Deployment Pipeline** | Ad-hoc | Continuous deployment to staging (every PR); production gates on benchmarks |
| **Security Scanning** | No | Cargo-deny + SBOM generation; malware detection via CodeQL |

**Key Milestones:**
- **2027 Q2:** All Rust workspaces in CI; test speed < 5min
- **2027 Q4:** Cross-platform UE4 builds in CI (Web, Win64, Android)
- **2028 Q2:** Continuous deployment to staging; automated performance regression detection
- **2029 Q1:** Zero-downtime production deployments; A/B testing framework
- **2030 Q1:** CI/CD maturity at scale; handles 10M DAU with <1% deployment failure rate

### 6.2 Observability & Monitoring

**By 2030, Rocket Craft will have enterprise-grade observability:**

- **Distributed tracing:** OpenTelemetry across all six AAA titles; trace a player request from client JS to server Rust to DB
- **Real-time dashboards:** Grafana dashboards for:
  - Player engagement (DAU, MAU, session length, churn)
  - Game balance metrics (character winrates, economy inflation, daily earnings)
  - Infrastructure health (latency P99, error rates, memory usage)
  - Security events (cheat detections, suspicious behavior, exploit attempts)
- **Alerting & escalation:** PagerDuty integration; <5min MTTR for critical issues
- **Player analytics:** Privacy-respecting analytics (differential privacy, aggregation) to inform design decisions

### 6.3 Global Deployment & Edge Computing

**2030 Deployment Topology:**

```
┌─────────────────────────────────────────────┐
│ Content Delivery Network (CDN)              │
│ (Cloudflare + regional caches)              │
└────────────────┬────────────────────────────┘
                 │ WebGL2 / HTML5 games
┌────────────────┴────────────────────────────┐
│ Regional Edge Servers (Cloudflare Workers)  │
│ - Asset streaming                           │
│ - Low-latency proxying                      │
│ - Client-side replay buffer (E2E Orchestr.) │
└────────────────┬────────────────────────────┘
                 │ WebSocket → Game Server
┌────────────────┴────────────────────────────┐
│ Global Game Servers (Kubernetes on AWS/GCP) │
│ - 6 regional clusters (NA, EU, APAC, etc.) │
│ - Stateless game servers (nexus-engine)    │
│ - Postgres (Supabase) as SSOT              │
└────────────────┬────────────────────────────┘
                 │ 
┌────────────────┴────────────────────────────┐
│ Analytical Data Warehouse (BigQuery)        │
│ - Cross-game player behavior                │
│ - Balance sheets as RDF (unify-rdf)        │
│ - SPARQL federation for game designers     │
└─────────────────────────────────────────────┘
```

**Reliability Targets:**
- Game server availability: 99.99% SLA
- WebGL2 game latency: <100ms P99 (client to server)
- Player progression durability: Zero loss of earned cosmetics/progression (Postgres HA + WAL archival)

---

## 7. Community & Governance

### 7.1 Contributor Paths & RFC Process

**By 2030, Rocket Craft will have formalized contribution workflows:**

#### 7.1.1 Contributor Levels

| Level | Requirements | Responsibilities | Compensation |
|-------|---|---|---|
| **Contributor** | 1 merged PR | Submit features, fixes | Bounties ($50–$500 per PR) |
| **Maintainer** | 50+ merged PRs + 1 year tenure | Code review, issue triage | $5K–$20K/month + equity options |
| **Architect** | 200+ PRs + RFC council approval | Design major systems, RFC lead | $50K–$150K/year + equity |
| **Steward** | Community vote + board approval | Governance, strategy | Board seat + share pool |

#### 7.1.2 RFC Process

All major features require an **RFC (Request for Comments)** proposal:

1. **Proposer** writes RFC in markdown: problem statement, design, trade-offs, implementation plan
2. **RFC Council** (20 elected contributors) reviews for 1–2 weeks; public discussion encouraged
3. **Acceptance criteria:** 60% council approval + no critical unresolved objections
4. **Implementation:** Merged PRs reference RFC; RFC updated with lessons learned
5. **Retrospective:** Post-launch, RFC author writes retrospective on decisions

**Example RFCs:**
- "Unify-MCP Phase 2: Game-Level Resource Exposure" (2027)
- "Blueprint-rs Binary Reverse-Engineering Pipeline" (2027)
- "Cross-Game Cosmetics Portability" (2028)
- "Permadeath Progression Open Standard" (2029)

### 7.2 Plugin & Modding Ecosystem

**By 2030, third-party plugins are first-class:**

#### 7.2.1 Plugin Categories

| Category | Technology | Revenue Model | Example |
|----------|-----------|---|---|
| **Nexus-MCP Plugins** | Rust | 80% creator share | Custom combo systems, AI directors |
| **Blueprint Macros** | Rust procedural macros | 70% creator share | Level generation macros, behavior trees |
| **Unify-LSP Extensions** | TypeScript + LSP | 75% creator share | Custom SPARQL query builders, asset inspectors |
| **Asset Library** | FBX, GLTF, PMX | 60% creator share | Character meshes, environments, VFX |
| **Modding Kits** | Lua/TypeScript scripting | Per-creator licensing | Custom game modes, progression systems |

#### 7.2.2 Plugin Marketplace

A web-based marketplace (powered by pwa-staff PWA):

- **Discovery:** Filter by category, rating, downloads, price
- **Installation:** One-click plugin install to local rocket workspace
- **Revenue tracking:** Real-time dashboard for creators (downloads, revenue, reviews)
- **Quality gates:** Malware scanning, performance benchmarking, compatibility testing
- **Governance:** Community votes on "Plugin of the Month"; highlighted creators

**2030 Targets:**
- 5,000 plugins in marketplace
- $4M annual marketplace revenue
- Top creator earning $100K+/year

### 7.3 Competitive & Esports Ecosystem

**By 2030, Rocket Craft becomes an esports platform:**

#### 7.3.1 ShooterGame Esports

- **Franchise partnerships:** 12–16 regional franchises (similar to Valorant Champions)
- **Prize pool:** $10M+ annually (worlds championship, regional opens, grassroots tournaments)
- **Streaming:** Integrated streaming features (1-click spectating, custom camera angles, player stats overlays)
- **Skill rating:** Open ELO system; tournament seeding based on SR + recent performance

#### 7.3.2 Content Creator Partnerships

- **Streamer program:** Revenue share on cosmetics designed by/for streamers
- **Content fund:** $100K/month bounty for community-created guides, tutorials, clips
- **Sponsorship opportunities:** Brands can sponsor seasonal tournaments; transparent, creator-friendly terms

---

## 8. Success Metrics & KPIs

### 8.1 Technical Excellence

| Metric | 2026 Baseline | 2030 Target | Measurement |
|--------|---|---|---|
| **Test Coverage** | 40% | 95%+ | codecov.io across all workspaces |
| **Build Time** | 15min | <2min | CI/CD runtime (sharded) |
| **Security Incidents** | 3/year | <1/year | CVE disclosures, incident reports |
| **Parser Robustness** | 80% | 99.9% | fuzz-testing coverage, zero panics |
| **Type Safety** | Limited | Comprehensive | Typestate coverage, clippy warnings |

### 8.2 Player & Community Growth

| Metric | 2026 Baseline | 2030 Target | Tracking |
|--------|---|---|---|
| **Total MAU** | 1M | 10M+ | Analytics dashboard |
| **DAU/MAU Ratio** | 35% | 50%+ | Daily active cohort |
| **Churn Rate** | 10%/month | <5%/month | Retention curves |
| **Cosmetic ARPU** | $2.50 | $8–$12 | Revenue / MAU |
| **Community Contributors** | 500 | 50,000+ | GitHub contributors, bounty claimants |

### 8.3 Creator & Ecosystem Health

| Metric | 2026 Baseline | 2030 Target | Tracking |
|--------|---|---|---|
| **Creator Revenue** | $0 | $12M+/year | Payout dashboards |
| **Marketplace Plugins** | 0 | 5,000+ | Plugin registry |
| **Content Creators (monthly)** | 100 | 10,000+ | UGC uploads, cosmetics submissions |
| **Third-Party Studios** | 0 | 50+ | Licensed engine users |

### 8.4 Infrastructure Reliability

| Metric | 2026 Baseline | 2030 Target | SLA |
|--------|---|---|---|
| **Game Server Uptime** | 99.5% | 99.99% | 4-nines SLA |
| **P99 Latency** | 150ms | <100ms | Client-to-server (network + game logic) |
| **Login Success Rate** | 98% | 99.9%+ | Across peak load |
| **Data Loss Incidents** | 0 | 0 | Postgres HA + archival |

### 8.5 Research Impact

| Metric | 2026 Baseline | 2030 Target | Venue |
|--------|---|---|---|
| **Peer-Reviewed Publications** | 0 | 5+ | TOCHI, TOGS, Games Transactions |
| **GDC Talks** | 1 | 5+ | Speaking slots at GDC/Unreal Fest |
| **Industry Adoption** | N/A | 20+ studios | Third-party implementations of typestate, permadeath, RDF systems |
| **Open-Source Citations** | 100 | 1,000+ | GitHub stars, academic papers |

---

## 9. Milestones & Roadmap (2026–2030)

### 2026 (Current)

**Q3 2026 (Now):**
- Combinatorial Testing Engine (`combinatorial-engine`) integrated
- AutoML DX framework (`unify-automl`) deployed
- WebGL2 E2E Orchestrator deployed
- Anti-LLM-cheat scanner v1 live in ShooterGame
- unify-mcp Phase 1 shipped

**Q4 2026:**
- ShooterGame reaches 500K DAU
- SurvivalGame closed beta → open beta
- Brm design document finalized; production greenlight
- First community law submitted to knhk Marketplace (pending)

### 2027 (Acceleration)

**Q1–Q2 2027:**
- Law Marketplace ships; 50+ community laws certified
- Combo enumeration engine live; ShooterGame duel system proven bug-free
- ib4-progression v1 ships with InfinityBlade4 alpha
- Blueprint-rs binary reverse-engineering pipeline open-sourced

**Q3–Q4 2027:**
- Workspace SemVer deployed; all crates tagged v1.0.0-stable
- MCP Phase 2 ships; nexus-mcp + economy-mcp integrated into 3+ AAA titles
- SurvivalGame balance sheet converted to RDF; 20% faster iteration proven
- ShooterGame reaches 3M DAU; franchising partnerships announced

### 2028 (Maturation)

**Q1–Q2 2028:**
- Continuous deployment to staging; automated regression detection live
- AI opponent training pipeline deployed; bot difficulty tunable
- InfinityBlade4 launches; 500K DAU at release
- Creator revenue share program paying $1M+/month

**Q3–Q4 2028:**
- Cross-game SPARQL federation working; cosmetic portability MVP shipped
- LSP Layer 2 in beta; VSCode extension >50K downloads
- Brm launches; first game fully integrated with unify-mcp + permadeath progression
- Esports franchising announced for ShooterGame; 12 regional franchises signed

### 2029 (Scale)

**Q1–Q2 2029:**
- Permadeath progression adopted as industry best practice; GDC publication
- Cross-game total MAU reaches 5M; ecosystem revenue $30M+/year
- RealisticRendering achieves WebGL2 visual parity with desktop (light baking, HBAO+)
- Fifth AAA title in concept phase (working title: TBD)

**Q3–Q4 2029:**
- Semantic game design becomes industry standard; published in TOCHI
- Plugin marketplace hits 2,000+ plugins; $2M+ annual marketplace revenue
- Contributor base reaches 25,000 active; maintainer team expanded to 20+
- 10 third-party studios licensed rocket-sdk + core crates

### 2030 (Vision Realization)

**Q1–Q2 2030:**
- All six AAA titles live and healthy; 10M MAU, $100M+ ecosystem revenue
- CI/CD maturity at scale; handles peak load with <1% failure rate
- FullSpectrum template used by 10K+ creators; 100+ creator-made games in ecosystem
- Anti-LLM-cheat detection adopted as industry standard; integrated into Unreal 5.2+

**Q3–Q4 2030:**
- Rocket Craft Decade: Retrospective publication; 5+ peer-reviewed papers published
- Esports championship: ShooterGame Worlds with $5M+ prize pool
- Second-order ecosystem: Cross-media licensing (anime, manga, web3 gaming); $10M+ licensing revenue
- Governance transition: Stewards elected; public benefit corporation formalized; contributor-owned future path charted

---

## 10. Risks & Mitigation Strategies

### 10.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| **UE4 EOL** (Epic shifts resources to UE5) | Medium | Critical | Plan UE5 migration in 2028–2029; maintain 4.24 LTS fork in perpetuity |
| **Type-safe Rust adoption bottleneck** | Low | High | Invest in onboarding, tutorials, mentorship program |
| **Semantic web (RDF) complexity** | Medium | High | Simplify UX via LSP + automated SPARQL generation; make it optional for newcomers |
| **Parser fuzzing edge cases** | Low | Medium | Continuous fuzzing; 24/7 fuzzer farm |

### 10.2 Market Risks

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| **Player monetization resistance** | Medium | High | Cosmetics-only model; no P2W mechanics; transparent pricing |
| **Competitor IP pressure** (e.g., Bandai Namco on Gundam terms) | Medium | High | Diversify away from single IP; "generic" Gundam-inspired mechanics |
| **Creator revenue sustainability** | Medium | Medium | Revenue share guarantees; bounty pool funded from licensing |

### 10.3 Governance Risks

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| **Contributor burnout** | High | High | Paid maintainer program; mandatory time-off for core team; rotate RFC council |
| **Forking / fragmentation** | Medium | High | Transparent governance; quick RFC decisions; creator revenue sharing |
| **IP disputes** | Low | Critical | Legal review of all submissions; contributor license agreement; clear ownership model |

### 10.4 Execution Risks

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| **Scope creep on game releases** | High | High | Strict milestone gates; ship MVP then iterate; "done is better than perfect" |
| **Infrastructure scaling bottlenecks** | Medium | High | Early stress-testing; capacity planning 6 months ahead; auto-scaling infrastructure |
| **Talent acquisition** | High | High | Competitive salaries; remote-first; equity options for top contributors |

---

## 11. Conclusion: A Collaborative AAA Future

By 2030, Rocket Craft will have demonstrated that open-source, type-safe, semantically-aware game development is not only viable but **essential** for the next generation of AAA games. We will have:

1. **Shipped six AAA titles** on a unified, contributor-driven infrastructure
2. **Established type-safe Rust + typestate patterns as the gold standard** for competition-grade game mechanics
3. **Proven that semantic web (RDF/SPARQL/MCP) improves game design velocity** by 2–3x
4. **Created a sustainable, contributor-friendly monetization model** where creators earn $12M+/year
5. **Fostered a global developer community** of 100,000+ contributors, with 50,000+ monthly modders
6. **Set the bar for infrastructure reliability** at 10M concurrent players with <100ms latency globally

Most importantly, **we will have restored agency and ownership to game creators and players.** In an industry increasingly consolidated around walled-garden platforms, Rocket Craft offers an alternative: a public, auditable, contributor-owned ecosystem where the rules are written by the community and enforced by types, not lawyers.

This is our North Star for 2030. The foundation is laid. The work is clear. The future is collaborative.

---

## Appendix A: Key Workspace Dependencies

```
rocket-sdk/knhk (Law Enforcement)
  ↓
rock-sdk/unrdf (RDF Triple Store)
  ↓
unify-rs (Semantic Web Ecosystem, 18 crates)
  ├─ unify-rdf (SPARQL queries)
  ├─ unify-mcp (Game Server tools)
  ├─ unify-lsp (IDE Integration)
  ├─ unify-codegen (Code Generation)
  └─ unify-automl (Auto-discovery & balance optimizer)
  
chicago-tdd-tools (BDD framework & combinatorial-engine)
  
nexus-engine (Game Logic, 10 crates)
  ├─ nexus-types (Unit types, IDs, typestate markers)
  ├─ nexus-combat (CombatMachine<S>, Combos)
  ├─ nexus-net (Connection<S>, Matchmaking)
  ├─ nexus-economy (Shop, Currency)
  └─ nexus-tests (Property-based testing)

blueprint-rs (Blueprint AST/T3D, 4 crates)
  ├─ blueprint-core (AST, types, serializers)
  ├─ blueprint-macros (Proc-macro generation)
  └─ blueprint-cli (REPL, T3D debugging)

ib4-mud (Infinity Blade 4 Backend, 6 crates)
  ├─ ib4-progression (Permadeath systems)
  ├─ ib4-combat (Action mechanics)
  ├─ ib4-ai (Procedural boss generation)
  └─ ib4-mud (MUD server)

asset-pipeline (3D Model Pipeline, 2 crates)
  ├─ pipeline-core (FBX/GLTF conversion)
  └─ pipeline-cli (Watch mode, batch processing)

pwa-staff (TypeScript PWA)
  ├─ auth.ts (Supabase auth)
  ├─ leaderboard.ts (Player rankings)
  ├─ admin.ts (Operator tools)
  └─ worker.ts (Service worker)
```

---

## Appendix B: Additional References

- **CLAUDE.md:** Project configuration guide; environment setup
- **project-manifest.json:** Authoritative list of all UE4 projects and their targets
- **RFC Process:** See `/docs/rfc-process.md` (to be created)
- **Permadeath Progression:** See `/docs/permadeath-design-patterns.md` (to be created)
- **Semantic Game Design:** See `/docs/sparql-for-game-designers.md` (to be created)

---

**Document prepared by:** Rocket Craft Vision 2030 Task Force  
**Last updated:** June 18, 2026  
**Next review:** December 2026 (mid-year progress assessment)
