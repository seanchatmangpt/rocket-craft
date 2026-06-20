# Phase 6 Kickoff — Marketplace Foundation

**Date:** 2026-06-18  
**Status:** Ready to Begin Development  
**Timeline:** 2027 Q1–Q2 (12 weeks, 240 hours)  
**Branch:** `claude/upbeat-euler-vp5x9f`  

---

## Executive Summary

Phase 6 launches the **N-Dimensional Rocket Craft Marketplace Ecosystem**, a $100M+ revenue platform connecting creators with players across 7 interconnected marketplace dimensions. This phase establishes the foundational payment, RDF, and API infrastructure required for all subsequent phases.

**Phase 6 Goals:**
- ✅ Stripe integration + webhook handler
- ✅ Creator payout system (double-entry ledger, monthly settlements)
- ✅ RDF linking (games, cosmetics, creators)
- ✅ Federated SPARQL endpoints (5 named graphs)
- ✅ React marketplace storefront + checkout
- ✅ First 100 cosmetics published
- ✅ First 50 creators onboarded
- ✅ First $10K revenue processed

---

## Part 1: Strategic Foundation (Completed)

All research and design work for Phase 6 has been completed. Reference these documents for strategic context:

| Document | Role |
|----------|------|
| **VISION_2030_N_DIMENSIONAL_MARKETPLACE.md** | 7-dimension marketplace design; economics model ($100M platform, 64% to creators) |
| **GGEN_MARKETPLACE_INTEGRATION_STRATEGY.md** | 5-stage ggen integration pipeline; 8 proof gates; Tera template architecture |
| **ONTOLOGY_SYNTHESIS_AND_DESIGN.md** | 6 custom OWL ontologies (rocket-craft-{core,states,types,manifest,quality,architecture}.owl) |
| **PHASE_6_IMPLEMENTATION_PLAN.md** | Detailed 12-week breakdown into 5 workstreams (Payment, Payouts, RDF, SPARQL, UI) |
| **SEMANTIC_FOUNDATION_INDEX.md** | Complete project metrics and navigation (72K Rust LOC, 44 crates, 911+ tests) |

---

## Part 2: Technology Stack

### Backend
- **Language:** Rust (all workspaces)
- **Core Crate:** `unify-rocket` (extends existing context, compliance, receipt modules)
- **Payment:** Stripe API + Stripe Connect
- **Database:** RDF triple store (Apache Jena Fuseki or native)
- **Ledger:** Extended `nexus-economy` double-entry ledger (USD support)
- **Web Server:** Actix-web or Tokio HTTP
- **Async Runtime:** Tokio (already used throughout)

### Frontend
- **Framework:** React 18 + TypeScript
- **Bundler:** Vite
- **State:** TanStack Query (React Query)
- **Styling:** Tailwind CSS
- **Payment UI:** Stripe.js
- **Hosting:** Vercel or Netlify

### Semantic Web
- **RDF Store:** Apache Jena Fuseki (Docker)
- **Query Language:** SPARQL 1.1
- **Validation:** SHACL shapes
- **Provenance:** PROV-O (W3C standard)

---

## Part 3: Codebase Readiness Check

### Existing Infrastructure (Ready to Use)

1. **unify-rocket** (`unify-rs/unify-rocket/src/lib.rs`)
   - ✅ WorkspaceContext, WorkspaceManifest (manifest parsing)
   - ✅ ProjectValidator, ProjectLaw (compliance framework)
   - ✅ RocketReceipt, RocketReceiptChain (audit trail)
   - ✅ RocketClassify commands (noun-verb protocol)
   - ✅ RocketMakefileCodegen, RocketDockerfileCodegen
   - ✅ LeaderboardStore (can be adapted for creator rankings)

2. **nexus-economy** (`nexus-engine/crates/nexus-economy/`)
   - ✅ Double-entry Ledger (must extend for USD)
   - ✅ Marketplace (item listings)
   - ✅ Auction system
   - ✅ Shop (gacha mechanics)

3. **unify-rdf** (`unify-rs/unify-rdf/src/`)
   - ✅ RDF triple store
   - ✅ SPARQL query engine
   - ✅ SHACL validation shapes
   - ✅ ProjectManifest typestate (Pending → Ingested → Validated)

4. **unify-mcp** (`unify-rs/unify-mcp/src/`)
   - ✅ MCP server framework
   - ✅ Tool & resource registries
   - ✅ JSON-RPC protocol
   - ✅ Already exposes rocket tools

5. **pwa-staff** (`pwa-staff/`)
   - ✅ TypeScript PWA foundation
   - ✅ Supabase auth integration
   - ✅ Service worker
   - ✅ Can be extended for marketplace UI

### New Components (Must Create)

| Component | Location | Estimated Size | Priority |
|-----------|----------|-----------------|----------|
| **Stripe Client** | `unify-rocket/src/payment/` | 300 LOC | P0 |
| **Fiat Ledger** | `nexus-economy/src/ledger_usd.rs` | 150 LOC | P0 |
| **Payment Webhook** | `unify-rocket/src/payment/webhook.rs` | 200 LOC | P0 |
| **Creator Accounts** | `unify-rocket/src/creator/` | 250 LOC | P0 |
| **Monthly Settlement** | `unify-rocket/src/settlement.rs` | 300 LOC | P0 |
| **Game Spec RDF Schema** | `unify-rdf/src/schemas/game.ttl` | 150 LOC | P0 |
| **Cosmetic RDF Schema** | `unify-rdf/src/schemas/cosmetic.ttl` | 150 LOC | P0 |
| **Creator RDF Schema** | `unify-rdf/src/schemas/creator.ttl` | 100 LOC | P0 |
| **SPARQL Endpoint** | `unify-rocket/src/sparql.rs` | 200 LOC | P0 |
| **Marketplace React UI** | `pwa-staff/src/marketplace/` | 2,000 LOC | P1 |

**Total New Code (P0):** ~1,800 LOC (Week 1–9)  
**Total Estimated Effort:** 240 hours (12 weeks)

---

## Part 4: Detailed Next Steps (Immediate)

### Week 1: Payment Infrastructure Foundation

#### Day 1–2: Stripe Client Wrapper
**File:** `unify-rocket/src/payment/client.rs`

```rust
pub struct StripeClient {
    api_key: String,
    client: reqwest::Client,
}

impl StripeClient {
    pub async fn create_checkout_session(
        &self,
        customer_email: &str,
        item_name: &str,
        amount_cents: u32,
    ) -> Result<CheckoutSession> { ... }
    
    pub async fn retrieve_payment_intent(
        &self,
        intent_id: &str,
    ) -> Result<PaymentIntent> { ... }
    
    pub async fn list_charges(
        &self,
        customer_id: &str,
    ) -> Result<Vec<Charge>> { ... }
}
```

**Acceptance Criteria:**
- [ ] Unit tests with mocked Stripe API responses
- [ ] Error types map Stripe errors to domain errors
- [ ] Stripe test API keys work
- [ ] All 3 methods implemented and tested

#### Day 3–4: Fiat Ledger Extension
**File:** `nexus-economy/src/ledger_usd.rs`

```rust
pub struct FiatLedger {
    entries: Vec<LedgerEntry<Cents>>,
    total_debit: Cents,
    total_credit: Cents,
}

impl FiatLedger {
    pub fn record_debit(
        &mut self,
        account: &str,
        amount: Cents,
        description: &str,
    ) -> Result<()> { ... }
    
    pub fn record_credit(
        &mut self,
        account: &str,
        amount: Cents,
        description: &str,
    ) -> Result<()> { ... }
}
```

**Acceptance Criteria:**
- [ ] All operations maintain `debit == credit` invariant
- [ ] Property-based test verifies invariant (proptest)
- [ ] Stripe charge IDs can be stored with entries
- [ ] Query by date range works

#### Day 5: Webhook Handler Scaffolding
**File:** `unify-rocket/src/payment/webhook.rs`

```rust
pub struct WebhookHandler {
    signing_secret: String,
}

impl WebhookHandler {
    pub fn verify_signature(&self, body: &[u8], signature: &str) -> bool { ... }
    
    pub async fn handle_charge_succeeded(
        &self,
        event: Event,
    ) -> Result<()> { ... }
}
```

**Acceptance Criteria:**
- [ ] HMAC-SHA256 signature verification works
- [ ] Signature verification rejects spoofed events
- [ ] Idempotency: replay doesn't create duplicate ledger entries

### Week 2: RDF Schema Design

#### Day 1–2: Game Spec Schema
**File:** `unify-rdf/src/schemas/game.ttl`

```turtle
@prefix rc: <http://rocketcraft.dev/ontology/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

rc:GameListing a owl:Class ;
    rdfs:subClassOf rc:MarketplaceItem ;
    rc:title rdf:langString ;
    rc:gameSpecUri xsd:anyURI ;
    rc:creator dc:creator ;
    rc:targetPlatforms rdf:Seq ;
    rc:genre xsd:string ;
    rc:maxPlayers xsd:integer .

rc:ReleaseStatus a owl:Class ;
    owl:oneOf ( rc:Alpha rc:Beta rc:Released rc:Retired ) .
```

**Acceptance Criteria:**
- [ ] Schema validates in Turtle parser
- [ ] All required properties defined
- [ ] SHACL shape validates instances
- [ ] JSON-LD context includes game vocab

#### Day 3–4: Cosmetic Schema
**File:** `unify-rdf/src/schemas/cosmetic.ttl`

**Acceptance Criteria:**
- [ ] Cosmetics link to games via `rc:applicableGames`
- [ ] Rarity tiers defined
- [ ] Price stored as xsd:decimal
- [ ] SHACL validates cosmetic instances

#### Day 5: Creator Profile Schema
**File:** `unify-rdf/src/schemas/creator.ttl`

**Acceptance Criteria:**
- [ ] Creator IDs unique across platform
- [ ] Earnings queryable via SPARQL aggregation
- [ ] Created works linked

### Week 3–4: Payment & Settlement Integration

#### Creator Account Onboarding
- [ ] Stripe Connect account creation flow
- [ ] KYC verification status tracking
- [ ] Bank account validation

#### Monthly Settlement Job
- [ ] Query RDF for creator earnings
- [ ] Calculate revenue splits (creator/platform/community)
- [ ] Create Stripe transfers
- [ ] Record PROV-O provenance chains
- [ ] Idempotency handling

### Week 5–7: SPARQL Endpoints

#### Fuseki Deployment
- [ ] Docker configuration
- [ ] Named graph setup (5 graphs)
- [ ] Authentication (API keys)
- [ ] Read replica for queries

#### Query Templates
1. "List all games by genre"
2. "Show cosmetics for Game X"
3. "Creator earnings summary"
4. "Cross-game cosmetic opportunities"
5. "Top creators by revenue"

### Week 8–12: Marketplace UI

#### Storefront Pages
1. **Browse Games** — Grid, filters, search
2. **Browse Cosmetics** — Grid, filters, search
3. **Game Detail** — Full spec, screenshots, buy button
4. **Cosmetic Detail** — 3D model viewer, applicable games, buy button
5. **Checkout** — Stripe Elements, payment flow
6. **Success Page** — Confirmation, next steps
7. **Creator Dashboard** — Earnings, payouts, published items

---

## Part 5: Development Rules & Practices

### Code Quality
- **100% test coverage** for payment + ledger + settlement logic
- **No TODO/FIXME** comments (anti-llm-cheat-lsp validates this)
- **Property-based testing** for financial logic (proptest)
- **SPARQL query performance** <500ms (p95)

### Git Workflow
- Branch: `claude/upbeat-euler-vp5x9f`
- Commit per workstream completion (not per file)
- Commit messages: `feat(payment): stripe integration` or `fix(ledger): balance invariant`

### Documentation
- Update PHASE_6_IMPLEMENTATION_PLAN.md weekly with progress
- Publish example SPARQL queries to marketplace API docs
- Create creator onboarding guide (KYC, payout timeline)
- Deployment runbook (Docker, env vars, monitoring)

### Testing Strategy
- **Unit tests** in each crate (`tests/` directory)
- **Integration tests** in `unify-integration-tests/`
- **End-to-end tests:**
  - User buys cosmetic → payment processed → cosmetic applied
  - Monthly settlement → creator receives payout
  - SPARQL query returns correct cross-dimensional results

### Monitoring & Observability
- **Tracing:** All payment operations logged with `tracing::info!`, `tracing::error!`
- **Metrics:** Transaction count, error rate, settlement job duration
- **Audit Trail:** Every operation recorded in PROV-O RDF graph
- **Alerting:** Failed webhook, settlement errors, SPARQL query timeouts

---

## Part 6: Risk Register & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Stripe API rate limits during testing | Medium | Low | Use Stripe test environment; stagger load tests |
| RDF query performance at scale | High | High | Materialize common queries; implement Redis cache layer |
| Creator KYC delays blocking payouts | Medium | Medium | Implement tiered payouts; initial threshold higher until KYC complete |
| Payment webhook failures (network, parsing) | Medium | High | Implement exponential backoff + retry queue; DLQ for failed webhooks |
| React UI integration delays | Low | Medium | Use OpenAPI spec + mock API server during frontend dev |
| SPARQL endpoint DoS via complex queries | Medium | Medium | Implement query timeout (5s), rate limiting (1K queries/day per API key) |

---

## Part 7: Success Criteria (Phase 6 Complete)

### Functional
- [ ] Stripe checkout sessions created successfully (test + production API keys)
- [ ] All transactions recorded in ledger + linked to Stripe charge IDs
- [ ] Creator Stripe Connect accounts onboarded (at least 5 test accounts)
- [ ] Monthly settlement job runs without errors (simulated 1K+ USD payout)
- [ ] RDF store has 100+ game specs + 100+ cosmetics + creator profiles
- [ ] SPARQL queries respond <500ms (p95) for typical queries
- [ ] Marketplace UI loads and renders games/cosmetics correctly
- [ ] Checkout flow completes successfully with test card
- [ ] Success page displays order confirmation
- [ ] Creator dashboard shows earnings + payout history

### Quality
- [ ] Zero payment-related bugs in production
- [ ] 100% test coverage for payment + ledger + settlement
- [ ] All SPARQL queries documented with examples
- [ ] Zero unhandled errors (all errors mapped to domain types)
- [ ] Webhook signature verification prevents spoofed events

### Business
- [ ] First 100 cosmetics published to platform
- [ ] First 50 creators onboarded (account created, Stripe Connect linked)
- [ ] First $10K revenue processed through Stripe
- [ ] First monthly settlement completed (500+ USD distributed)
- [ ] 1,000+ monthly active users (MAU) on storefront
- [ ] <2% payment failure rate

---

## Part 8: Next Phase Preview (Phase 7)

Once Phase 6 completes (Week 12), Phase 7 launches:

**Phase 7: Dimension Scaling (Weeks 13–24)**
- Launch tools marketplace (MCP plugins, Tera templates, Rust crates)
- Launch ontology registry (reusable game specs)
- Implement cross-dimensional SPARQL discovery
- Build creator analytics dashboard
- Launch creator councils & governance

See VISION_2030_N_DIMENSIONAL_MARKETPLACE.md, Phase 7 section for details.

---

## Getting Started (First Action Items)

### Day 1
1. [ ] Read PHASE_6_IMPLEMENTATION_PLAN.md (30 min)
2. [ ] Read VISION_2030_N_DIMENSIONAL_MARKETPLACE.md Part 1 (economics overview) (30 min)
3. [ ] Create `unify-rocket/src/payment/` module directory
4. [ ] Stub `payment/client.rs` with `StripeClient` struct

### Day 2–3
1. [ ] Implement Stripe client wrapper methods
2. [ ] Add Stripe error types
3. [ ] Write unit tests (mock Stripe responses)
4. [ ] Add `stripe` crate to dependencies

### Day 4–5
1. [ ] Design fiat ledger extension
2. [ ] Property-based tests for ledger invariant
3. [ ] Integrate with existing ledger code

---

## Resources

### Documentation
- Stripe API docs: https://stripe.com/docs/api
- Stripe Webhooks: https://stripe.com/docs/webhooks
- Apache Jena Fuseki: https://jena.apache.org/documentation/fuseki2/
- SPARQL 1.1 Query: https://www.w3.org/TR/sparql11-query/
- SHACL: https://www.w3.org/TR/shacl/
- PROV-O: https://www.w3.org/TR/prov-o/

### Related Rocket Craft Docs
- PHASE_6_IMPLEMENTATION_PLAN.md
- VISION_2030_N_DIMENSIONAL_MARKETPLACE.md
- GGEN_MARKETPLACE_INTEGRATION_STRATEGY.md
- ONTOLOGY_SYNTHESIS_AND_DESIGN.md
- SEMANTIC_FOUNDATION_INDEX.md

---

**Phase 6 Status:** Ready to Begin  
**Branch:** `claude/upbeat-euler-vp5x9f`  
**Created:** 2026-06-18  
**Last Updated:** 2026-06-18
