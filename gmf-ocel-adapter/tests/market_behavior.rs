//! GMF Market — Chicago-style behavior tests.
//! A listing is not a shop entry. It is a lifecycle with provenance, proof, and story.
//! Every market interaction emits events that can be mined into a process twin.

use gmf_ocel_adapter::{
    market::{
        ListingMetadata, MarketEventKind, faction_token_mint, farm_session_events,
        listing_event, prediction_event,
    },
    ocel::{OcelLog, OcelObject},
};

// ── Listing ──────────────────────────────────────────────────────────────────

#[test]
fn listing_event_has_correct_activity_name() {
    let meta = ListingMetadata::simple(500, "Mars");
    let ev = listing_event("listing_001", "part_ArmL_001", "player_001", "market_mars", meta, "ev_list_001", 1000);
    assert_eq!(ev.kind.activity_name(), "market.listed");
}

#[test]
fn listing_event_references_listing_asset_seller_market() {
    let meta = ListingMetadata::simple(750, "Earth");
    let ev = listing_event("listing_002", "mech_001", "player_002", "market_earth", meta, "ev_list_002", 2000);
    let ids: Vec<&str> = ev.object_refs.iter().map(|r| r.object_id.as_str()).collect();
    assert!(ids.contains(&"listing_002"), "Must reference listing object");
    assert!(ids.contains(&"mech_001"), "Must reference listed asset");
    assert!(ids.contains(&"player_002"), "Must reference seller");
    assert!(ids.contains(&"market_earth"), "Must reference market");
}

#[test]
fn listing_encodes_full_n_dimensional_provenance() {
    let meta = ListingMetadata {
        price_credits: 9500,
        planet_origin: "Mars".into(),
        material_batch: "Olympus-Red-42".into(),
        thermal_rating: 7,
        battle_history_missions: 3,
        repair_count: 1,
        boundary_proof_present: true,
        health_level: 2, // Degraded
        delivery_risk: "high".into(),
        rarity_story: "meme-backed faction artifact".into(),
    };
    let ev = listing_event("listing_003", "part_Torso_001", "player_003", "market_mars", meta, "ev_list_003", 3000);

    assert_eq!(ev.attributes["price_credits"].as_i64(), Some(9500));
    assert_eq!(ev.attributes["planet_origin"].as_str(), Some("Mars"));
    assert_eq!(ev.attributes["material_batch"].as_str(), Some("Olympus-Red-42"));
    assert_eq!(ev.attributes["thermal_rating"].as_i64(), Some(7));
    assert_eq!(ev.attributes["battle_history_missions"].as_i64(), Some(3));
    assert_eq!(ev.attributes["repair_count"].as_i64(), Some(1));
    assert_eq!(ev.attributes["boundary_proof_present"].as_bool(), Some(true));
    assert_eq!(ev.attributes["health_level"].as_i64(), Some(2));
    assert_eq!(ev.attributes["delivery_risk"].as_str(), Some("high"));
    assert_eq!(ev.attributes["rarity_story"].as_str(), Some("meme-backed faction artifact"));
}

// ── Prediction market (Mars) ──────────────────────────────────────────────────

#[test]
fn prediction_event_has_correct_activity_name() {
    let ev = prediction_event("pred_001", "player_001", "market_mars", "will_convoy_survive", 1000, "ev_pred_001", 5000);
    assert_eq!(ev.kind.activity_name(), "market.prediction_opened");
}

#[test]
fn prediction_event_has_mars_planetary_affinity() {
    assert_eq!(MarketEventKind::PredictionOpened.planetary_affinity(), "Mars");
    assert_eq!(MarketEventKind::PredictionResolved.planetary_affinity(), "Mars");
    assert_eq!(MarketEventKind::TokenMinted.planetary_affinity(), "Mars");
    assert_eq!(MarketEventKind::FactionHypeEmitted.planetary_affinity(), "Mars");
}

#[test]
fn prediction_event_encodes_subject_and_stake() {
    let ev = prediction_event("pred_002", "player_002", "market_mars", "factory_throughput_exceeds_1000", 500, "ev_pred_002", 6000);
    assert_eq!(ev.attributes["subject"].as_str(), Some("factory_throughput_exceeds_1000"));
    assert_eq!(ev.attributes["stake_credits"].as_i64(), Some(500));
    assert_eq!(ev.attributes["planetary_affinity"].as_str(), Some("Mars"));
}

// ── Planetary affinity law ─────────────────────────────────────────────────────

#[test]
fn earth_events_have_earth_affinity() {
    assert_eq!(MarketEventKind::CropPlanted.planetary_affinity(), "Earth");
    assert_eq!(MarketEventKind::EdenFoodDelivered.planetary_affinity(), "Earth");
    assert_eq!(MarketEventKind::StructureBuilt.planetary_affinity(), "Earth");
}

#[test]
fn jupiter_events_have_jupiter_affinity() {
    assert_eq!(MarketEventKind::ContractSettled.planetary_affinity(), "Jupiter");
    assert_eq!(MarketEventKind::OrderFulfilled.planetary_affinity(), "Jupiter");
    assert_eq!(MarketEventKind::ClaimPaid.planetary_affinity(), "Jupiter");
    assert_eq!(MarketEventKind::ContractBreached.planetary_affinity(), "Jupiter");
}

#[test]
fn saturn_events_have_saturn_affinity() {
    assert_eq!(MarketEventKind::ReceiptVerified.planetary_affinity(), "Saturn");
    assert_eq!(MarketEventKind::ReputationGranted.planetary_affinity(), "Saturn");
    assert_eq!(MarketEventKind::StoryTagAttached.planetary_affinity(), "Saturn");
}

// ── Token / memecoin (Mars faction economy) ─────────────────────────────────────

#[test]
fn faction_token_mint_has_correct_activity_name() {
    let ev = faction_token_mint("token_ARES", "faction_red_mars", "player_001", 1_000_000, "ev_mint_001", 7000);
    assert_eq!(ev.kind.activity_name(), "market.token_minted");
}

#[test]
fn faction_token_mint_encodes_mars_value_basis() {
    let ev = faction_token_mint("token_ARES", "faction_red_mars", "player_001", 1_000_000, "ev_mint_002", 8000);
    assert_eq!(ev.attributes["planetary_affinity"].as_str(), Some("Mars"));
    assert_eq!(ev.attributes["value_basis"].as_str(), Some("attention_and_belief"));
    assert_eq!(ev.attributes["initial_supply"].as_i64(), Some(1_000_000));
}

// ── FarmVille production cell (Earth → Eden chain) ──────────────────────────────

#[test]
fn farm_session_produces_four_events() {
    let evs = farm_session_events("crop_wheat_001", "farm_cell_001", "player_001", "eden_node_A1", 10000);
    assert_eq!(evs.len(), 4, "FarmVille loop must emit: plant → tend → harvest → deliver");
}

#[test]
fn farm_session_follows_lawful_temporal_order() {
    let evs = farm_session_events("crop_corn_001", "farm_cell_002", "player_002", "eden_node_B2", 20000);
    let timestamps: Vec<u64> = evs.iter().map(|e| e.timestamp_ms).collect();
    let is_monotonic = timestamps.windows(2).all(|w| w[0] <= w[1]);
    assert!(is_monotonic, "Farm session events must be temporally ordered: {timestamps:?}");
}

#[test]
fn farm_session_last_event_delivers_to_eden() {
    let evs = farm_session_events("crop_rye_001", "farm_cell_003", "player_003", "eden_node_C3", 30000);
    let last = evs.last().unwrap();
    assert_eq!(last.kind.activity_name(), "eden.food_delivered");
    let ids: Vec<&str> = last.object_refs.iter().map(|r| r.object_id.as_str()).collect();
    assert!(ids.contains(&"eden_node_C3"), "Must deliver to Eden node");
}

#[test]
fn farm_events_reference_crop_throughout_lifecycle() {
    let evs = farm_session_events("crop_hemp_001", "farm_cell_004", "player_004", "eden_node_D4", 40000);
    for ev in &evs {
        let ids: Vec<&str> = ev.object_refs.iter().map(|r| r.object_id.as_str()).collect();
        assert!(ids.contains(&"crop_hemp_001"),
            "Event '{}' must reference crop throughout its lifecycle", ev.kind.activity_name());
    }
}

// ── Market OCEL log integration ───────────────────────────────────────────────

#[test]
fn market_events_integrate_into_ocel_log() {
    let mut log = OcelLog::default();

    log.add_object(OcelObject::new("listing_001", "Listing"));
    log.add_object(OcelObject::new("part_ArmL_001", "Part"));
    log.add_object(OcelObject::new("player_001", "Player"));
    log.add_object(OcelObject::new("market_mars", "Market"));

    let meta = ListingMetadata {
        price_credits: 9500,
        planet_origin: "Mars".into(),
        material_batch: "Olympus-Red-42".into(),
        thermal_rating: 7,
        battle_history_missions: 3,
        repair_count: 1,
        boundary_proof_present: true,
        health_level: 2,
        delivery_risk: "high".into(),
        rarity_story: "meme-backed faction artifact".into(),
    };
    let ev = listing_event("listing_001", "part_ArmL_001", "player_001", "market_mars", meta, "ev_001", 1000);
    log.add_event(ev.into_ocel_event());

    let violations = log.validate();
    assert!(violations.is_empty(), "Market OCEL must be valid: {violations:?}");
}

// ── All market event kinds have dot-namespaced names ──────────────────────────

#[test]
fn all_market_event_kinds_have_dot_namespaced_activity_names() {
    let representative = vec![
        MarketEventKind::Listed,
        MarketEventKind::BidPlaced,
        MarketEventKind::SaleCompleted,
        MarketEventKind::OrderFulfilled,
        MarketEventKind::ContractIssued,
        MarketEventKind::ContractSettled,
        MarketEventKind::InsuranceCreated,
        MarketEventKind::ClaimPaid,
        MarketEventKind::PredictionOpened,
        MarketEventKind::TokenMinted,
        MarketEventKind::ReceiptRequired,
        MarketEventKind::CropPlanted,
        MarketEventKind::EdenFoodDelivered,
        MarketEventKind::FabOrderReceived,
        MarketEventKind::MineralSorted,
        MarketEventKind::DefensePlaced,
        MarketEventKind::MotionTestCrashed,
        MarketEventKind::RecipeDiscovered,
        MarketEventKind::AutomationTriggered,
        MarketEventKind::ReputationGranted,
        MarketEventKind::PlanetRiskPriced,
        MarketEventKind::FactionHypeEmitted,
    ];
    for kind in representative {
        let name = kind.activity_name();
        assert!(name.contains('.'), "Market activity_name must be dot-namespaced: '{name}'");
    }
}

// ── Flash-game loop activity names follow namespacing convention ──────────────

#[test]
fn flash_game_loops_use_cell_namespaced_activity_names() {
    assert!(MarketEventKind::CropPlanted.activity_name().starts_with("farm."));
    assert!(MarketEventKind::StructureBuilt.activity_name().starts_with("city."));
    assert!(MarketEventKind::FabOrderReceived.activity_name().starts_with("fab."));
    assert!(MarketEventKind::MineralSorted.activity_name().starts_with("refinery."));
    assert!(MarketEventKind::DefensePlaced.activity_name().starts_with("defense."));
    assert!(MarketEventKind::MotionTestStarted.activity_name().starts_with("test_rig."));
    assert!(MarketEventKind::ElementsCombined.activity_name().starts_with("foundry."));
    assert!(MarketEventKind::AutomationTriggered.activity_name().starts_with("automation."));
}
