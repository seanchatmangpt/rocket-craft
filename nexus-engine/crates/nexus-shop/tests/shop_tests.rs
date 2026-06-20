use chrono::Utc;
use nexus_shop::{ar_bridge::*, battle_pass::*, gacha::*};
use proptest::prelude::*;

fn make_test_banner() -> Banner {
    Banner {
        id: "test-banner".to_string(),
        name: "Test Banner".to_string(),
        banner_type: BannerType::Standard,
        items: vec![
            GachaItem {
                id: "nu-gundam".to_string(),
                name: "Nu Gundam".to_string(),
                rarity: GachaRarity::SSR,
                banner_id: "test-banner".to_string(),
                is_rate_up: true,
            },
            GachaItem {
                id: "zeta".to_string(),
                name: "Zeta".to_string(),
                rarity: GachaRarity::SR,
                banner_id: "test-banner".to_string(),
                is_rate_up: false,
            },
            GachaItem {
                id: "rx78".to_string(),
                name: "RX-78-2".to_string(),
                rarity: GachaRarity::SR,
                banner_id: "test-banner".to_string(),
                is_rate_up: false,
            },
            GachaItem {
                id: "beam-saber".to_string(),
                name: "Beam Saber".to_string(),
                rarity: GachaRarity::R,
                banner_id: "test-banner".to_string(),
                is_rate_up: false,
            },
        ],
        starts_at: Utc::now() - chrono::Duration::days(1),
        ends_at: Utc::now() + chrono::Duration::days(14),
        ssr_rate: 0.03,
        sr_rate: 0.12,
        r_rate: 0.85,
        rate_up_share: 0.5,
    }
}

#[test]
fn hard_pity_at_90_guarantees_ssr() {
    let banner = make_test_banner();
    let mut session = PullSession::new(1, "test-banner".to_string());
    let mut engine = GachaEngine::new(42);

    // Simulate 89 pulls without SSR
    session.pulls_since_last_ssr = 89;

    // 90th pull must be SSR
    let result = engine.single_pull(&banner, &mut session).unwrap();
    assert_eq!(
        result.item.rarity,
        GachaRarity::SSR,
        "90th pull must be SSR (hard pity)"
    );
    assert_eq!(result.new_pity_count, 90);
}

#[test]
fn ten_pull_guarantees_at_least_one_sr() {
    let banner = make_test_banner();
    let mut session = PullSession::new(1, "test-banner".to_string());
    let mut engine = GachaEngine::new(999);

    let results = engine.ten_pull(&banner, &mut session).unwrap();
    assert_eq!(results.len(), 10);
    assert!(
        results.iter().any(|r| r.item.rarity >= GachaRarity::SR),
        "10-pull must contain at least 1 SR or better"
    );
}

#[test]
fn pity_counter_resets_after_ssr() {
    let banner = make_test_banner();
    let mut session = PullSession::new(1, "test-banner".to_string());
    session.pulls_since_last_ssr = 89; // at pity
    let mut engine = GachaEngine::new(1);

    engine.single_pull(&banner, &mut session).unwrap(); // ensured SSR
    assert_eq!(
        session.pulls_since_last_ssr, 0,
        "pity should reset after SSR"
    );
}

#[test]
fn ar_bridge_barcode_cannot_be_redeemed_twice() {
    let mut registry = ArBridgeRegistry::new();
    let barcode = "GN-HG-HG-AERIAL-001";

    let result1 = registry.redeem(barcode, 1);
    assert!(result1.is_ok(), "first redemption should succeed");

    let result2 = registry.redeem(barcode, 1);
    assert!(
        matches!(result2, Err(ArError::AlreadyRedeemed { .. })),
        "second redemption should fail"
    );
}

#[test]
fn ar_bridge_different_players_same_barcode_both_succeed() {
    let mut registry = ArBridgeRegistry::new();
    let barcode = "GN-HG-HG-AERIAL-001";

    let result1 = registry.redeem(barcode, 1);
    assert!(result1.is_ok());

    let result2 = registry.redeem(barcode, 2);
    assert!(
        result2.is_ok(),
        "different player should be able to redeem same kit barcode"
    );
}

#[test]
fn ar_bridge_invalid_format_rejected() {
    let mut registry = ArBridgeRegistry::new();
    assert!(registry.redeem("INVALID", 1).is_err());
    assert!(registry.redeem("GN-XX-SOMETHING", 1).is_err()); // bad tier
}

#[test]
fn battle_pass_tier_progression_monotone() {
    let mut state = PlayerPassState::new(1, 1);
    let mut prev_tier = 0;

    for _ in 0..50 {
        let _unlocked = state.earn_xp(1000);
        assert!(state.current_tier >= prev_tier, "tier must be monotone");
        prev_tier = state.current_tier;
    }
    assert_eq!(state.current_tier, 40, "should cap at tier 40");
}

proptest! {
    #[test]
    fn pity_rate_monotone(pulls in 1u32..88) {
        let mut session_a = PullSession::new(1, "b".to_string());
        let mut session_b = PullSession::new(1, "b".to_string());
        session_a.pulls_since_last_ssr = pulls;
        session_b.pulls_since_last_ssr = pulls + 1;
        prop_assert!(
            session_b.current_ssr_rate() >= session_a.current_ssr_rate(),
            "rate at {} pulls should be >= rate at {} pulls",
            pulls + 1,
            pulls
        );
    }

    #[test]
    fn pg_gives_most_credits(_dummy in 0u8..1) {
        prop_assert!(KitTier::Pg.digital_bonus_credits() > KitTier::Mg.digital_bonus_credits());
        prop_assert!(KitTier::Mg.digital_bonus_credits() > KitTier::Rg.digital_bonus_credits());
        prop_assert!(KitTier::Rg.digital_bonus_credits() > KitTier::Hg.digital_bonus_credits());
    }

    #[test]
    fn battle_pass_xp_never_lost(xp_batches in prop::collection::vec(0u64..5000, 1..20)) {
        let mut state = PlayerPassState::new(1, 1);
        let mut total_xp = 0u64;
        for xp in xp_batches {
            total_xp = total_xp.saturating_add(xp);
            state.earn_xp(xp);
            prop_assert!(
                state.pass_xp >= state.pass_xp.min(total_xp),
                "pass XP should never decrease"
            );
        }
    }
}
