# Phase 6: Marketplace Foundation — Implementation Plan

**Timeline:** 2027 Q1–Q2 (12 weeks)  
**Effort:** 240 hours  
**Status:** Ready for Development  

---

## Overview

Phase 6 establishes the foundational infrastructure for the N-dimensional Rocket Craft Marketplace Ecosystem. This phase focuses on:

1. **Payment Infrastructure** — Stripe integration for real-money transactions
2. **RDF Linking** — Connect game and cosmetics dimensions via semantic web
3. **Creator Payouts** — Monthly settlement system with PROV-O audit trails
4. **Marketplace UI** — React + TypeScript storefront for cosmetics and games
5. **Federated SPARQL** — Query endpoints enabling cross-dimensional discovery

---

## Part 1: Architecture Overview

### System Topology

```
┌─────────────────────────────────────────────────────────────┐
│                    Marketplace Services                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐  ┌─────────────────┐  ┌────────────┐ │
│  │  Payment Engine  │  │  Creator Payouts│  │ RDF Linker │ │
│  │  (Stripe)        │  │  (Monthly Settle)│  │ (SPARQL)   │ │
│  └────────┬─────────┘  └────────┬────────┘  └─────┬──────┘ │
│           │                     │                 │         │
│  ┌────────▼─────────────────────▼──────────────┬──▼──────┐ │
│  │         Marketplace Core Service             │         │ │
│  │  - Transaction ledger                        │         │ │
│  │  - Creator accounts                          │  unify- │ │
│  │  - Revenue attribution                       │  rocket │ │
│  │  - Fulfillment pipeline                      │         │ │
│  └──────────────────────────────────────────────┴─────────┘ │
│                        │                                      │
│  ┌─────────────────────▼──────────────────────────────────┐ │
│  │              RDF Triple Store (unify-rdf)             │ │
│  │  - Game specs                                         │ │
│  │  - Cosmetic definitions                               │ │
│  │  - Creator profiles                                   │ │
│  │  - Revenue distributions (PROV-O)                     │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Federated SPARQL Endpoints                    │  │
│  │  - /sparql/games: Query all games + specs           │  │
│  │  - /sparql/cosmetics: Cross-game cosmetics          │  │
│  │  - /sparql/creators: Creator profiles & earnings    │  │
│  │  - /sparql/revenue: PROV-O distribution chains      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │    Marketplace UI (pwa-staff + React UI)            │  │
│  │  - Game storefront                                   │  │
│  │  - Cosmetics catalog                                │  │
│  │  - Creator dashboard                                │  │
│  │  - Purchase & checkout                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

Game Backend (nexus-engine, UE4 projects)
        │
        ▼
    Ggen Pipeline
        │
        ▼
    Game Specs (RDF)
```

---

## Part 2: Detailed Breakdown by Workstream

### Workstream 1: Payment Infrastructure (Week 1–4)

**Owner:** Backend Lead

#### 1.1 Stripe Integration Core
- [ ] Create `unify-rocket/src/payment/` module with Stripe client wrapper
- [ ] Define `PaymentConfig` struct (API keys, webhook signing secret, currency)
- [ ] Implement `StripeClient` wrapper:
  - `create_checkout_session()` → SessionId
  - `retrieve_payment_intent()` → PaymentIntent
  - `list_customer_charges()` → Vec<Charge>
  - Error handling: `StripeError` enum with variants for network, validation, rate-limit
- [ ] Add `stripe` crate to `unify-rocket/Cargo.toml`
- [ ] Test with Stripe test keys in CI

**Success Criteria:**
- [ ] Checkout sessions created successfully
- [ ] Payment intents retrieved correctly
- [ ] Errors mapped to domain error types
- [ ] Stripe test API calls succeed

#### 1.2 Transaction Ledger (extended from nexus-economy)
- [ ] Extend `nexus-economy/src/ledger.rs` to support fiat currency (USD)
  - Current: in-game Gold only
  - New: typed ledger entries for USD transactions
- [ ] Define `FiatLedger` struct:
  ```rust
  pub struct FiatLedger {
      entries: Vec<LedgerEntry>,
      total_debit: Cents,  // in USD cents
      total_credit: Cents,
  }
  ```
- [ ] Implement invariant: `total_debit == total_credit` (double-entry accounting)
- [ ] Add transaction timestamps (Utc)
- [ ] Add reference to Stripe charge ID for reconciliation

**Success Criteria:**
- [ ] Ledger entries balance on every transaction
- [ ] Stripe charge IDs linked to ledger entries
- [ ] Can query ledger by date range
- [ ] Property-based tests verify double-entry invariant

#### 1.3 Webhook Handler
- [ ] Create `payment_webhook.rs` in `unify-rocket`
- [ ] Implement webhook signature verification (Stripe's HMAC-SHA256)
- [ ] Handle events:
  - `charge.succeeded` → record payment, trigger fulfillment
  - `charge.refunded` → record refund, reverse fulfillment
  - `account.updated` → update creator bank details
- [ ] Idempotency: webhook events must be safely replayable
- [ ] Logging: all webhook events logged to audit trail (PROV-O)

**Success Criteria:**
- [ ] Signature verification prevents spoofed webhooks
- [ ] Replayed events don't create duplicate ledger entries
- [ ] All event types handled
- [ ] Audit trail populated

---

### Workstream 2: Creator Payouts System (Week 3–6)

**Owner:** Payments/Finance Lead

#### 2.1 Creator Account & Bank Setup
- [ ] Define `CreatorAccount` struct:
  ```rust
  pub struct CreatorAccount {
      pub creator_id: CreatorId,
      pub name: String,
      pub email: String,
      pub stripe_connect_id: String,  // Stripe Connect account ID
      pub bank_account_id: String,    // Stripe bank account
      pub monthly_payout_threshold: Cents,  // minimum $50 to trigger payout
      pub created_at: DateTime<Utc>,
      pub kyc_verified: bool,  // KYC verification flag
  }
  ```
- [ ] Implement Stripe Connect onboarding flow:
  - `create_connect_account()` → onboarding link
  - `verify_connect_account()` → check if account complete
  - Store `stripe_connect_id` in database
- [ ] Add bank account & tax info validation

**Success Criteria:**
- [ ] Test Stripe Connect account creation
- [ ] Onboarding links generate correctly
- [ ] Creator data persists to database

#### 2.2 Revenue Distribution Engine
- [ ] Define revenue distribution tiers in `rocket-craft-quality.owl`:
  ```turtle
  rc:RevenueDistribution a owl:Class ;
      rdfs:subClassOf rc:RevenueEvent ;
      rc:platformCut 0.30 ;
      rc:creatorShare 0.50 ;
      rc:communityFund 0.20 .
  ```
- [ ] Implement distribution calculator:
  ```rust
  pub fn calculate_distribution(
      transaction_amount: Cents,
      creator_share_pct: f64,
      platform_cut_pct: f64,
      community_pct: f64,
  ) -> DistributionResult {
      let creator_cut = (transaction_amount as f64 * creator_share_pct) as Cents;
      let platform_cut = (transaction_amount as f64 * platform_cut_pct) as Cents;
      let community_cut = (transaction_amount as f64 * community_pct) as Cents;
      assert_eq!(creator_cut + platform_cut + community_cut, transaction_amount);
      DistributionResult { creator_cut, platform_cut, community_cut }
  }
  ```
- [ ] Link each transaction to PROV-O provenance chain:
  - `prov:Entity` = payment
  - `prov:Activity` = distribution
  - `prov:Agent` = creator, platform, community
  - `prov:wasGeneratedBy` = payment event
  - `prov:wasAttributedTo` = revenue recipients

**Success Criteria:**
- [ ] Distribution sums always equal transaction amount (property-based test)
- [ ] PROV-O provenance chains created and queryable
- [ ] Percentages configurable per dimension

#### 2.3 Monthly Settlement Job
- [ ] Create `settlement.rs` in `unify-rocket`:
  ```rust
  pub async fn run_monthly_settlement() -> Result<SettlementReport> {
      // 1. Query all creators with pending balance > threshold
      let creators = query_creators_for_settlement().await?;
      
      // 2. For each creator, query accumulated earnings from RDF
      for creator in creators {
          let earnings = query_creator_earnings_rdf(&creator.id).await?;
          
          // 3. Create Stripe transfer to Stripe Connect account
          let transfer = stripe_client.create_transfer(
              &creator.stripe_connect_id,
              earnings,
          ).await?;
          
          // 4. Record transfer in ledger + RDF provenance
          ledger.record_payout(&creator.id, earnings, &transfer.id)?;
          rdf_store.record_payout_prov_o(&creator.id, transfer.id, earnings)?;
      }
      
      Ok(SettlementReport { ... })
  }
  ```
- [ ] Implement idempotency: if settlement job runs twice, no duplicate payouts
- [ ] Error handling: partial failures (some creators paid, others failed) should be recoverable
- [ ] Logging: detailed audit trail for each payout
- [ ] Scheduling: runs on 1st of month via cron job in Docker

**Success Criteria:**
- [ ] Settlement job completes monthly
- [ ] All earnings distributed correctly
- [ ] Ledger and RDF in sync
- [ ] Audit trail complete
- [ ] Idempotent (safe to replay)

---

### Workstream 3: RDF Linking (Game & Cosmetics) (Week 2–5)

**Owner:** Semantic Web Lead

#### 3.1 Game Spec RDF Schema
- [ ] Extend `rocket-craft-core.owl` with game marketplace vocabulary:
  ```turtle
  rc:GameListing a owl:Class ;
      rdfs:subClassOf rc:MarketplaceItem ;
      rc:title rdf:langString ;
      rc:description rdf:langString ;
      rc:gameSpecUri xsd:anyURI ;  # URI to game spec in ggen output
      rc:creator dc:creator ;  # Link to creator
      rc:launchDate xsd:date ;
      rc:targetPlatforms rdf:Seq ;  # [Win64, HTML5, Android, iOS]
      rc:genre xsd:string ;
      rc:playersPerSession xsd:integer ;
      rc:maxPlayers xsd:integer ;
      rc:releaseStatus rc:ReleaseStatus .  # [Alpha, Beta, Released, Retired]
  ```
- [ ] Implement JSON-LD context for game listings
- [ ] Create Turtle generator from ggen game specs:
  - Input: game spec JSON (from ggen output)
  - Output: Turtle triples in RDF store

**Success Criteria:**
- [ ] Game specs converted to RDF
- [ ] Queryable via SPARQL

#### 3.2 Cosmetics Definition RDF Schema
- [ ] Extend `rocket-craft-core.owl` with cosmetics vocabulary:
  ```turtle
  rc:Cosmetic a owl:Class ;
      rdfs:subClassOf rc:MarketplaceItem ;
      rc:name rdf:langString ;
      rc:description rdf:langString ;
      rc:creator dc:creator ;
      rc:applicableGames rdf:Seq ;  # games this cosmetic works with
      rc:priceUsd xsd:decimal ;
      rc:asset3dUri xsd:anyURI ;  # link to 3D model in asset library
      rc:rarityTier rc:RarityTier ;  # [Common, Uncommon, Rare, Epic, Legendary]
      rc:releaseDate xsd:date ;
      rc:purchaseCount xsd:integer ;
      rc:lastModified xsd:dateTime .
  ```
- [ ] Link cosmetics to games via `rc:applicableGames`
- [ ] Calculate cosmetic popularity metrics (SPARQL aggregation)

**Success Criteria:**
- [ ] Cosmetics defined in RDF
- [ ] Linkage to games stored semantically
- [ ] Query: "Show all cosmetics for Game X" returns correct results

#### 3.3 Creator Attribution
- [ ] Extend `rocket-craft-core.owl` with creator vocabulary:
  ```turtle
  rc:Creator a owl:Class ;
      rc:creatorId xsd:string ;  # Unique creator identifier
      rc:displayName rdf:langString ;
      rc:profileUri xsd:anyURI ;  # link to creator dashboard
      rc:createdWorks rdf:Seq ;  # games, cosmetics, tools, etc.
      rc:totalEarnings xsd:decimal ;
      rc:joinDate xsd:date ;
      rc:status rc:CreatorStatus .  # [Active, Suspended, Retired]
  ```
- [ ] Use `dc:creator` property to link all marketplace items to creators
- [ ] Implement creator profile view via SPARQL

**Success Criteria:**
- [ ] Creators have RDF profiles
- [ ] All marketplace items attributed
- [ ] Creator earnings queryable

---

### Workstream 4: Federated SPARQL Endpoints (Week 4–7)

**Owner:** Semantic Web / DevOps Lead

#### 4.1 SPARQL Endpoint Setup
- [ ] Deploy Apache Jena Fuseki (or RDF-native solution) in Docker
  - Endpoint: `https://api.rocketcraft.dev/sparql`
  - Query endpoint: `/sparql/query` (GET/POST)
  - Update endpoint: `/sparql/update` (POST)
  - Graph Store HTTP Protocol: `/sparql/data`
- [ ] Configure named graphs:
  - `http://rocketcraft.dev/graphs/games` — all game specs
  - `http://rocketcraft.dev/graphs/cosmetics` — all cosmetics
  - `http://rocketcraft.dev/graphs/creators` — creator profiles
  - `http://rocketcraft.dev/graphs/revenue` — PROV-O provenance
  - `http://rocketcraft.dev/graphs/ontologies` — schema definitions
- [ ] Enable authentication: API key header `X-Rocket-Key`
- [ ] Set up read-only query replica (separate from write instance)

**Success Criteria:**
- [ ] SPARQL endpoint responds to queries
- [ ] Named graphs organized
- [ ] Authentication enforced
- [ ] Query performance acceptable (<500ms for typical queries)

#### 4.2 Query Templates & Documentation
- [ ] Publish example SPARQL queries in documentation:
  1. **List all games** → `SELECT ?game ?title WHERE { ?game rc:title ?title . }`
  2. **Games by genre** → filter by `rc:genre`
  3. **Cosmetics for game X** → `?cosmetic rc:applicableGames ?game .`
  4. **Creator earnings** → aggregate revenue via PROV-O
  5. **Cross-dimensional queries** → join games + cosmetics + tools
- [ ] Add query results caching (Redis) for popular queries
- [ ] Rate limiting: 1000 queries/day per API key

**Success Criteria:**
- [ ] Queries documented and tested
- [ ] Performance acceptable
- [ ] Rate limits enforced

#### 4.3 Real-time Updates
- [ ] Implement pub/sub for RDF updates:
  - When game published → emit RDF insertion
  - When cosmetic purchased → update purchase count in RDF
  - When creator payout settled → update PROV-O provenance chain
- [ ] Options:
  - **A:** Kafka topic `rdf-events` → Kafka Streams processor → Fuseki updates
  - **B:** Direct API calls from payment service → Fuseki update endpoint
  - **Recommendation:** Option B for MVP (simpler), Option A for scale

**Success Criteria:**
- [ ] RDF updates within 5 seconds of event
- [ ] No stale queries

---

### Workstream 5: Marketplace UI (Week 5–9)

**Owner:** Frontend Lead

#### 5.1 Store Frontend Architecture
- [ ] Tech stack:
  - **Framework:** React 18 (TypeScript)
  - **Bundler:** Vite
  - **State:** TanStack Query (React Query) for API state
  - **Styling:** Tailwind CSS
  - **Payment:** Stripe.js client library
  - **Hosted:** Vercel or Netlify
- [ ] Component structure:
  ```
  src/
  ├── pages/
  │   ├── Browse.tsx          # Game & cosmetics catalog
  │   ├── GameDetail.tsx      # Single game details + purchase
  │   ├── CosmeticDetail.tsx  # Single cosmetic + purchase
  │   ├── Checkout.tsx        # Stripe checkout modal
  │   └── Success.tsx         # Post-purchase confirmation
  ├── components/
  │   ├── GameCard.tsx
  │   ├── CosmeticCard.tsx
  │   ├── FilterBar.tsx
  │   ├── SearchBox.tsx
  │   └── CreatorProfile.tsx
  ├── api/
  │   ├── games.ts            # API calls: list games, get game detail
  │   ├── cosmetics.ts        # API calls: list cosmetics, filter
  │   ├── checkout.ts         # API calls: create checkout session
  │   └── sparql.ts           # SPARQL query helpers
  └── types/
      └── models.ts           # TypeScript types for games, cosmetics, orders
  ```
- [ ] Environment variables: `VITE_STRIPE_KEY`, `VITE_API_BASE_URL`, `VITE_SPARQL_ENDPOINT`

#### 5.2 Game Storefront Page
- [ ] Display:
  - Grid of game cards (title, thumbnail, rating, player count, genre tags)
  - Filters: Genre, Platform, Release Status, Player Count
  - Sort: Popularity, Release Date, Rating
  - Search box (searches game titles + descriptions)
- [ ] Game detail view:
  - Large banner image
  - Title, description, creator name
  - Screenshots carousel
  - "Play Now" button (links to game launcher)
  - Rating & reviews section (stub for future)
  - Price (Free or USD amount)
  - Target platforms (Win64, HTML5, Android, iOS icons)
- [ ] API integration:
  - Fetch games via `/api/games?genre=action&platform=html5&sort=popularity`
  - Query SPARQL endpoint for dynamic filtering

**Success Criteria:**
- [ ] Games display correctly
- [ ] Filters work
- [ ] Search functional
- [ ] Detail pages load game data

#### 5.3 Cosmetics Storefront Page
- [ ] Display:
  - Grid of cosmetic cards (name, preview image, applicable games, price, rarity)
  - Filters: Rarity, Applicable Game, Price Range
  - Sort: Newest, Popularity, Price Low-to-High
  - Search cosmetic names
- [ ] Cosmetic detail view:
  - Large 3D model viewer (or video preview)
  - Name, description, creator
  - Applicable games (with checkmarks showing which ones player owns)
  - Price in USD
  - "Buy Now" button
  - Creator profile link
- [ ] API integration:
  - Fetch cosmetics via `/api/cosmetics?game=gundam-nexus&rarity=epic`
  - SPARQL: "Show me all cosmetics for games I own"

**Success Criteria:**
- [ ] Cosmetics grid loads and displays
- [ ] Filtering works
- [ ] Detail view shows 3D model or video
- [ ] Creator attribution visible

#### 5.4 Checkout & Payment Flow
- [ ] Checkout modal:
  - Display item (game or cosmetic) + price
  - Stripe Elements form (card number, expiry, CVC, billing address)
  - "Pay [USD amount]" button
  - Loading state during payment processing
- [ ] API call:
  ```typescript
  const response = await fetch('/api/checkout', {
    method: 'POST',
    body: JSON.stringify({
      item_type: 'cosmetic',
      item_id: cosmetic_id,
      price_cents: 999,  // $9.99
      buyer_email: user.email,
    }),
  });
  const session = await response.json();
  // Redirect to Stripe: stripe.redirectToCheckout({ sessionId: session.id })
  ```
- [ ] Success page:
  - Confirmation message + order number
  - Download link (for games) or apply button (for cosmetics)
  - "Browse more" link back to store
- [ ] Error handling:
  - Network errors → retry UI
  - Payment declined → show Stripe error message
  - 3D Secure challenges → redirect to authentication

**Success Criteria:**
- [ ] Checkout modal displays
- [ ] Stripe card form renders
- [ ] Payment processes successfully (test environment)
- [ ] Success page shows after payment
- [ ] Error cases handled gracefully

#### 5.5 Creator Dashboard (MVP)
- [ ] View:
  - Total lifetime earnings
  - Earnings this month
  - Recent transactions (last 30 days)
  - Published games + cosmetics + tools
  - Payout history
  - Bank account status (verified/pending KYC)
- [ ] API integration:
  - Fetch creator profile via `/api/creator/me`
  - Query SPARQL for earnings: `SELECT SUM(?amount) WHERE { ?prov prov:wasAttributedTo ?creator . }`
  - List published items via `/api/creator/me/items`

**Success Criteria:**
- [ ] Creator can view earnings
- [ ] Payout history visible
- [ ] Published items listed

---

## Part 3: Implementation Schedule (Gantt View)

```
Week  1  2  3  4  5  6  7  8  9  10 11 12
├─────────────────────────────────────────────┤
Payment Infrastructure
├──────────────────────┤
RDF Linking
        ├──────────────────────┤
Creator Payouts
              ├──────────────────────┤
SPARQL Endpoints
                  ├──────────────────────┤
Marketplace UI
                        ├──────────────────────┤
Testing & Deployment
                                      ├───────┤
```

---

## Part 4: Risk & Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Stripe API rate limits during testing | Medium | Use Stripe test environment; stagger load tests |
| RDF store query performance at scale | High | Index commonly-queried properties; use Fuseki optimization |
| Payment webhook replay attacks | High | Implement idempotency keys; validate HMAC signatures |
| Creator KYC delays blocking payouts | Medium | Implement tiered payouts (initial threshold higher until KYC complete) |
| Cross-dimensional SPARQL queries slow | Medium | Materialize common queries; implement caching layer |
| UI integration delays with backend API | Medium | Parallel development with OpenAPI spec; mock API endpoints |

---

## Part 5: Definition of Done (Phase 6)

### Acceptance Criteria
- [ ] **Payment Processing**
  - [ ] Stripe checkout sessions created and processed
  - [ ] All transactions recorded in ledger with Stripe charge IDs
  - [ ] Test payments succeed in Stripe test environment
  - [ ] Production Stripe account configured and live key integrated
  
- [ ] **Creator Payouts**
  - [ ] Creator Stripe Connect accounts onboarded
  - [ ] Monthly settlement job runs without errors
  - [ ] All creators with balance > $50 paid within settlement window
  - [ ] Payment audit trail (PROV-O) complete and queryable
  
- [ ] **RDF Linking**
  - [ ] All games have RDF specs (converted from ggen output)
  - [ ] All cosmetics defined in RDF with creator & game linkage
  - [ ] Creator profiles queryable
  - [ ] Cross-game cosmetics link correctly
  
- [ ] **SPARQL Endpoints**
  - [ ] Fuseki instance running and responding
  - [ ] All 5 named graphs populated
  - [ ] Example queries documented and tested
  - [ ] Query performance <500ms for typical queries
  
- [ ] **Marketplace UI**
  - [ ] Game storefront page renders and filters work
  - [ ] Cosmetics storefront page renders and filters work
  - [ ] Checkout flow completes successfully
  - [ ] Success page displays after payment
  - [ ] Creator dashboard shows earnings and payouts
  - [ ] All pages responsive on mobile
  - [ ] No console errors or warnings
  
- [ ] **Integration Tests**
  - [ ] End-to-end: user purchases cosmetic → payment processed → cosmetic applied
  - [ ] End-to-end: monthly settlement → creator receives payout
  - [ ] SPARQL queries return correct cross-dimensional results
  - [ ] Payment webhook replay does not create duplicate ledger entries
  
- [ ] **Documentation**
  - [ ] Architecture diagrams (as above)
  - [ ] API documentation (OpenAPI/Swagger)
  - [ ] SPARQL query guide with examples
  - [ ] Creator onboarding guide (KYC, Stripe Connect, payout timeline)
  - [ ] Deployment runbook (Docker, environment variables, monitoring)

---

## Part 6: Success Metrics (End of Phase 6)

By end of week 12:

- **Technical**
  - [ ] 100% test coverage for payment + ledger + settlement logic
  - [ ] SPARQL query response time <500ms (p95)
  - [ ] Zero payment webhook failures
  - [ ] Uptime: 99.5% for marketplace UI & API
  
- **Business**
  - [ ] First 100 cosmetics published
  - [ ] First 50 creators onboarded
  - [ ] First $10K revenue processed
  - [ ] First monthly settlement completed (1,000+ USD distributed)
  - [ ] 1,000+ monthly active users (MAU)

---

## Next Phase (Phase 7)

After Phase 6 completes:
- Launch tools & ontology marketplaces (MCP plugins, Tera templates, Rust crates)
- Implement cross-dimensional SPARQL discovery patterns
- Launch creator councils & governance framework
- See VISION_2030_N_DIMENSIONAL_MARKETPLACE.md, Phase 7 section

---

**Document:** PHASE_6_IMPLEMENTATION_PLAN.md  
**Status:** Ready for Engineering Review  
**Last Updated:** 2026-06-18
