//! Market event vocabulary for the GMF civilization economy.
//!
//! Every marketplace interaction is a process event. A listing is not a shop entry —
//! it is an object with a lifecycle: listed → bid → sold → delivered → receipted.
//!
//! Mars specializes in pricing future value (prediction markets, factory futures,
//! mission insurance, memecoin factions). Earth grows value. Mars prices it.
//! Jupiter enforces it. Saturn records it. Venus aestheticizes it.

use serde::{Deserialize, Serialize};

use crate::ocel::{OcelEvent, OcelObjectRef};

// ── Object types ─────────────────────────────────────────────────────────────

/// Every object type that participates in market events.
/// A listing is not a primitive — it references a Part, Mech, Contract, or Resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketObjectType {
    // Participants
    Player,
    Faction,
    Planet,

    // Market infrastructure
    Market,
    Listing,
    Bid,
    Order,

    // Value instruments
    Contract,
    InsurancePolicy,
    Claim,
    Prediction,
    Token,
    ReputationBond,

    // Physical assets (cross-referenced from manufacturing)
    Resource,
    Part,
    Mech,
    Crop,
    Factory,
    Receipt,

    // Flash-game production cells
    FarmCell,       // FarmVille loop: plant → tend → harvest → store → sell → feed Eden
    CityCell,       // CityVille loop: build → supply → collect → upgrade → repair
    FabCell,        // Papa's loop: order → station → timing → quality → reward
    RefineryCell,   // Candy Crush loop: sort minerals → socket batches → energy alignment
    DefenseCell,    // Tower defense loop: route → place → upgrade → survive
    TestRig,        // Happy Wheels loop: test → crash → diagnose → redesign
    Foundry,        // Doodle God loop: combine → discover recipe → unlock family
    AutomationLine, // Cookie Clicker loop: manual → automate → multiplier → prestige
}

impl MarketObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Player => "Player",
            Self::Faction => "Faction",
            Self::Planet => "Planet",
            Self::Market => "Market",
            Self::Listing => "Listing",
            Self::Bid => "Bid",
            Self::Order => "Order",
            Self::Contract => "Contract",
            Self::InsurancePolicy => "InsurancePolicy",
            Self::Claim => "Claim",
            Self::Prediction => "Prediction",
            Self::Token => "Token",
            Self::ReputationBond => "ReputationBond",
            Self::Resource => "Resource",
            Self::Part => "Part",
            Self::Mech => "Mech",
            Self::Crop => "Crop",
            Self::Factory => "Factory",
            Self::Receipt => "Receipt",
            Self::FarmCell => "FarmCell",
            Self::CityCell => "CityCell",
            Self::FabCell => "FabCell",
            Self::RefineryCell => "RefineryCell",
            Self::DefenseCell => "DefenseCell",
            Self::TestRig => "TestRig",
            Self::Foundry => "Foundry",
            Self::AutomationLine => "AutomationLine",
        }
    }
}

// ── Event kinds ───────────────────────────────────────────────────────────────

/// The minimum market event vocabulary. Every listing, bid, contract, and
/// prediction has a lifecycle, and every lifecycle step emits one of these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketEventKind {
    // ── Core market ──────────────────────────────────────────────────────────
    Listed,
    BidPlaced,
    BidRetracted,
    SaleCompleted,
    SaleCancelled,
    PriceChanged,

    // ── Order lifecycle ──────────────────────────────────────────────────────
    OrderCreated,
    OrderFulfilled,
    OrderDefaulted,
    OrderDisputed,

    // ── Contract lifecycle ───────────────────────────────────────────────────
    ContractIssued,
    ContractSigned,
    ContractSettled,
    ContractBreached,
    ContractExpired,

    // ── Insurance lifecycle ──────────────────────────────────────────────────
    InsuranceCreated,
    ClaimFiled,
    ClaimAssessed,
    ClaimPaid,
    ClaimRefused,

    // ── Prediction market (Mars speculative economy) ──────────────────────
    PredictionOpened,
    PredictionResolved,
    PredictionDefaulted,

    // ── Token / memecoin (Mars faction economy) ───────────────────────────
    TokenMinted,
    TokenTransferred,
    TokenBurned,
    TokenSlashed,

    // ── Receipt / proof exchange ──────────────────────────────────────────
    ReceiptRequired,
    ReceiptVerified,
    ReceiptRefused,
    ProofUpgraded,

    // ── Flash-game production cell events ────────────────────────────────
    // FarmVille loop
    CropPlanted,
    CropTended,
    CropHarvested,
    CropStored,
    FoodContractFulfilled,
    EdenFoodDelivered,

    // CityVille loop
    StructureBuilt,
    SupplyCollected,
    InfrastructureUpgraded,

    // Papa's loop
    FabOrderReceived,
    FabStationCompleted,
    FabQualityScored,

    // Candy Crush loop
    MineralSorted,
    SocketBatchCleared,
    EnergyAligned,

    // Tower defense loop
    ThreatRouteIdentified,
    DefensePlaced,
    SecurityContractPaid,

    // Happy Wheels loop
    MotionTestStarted,
    MotionTestCrashed,
    DesignReiterated,
    SafetyRatingIssued,

    // Doodle God loop
    ElementsCombined,
    RecipeDiscovered,
    MaterialFamilyUnlocked,

    // Cookie Clicker loop
    AutomationTriggered,
    MultiplierApplied,
    ThroughputPrestiged,

    // ── Reputation / story ─────────────────────────────────────────────────
    ReputationGranted,
    ReputationRevoked,
    StoryTagAttached,
    RarityClassified,

    // ── Planetary specialization events ───────────────────────────────────
    PlanetMarketOpened,
    PlanetRiskPriced,
    PlanetFutureCreated,
    FactionHypeEmitted,
}

impl MarketEventKind {
    pub fn activity_name(&self) -> &'static str {
        match self {
            Self::Listed => "market.listed",
            Self::BidPlaced => "market.bid_placed",
            Self::BidRetracted => "market.bid_retracted",
            Self::SaleCompleted => "market.sale_completed",
            Self::SaleCancelled => "market.sale_cancelled",
            Self::PriceChanged => "market.price_changed",
            Self::OrderCreated => "market.order_created",
            Self::OrderFulfilled => "market.order_fulfilled",
            Self::OrderDefaulted => "market.order_defaulted",
            Self::OrderDisputed => "market.order_disputed",
            Self::ContractIssued => "market.contract_issued",
            Self::ContractSigned => "market.contract_signed",
            Self::ContractSettled => "market.contract_settled",
            Self::ContractBreached => "market.contract_breached",
            Self::ContractExpired => "market.contract_expired",
            Self::InsuranceCreated => "market.insurance_created",
            Self::ClaimFiled => "market.claim_filed",
            Self::ClaimAssessed => "market.claim_assessed",
            Self::ClaimPaid => "market.claim_paid",
            Self::ClaimRefused => "market.claim_refused",
            Self::PredictionOpened => "market.prediction_opened",
            Self::PredictionResolved => "market.prediction_resolved",
            Self::PredictionDefaulted => "market.prediction_defaulted",
            Self::TokenMinted => "market.token_minted",
            Self::TokenTransferred => "market.token_transferred",
            Self::TokenBurned => "market.token_burned",
            Self::TokenSlashed => "market.token_slashed",
            Self::ReceiptRequired => "market.receipt_required",
            Self::ReceiptVerified => "market.receipt_verified",
            Self::ReceiptRefused => "market.receipt_refused",
            Self::ProofUpgraded => "market.proof_upgraded",
            Self::CropPlanted => "farm.crop_planted",
            Self::CropTended => "farm.crop_tended",
            Self::CropHarvested => "farm.crop_harvested",
            Self::CropStored => "farm.crop_stored",
            Self::FoodContractFulfilled => "farm.food_contract_fulfilled",
            Self::EdenFoodDelivered => "eden.food_delivered",
            Self::StructureBuilt => "city.structure_built",
            Self::SupplyCollected => "city.supply_collected",
            Self::InfrastructureUpgraded => "city.infrastructure_upgraded",
            Self::FabOrderReceived => "fab.order_received",
            Self::FabStationCompleted => "fab.station_completed",
            Self::FabQualityScored => "fab.quality_scored",
            Self::MineralSorted => "refinery.mineral_sorted",
            Self::SocketBatchCleared => "refinery.socket_batch_cleared",
            Self::EnergyAligned => "refinery.energy_aligned",
            Self::ThreatRouteIdentified => "defense.threat_route_identified",
            Self::DefensePlaced => "defense.placed",
            Self::SecurityContractPaid => "defense.security_contract_paid",
            Self::MotionTestStarted => "test_rig.motion_test_started",
            Self::MotionTestCrashed => "test_rig.motion_test_crashed",
            Self::DesignReiterated => "test_rig.design_reiterated",
            Self::SafetyRatingIssued => "test_rig.safety_rating_issued",
            Self::ElementsCombined => "foundry.elements_combined",
            Self::RecipeDiscovered => "foundry.recipe_discovered",
            Self::MaterialFamilyUnlocked => "foundry.material_family_unlocked",
            Self::AutomationTriggered => "automation.triggered",
            Self::MultiplierApplied => "automation.multiplier_applied",
            Self::ThroughputPrestiged => "automation.throughput_prestiged",
            Self::ReputationGranted => "reputation.granted",
            Self::ReputationRevoked => "reputation.revoked",
            Self::StoryTagAttached => "story.tag_attached",
            Self::RarityClassified => "rarity.classified",
            Self::PlanetMarketOpened => "planet.market_opened",
            Self::PlanetRiskPriced => "planet.risk_priced",
            Self::PlanetFutureCreated => "planet.future_created",
            Self::FactionHypeEmitted => "planet.faction_hype_emitted",
        }
    }

    /// Which planet does this event naturally belong to?
    /// Earth grows value. Mars prices it. Jupiter enforces it.
    /// Saturn records it. Venus aestheticizes it.
    pub fn planetary_affinity(&self) -> &'static str {
        match self {
            // Earth: real productive activity
            Self::CropPlanted | Self::CropTended | Self::CropHarvested
            | Self::EdenFoodDelivered | Self::StructureBuilt | Self::InfrastructureUpgraded => "Earth",

            // Mars: speculation, risk, prediction, tokens
            Self::PredictionOpened | Self::PredictionResolved | Self::PredictionDefaulted
            | Self::TokenMinted | Self::TokenTransferred | Self::TokenBurned | Self::TokenSlashed
            | Self::PlanetRiskPriced | Self::PlanetFutureCreated | Self::FactionHypeEmitted => "Mars",

            // Jupiter: enforcement, contracts, settlement
            Self::ContractIssued | Self::ContractSigned | Self::ContractSettled
            | Self::ContractBreached | Self::OrderFulfilled | Self::OrderDefaulted
            | Self::ClaimPaid | Self::ClaimRefused => "Jupiter",

            // Saturn: records, receipts, proofs, memory
            Self::ReceiptRequired | Self::ReceiptVerified | Self::ReceiptRefused
            | Self::ProofUpgraded | Self::ReputationGranted | Self::ReputationRevoked
            | Self::StoryTagAttached | Self::RarityClassified => "Saturn",

            // Venus: beauty, aesthetics, story, rarity
            Self::MaterialFamilyUnlocked | Self::RecipeDiscovered | Self::RarityClassified => "Venus",

            // Universal
            _ => "Universal",
        }
    }
}

// ── Event builder ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: String,
    pub kind: MarketEventKind,
    pub timestamp_ms: u64,
    pub object_refs: Vec<OcelObjectRef>,
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

impl MarketEvent {
    pub fn new(id: impl Into<String>, kind: MarketEventKind, timestamp_ms: u64) -> Self {
        Self {
            id: id.into(),
            kind,
            timestamp_ms,
            object_refs: Vec::new(),
            attributes: serde_json::Map::new(),
        }
    }

    pub fn with_object(mut self, object_id: impl Into<String>, qualifier: impl Into<String>) -> Self {
        self.object_refs.push(OcelObjectRef {
            object_id: object_id.into(),
            qualifier: qualifier.into(),
        });
        self
    }

    pub fn with_attr(mut self, key: impl Into<String>, val: impl Into<serde_json::Value>) -> Self {
        self.attributes.insert(key.into(), val.into());
        self
    }

    pub fn into_ocel_event(self) -> OcelEvent {
        OcelEvent {
            id: self.id,
            activity: self.kind.activity_name().to_string(),
            timestamp_ms: self.timestamp_ms,
            object_refs: self.object_refs,
            attributes: self.attributes,
        }
    }
}

// ── Factory functions ─────────────────────────────────────────────────────────

/// A rich market listing: not just price, but provenance, history, and proof.
pub fn listing_event(
    listing_id: impl Into<String>,
    asset_id: impl Into<String>,
    seller_id: impl Into<String>,
    market_id: impl Into<String>,
    listing: ListingMetadata,
    event_id: impl Into<String>,
    ts: u64,
) -> MarketEvent {
    MarketEvent::new(event_id, MarketEventKind::Listed, ts)
        .with_object(listing_id, "listing")
        .with_object(asset_id, "listed_asset")
        .with_object(seller_id, "seller")
        .with_object(market_id, "market")
        .with_attr("price_credits", listing.price_credits as i64)
        .with_attr("planet_origin", listing.planet_origin)
        .with_attr("material_batch", listing.material_batch)
        .with_attr("thermal_rating", listing.thermal_rating as i64)
        .with_attr("battle_history_missions", listing.battle_history_missions as i64)
        .with_attr("repair_count", listing.repair_count as i64)
        .with_attr("boundary_proof_present", listing.boundary_proof_present)
        .with_attr("health_level", listing.health_level as i64)
        .with_attr("delivery_risk", listing.delivery_risk)
        .with_attr("rarity_story", listing.rarity_story)
}

/// A Mars prediction market: price a future event before the log proves it.
pub fn prediction_event(
    prediction_id: impl Into<String>,
    predictor_id: impl Into<String>,
    market_id: impl Into<String>,
    subject: impl Into<String>,
    stake_credits: u64,
    event_id: impl Into<String>,
    ts: u64,
) -> MarketEvent {
    MarketEvent::new(event_id, MarketEventKind::PredictionOpened, ts)
        .with_object(prediction_id, "prediction")
        .with_object(predictor_id, "predictor")
        .with_object(market_id, "market")
        .with_attr("subject", subject.into())
        .with_attr("stake_credits", stake_credits as i64)
        .with_attr("planetary_affinity", "Mars")
}

/// Factory chain: the full session of a FarmVille production cell.
/// Returns events for: plant → tend → harvest → sell contract → deliver to Eden.
pub fn farm_session_events(
    crop_id: impl Into<String> + Clone,
    farm_cell_id: impl Into<String> + Clone,
    player_id: impl Into<String> + Clone,
    eden_node_id: impl Into<String>,
    base_ts: u64,
) -> Vec<MarketEvent> {
    let crop = crop_id.into();
    let farm = farm_cell_id.into();
    let player = player_id.into();

    vec![
        MarketEvent::new("farm_plant", MarketEventKind::CropPlanted, base_ts)
            .with_object(crop.clone(), "crop")
            .with_object(farm.clone(), "cell")
            .with_object(player.clone(), "farmer"),

        MarketEvent::new("farm_tend", MarketEventKind::CropTended, base_ts + 1000)
            .with_object(crop.clone(), "crop")
            .with_object(player.clone(), "farmer"),

        MarketEvent::new("farm_harvest", MarketEventKind::CropHarvested, base_ts + 5000)
            .with_object(crop.clone(), "crop")
            .with_object(farm.clone(), "cell")
            .with_attr("yield_kg", 42_i64),

        MarketEvent::new("farm_deliver", MarketEventKind::EdenFoodDelivered, base_ts + 6000)
            .with_object(crop.clone(), "crop")
            .with_object(eden_node_id, "eden_node")
            .with_attr("civilization_benefit", "food_security"),
    ]
}

/// Mars token: faction hype minted as a market object.
pub fn faction_token_mint(
    token_id: impl Into<String>,
    faction_id: impl Into<String>,
    minter_id: impl Into<String>,
    supply: u64,
    event_id: impl Into<String>,
    ts: u64,
) -> MarketEvent {
    MarketEvent::new(event_id, MarketEventKind::TokenMinted, ts)
        .with_object(token_id, "token")
        .with_object(faction_id, "issuing_faction")
        .with_object(minter_id, "minter")
        .with_attr("initial_supply", supply as i64)
        .with_attr("planetary_affinity", "Mars")
        .with_attr("value_basis", "attention_and_belief")
}

// ── Metadata types ────────────────────────────────────────────────────────────

/// Rich provenance metadata for a market listing.
/// This is the n-dimensional listing the orchestrator described:
/// Object × Process × Planet × Risk × Time × Reliability × Scarcity × Reputation × Proof × Story.
#[derive(Debug, Clone)]
pub struct ListingMetadata {
    pub price_credits: u64,
    pub planet_origin: String,
    pub material_batch: String,
    /// 0–10 thermal resistance rating
    pub thermal_rating: u8,
    /// Number of missions the asset survived before listing
    pub battle_history_missions: u32,
    pub repair_count: u32,
    pub boundary_proof_present: bool,
    /// 0=Healthy, 4=Failed
    pub health_level: u8,
    /// "low" | "medium" | "high" | "extreme"
    pub delivery_risk: String,
    /// Narrative tag: "meme-backed faction artifact", "salvage relic", etc.
    pub rarity_story: String,
}

impl ListingMetadata {
    pub fn simple(price_credits: u64, planet_origin: impl Into<String>) -> Self {
        Self {
            price_credits,
            planet_origin: planet_origin.into(),
            material_batch: "standard".into(),
            thermal_rating: 5,
            battle_history_missions: 0,
            repair_count: 0,
            boundary_proof_present: false,
            health_level: 0,
            delivery_risk: "low".into(),
            rarity_story: "standard issue".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MarketObjectType ──────────────────────────────────────────────────────

    #[test]
    fn market_object_type_as_str_nonempty() {
        let types = [
            MarketObjectType::Player, MarketObjectType::Listing,
            MarketObjectType::Mech, MarketObjectType::FarmCell,
        ];
        for t in &types {
            assert!(!t.as_str().is_empty(), "{:?}.as_str() must not be empty", t);
        }
    }

    // ── MarketEventKind ───────────────────────────────────────────────────────

    #[test]
    fn event_kind_activity_names_are_nonempty() {
        let kinds = [
            MarketEventKind::Listed, MarketEventKind::BidPlaced,
            MarketEventKind::SaleCompleted, MarketEventKind::CropHarvested,
        ];
        for k in &kinds {
            assert!(!k.activity_name().is_empty(), "{:?}.activity_name() must not be empty", k);
        }
    }

    #[test]
    fn event_kind_planetary_affinity_nonempty() {
        assert!(!MarketEventKind::TokenMinted.planetary_affinity().is_empty());
        assert!(!MarketEventKind::CropPlanted.planetary_affinity().is_empty());
    }

    // ── MarketEvent ───────────────────────────────────────────────────────────

    #[test]
    fn market_event_new_sets_fields() {
        let ev = MarketEvent::new("ev-001", MarketEventKind::Listed, 1000);
        assert_eq!(ev.id, "ev-001");
        assert_eq!(ev.timestamp_ms, 1000);
        assert!(ev.object_refs.is_empty());
        assert!(ev.attributes.is_empty());
    }

    #[test]
    fn with_object_appends_ref() {
        let ev = MarketEvent::new("ev-1", MarketEventKind::BidPlaced, 0)
            .with_object("listing-42", "subject");
        assert_eq!(ev.object_refs.len(), 1);
        assert_eq!(ev.object_refs[0].object_id, "listing-42");
        assert_eq!(ev.object_refs[0].qualifier, "subject");
    }

    #[test]
    fn with_attr_inserts_value() {
        let ev = MarketEvent::new("ev-2", MarketEventKind::SaleCompleted, 0)
            .with_attr("price", 500_i64);
        assert!(ev.attributes.contains_key("price"));
    }

    #[test]
    fn into_ocel_event_preserves_id_and_timestamp() {
        let ev = MarketEvent::new("ev-3", MarketEventKind::ReceiptVerified, 9999)
            .into_ocel_event();
        assert_eq!(ev.id, "ev-3");
        assert_eq!(ev.timestamp_ms, 9999);
        assert!(!ev.activity.is_empty());
    }

    // ── factory functions ─────────────────────────────────────────────────────

    #[test]
    fn listing_event_sets_id_and_kind() {
        let meta = ListingMetadata::simple(1000, "Earth");
        let ev = listing_event("lst-1", "asset-1", "seller-1", "mkt-1", meta, "ev-lst-1", 100);
        assert_eq!(ev.id, "ev-lst-1");
        // factory attaches at least 4 object refs (listing, asset, seller, market)
        assert!(ev.object_refs.len() >= 4);
    }

    #[test]
    fn prediction_event_sets_id() {
        let ev = prediction_event("pred-1", "player-1", "mkt-1", "will-battle-win", 250, "ev-p-1", 200);
        assert_eq!(ev.id, "ev-p-1");
        assert_eq!(ev.timestamp_ms, 200);
    }

    #[test]
    fn farm_session_events_returns_multiple_events() {
        let events = farm_session_events("crop-1", "farm-1", "player-1", "eden-1", 300);
        assert!(events.len() >= 4, "farm session should produce plant/tend/harvest/deliver");
    }

    #[test]
    fn faction_token_mint_sets_id() {
        let ev = faction_token_mint("tok-1", "faction-moon", "player-1", 100, "ev-mint-1", 400);
        assert_eq!(ev.id, "ev-mint-1");
        assert!(ev.attributes.contains_key("initial_supply"));
    }

    // ── ListingMetadata ───────────────────────────────────────────────────────

    #[test]
    fn listing_metadata_simple_sets_price() {
        let m = ListingMetadata::simple(500, "Mars");
        assert_eq!(m.price_credits, 500);
        assert_eq!(m.planet_origin, "Mars");
    }
}
