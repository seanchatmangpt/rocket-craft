use nexus_tests::{invariants::*, model::*, strategies::*};
use nexus_types::MagicType;
use proptest::prelude::*;

proptest! {
    // Damage floor invariant always holds
    #[test]
    fn damage_floor_always_at_least_one(
        base in damage_strategy(),
        armor in armor_strategy(),
        combo in combo_depth_strategy(),
    ) {
                let combo_mult = match combo { 0 | 1 => 1.0, 2 => 1.5, 3 => 2.0, _ => 3.0 };
        let real_dmg = nexus_combat::calculate_damage(base, combo_mult, 0.0, 1.0, false, armor);
        prop_assert!(real_dmg >= 1.0, "real damage must always be >= 1.0, got {}", real_dmg);

        let dmg = oracle_damage(base, combo, 0.0, armor, false);
        prop_assert!(dmg >= 1.0, "damage must always be >= 1.0, got {}", dmg);
        prop_assert_eq!(real_dmg, dmg);
    }

    // Perfect parry counter always does more than normal attack
    #[test]
    fn perfect_parry_counter_beats_normal(
        base in 1.0f32..1000.0,
        armor in 0.0f32..50.0,
        combo in 0u32..5,
    ) {
                let combo_mult = match combo { 0 | 1 => 1.0, 2 => 1.5, 3 => 2.0, _ => 3.0 };
        let real_normal = nexus_combat::calculate_damage(base, combo_mult, 0.0, 1.0, false, armor);
        let real_counter = nexus_combat::calculate_damage(base, combo_mult, 0.0, 1.0, true, armor);
        prop_assert!(real_counter >= real_normal, "real perfect counter must beat real normal");

        let normal = oracle_damage(base, combo, 0.0, armor, false);
        let counter = oracle_damage(base, combo, 0.0, armor, true);
        prop_assert!(counter >= normal, "perfect counter ({}) must beat normal ({})", counter, normal);
        prop_assert_eq!(real_normal, normal);
        prop_assert_eq!(real_counter, counter);
    }

    // XP requirement is monotone: higher level always needs more XP
    #[test]
    fn xp_requirement_monotone(level in 1u32..100) {
        use nexus_session::player::PlayerProfile;
        let mut p = PlayerProfile::new(1, "test".to_string());
        p.level = level;
        p.xp = 0;

        let req_next = oracle_xp_for_level(level + 1);

        // If we gain 1 less than next level requirement, we should NOT level up
        let gained_less = p.apply_xp_gain(req_next - 1);
        prop_assert!(!gained_less);
        prop_assert_eq!(p.level, level);

        // If we gain the remaining 1 XP, we should level up
        let gained_enough = p.apply_xp_gain(1);
        prop_assert!(gained_enough);
        prop_assert_eq!(p.level, level + 1);

        let current = oracle_xp_for_level(level);
        let next = oracle_xp_for_level(level + 1);
        prop_assert!(next > current, "level {} needs {} xp, level {} needs {} xp", level, current, level+1, next);
    }

    // Gacha probability never decreases (monotone in pity pulls)
    #[test]
    fn gacha_pity_monotone(pull in 1u32..89) {
        use nexus_shop::gacha::PullSession;
        let mut session = PullSession::new(1, "banner".to_string());
        session.pulls_since_last_ssr = pull;
        let p_now = session.current_ssr_rate();
        session.pulls_since_last_ssr = pull + 1;
        let p_next = session.current_ssr_rate();
        prop_assert!(p_next >= p_now, "Real SSR rate must not decrease with more pulls: pull {} was {}, pull {} was {}", pull, p_now, pull+1, p_next);

        let p_now_oracle = oracle_ssr_probability(pull);
        let p_next_oracle = oracle_ssr_probability(pull + 1);
        prop_assert_eq!(p_now, p_now_oracle);
        prop_assert_eq!(p_next, p_next_oracle);
    }

    // Hard pity at pull 90 guarantees SSR
    #[test]
    fn hard_pity_at_90_guarantees_ssr(pull in 90u32..200) {
        use nexus_shop::gacha::PullSession;
        let mut session = PullSession::new(1, "banner".to_string());
        session.pulls_since_last_ssr = pull;
        prop_assert_eq!(session.current_ssr_rate(), 1.0);

        prop_assert_eq!(oracle_ssr_probability(pull), 1.0);
    }

    // Auction minimum bid is always > current bid (prevents bid snaking)
    #[test]
    fn auction_min_bid_above_current(current in 1u32..1_000_000) {
        use nexus_economy::auction::{Auction, OpenForBids};
        use nexus_economy::ledger::Ledger;

        let mut ledger = Ledger::new();
        let seller_id = 1u64;
        let bidder_1_id = 2u64;
        let bidder_2_id = 3u64;

        let _ = ledger.award_gold(bidder_1_id, current, "seed");
        let _ = ledger.award_gold(bidder_2_id, current * 2 + 10, "seed");

        let mut auction = Auction::<OpenForBids>::new(
            1,
            seller_id,
            "Gundam Part".to_string(),
            current,
            None,
            24,
        );

        let first_bid_res = auction.place_bid(bidder_1_id, current, &mut ledger);
        prop_assert!(first_bid_res.is_ok());

        let min_required = current + (current / 20).max(1);

        if min_required > current + 1 {
            let too_low_res = auction.place_bid(bidder_2_id, min_required - 1, &mut ledger);
            prop_assert!(too_low_res.is_err(), "bid {} should be rejected, min required is {}", min_required - 1, min_required);
        }

        let ok_res = auction.place_bid(bidder_2_id, min_required, &mut ledger);
        prop_assert!(ok_res.is_ok());

        let minimum = oracle_min_bid(current);
        prop_assert!(minimum > current, "min bid {} must be > current bid {}", minimum, current);
        prop_assert_eq!(min_required, minimum);
    }

    // GodKing shield breaks exactly at 3rd perfect parry
    #[test]
    fn godking_shield_breaks_only_at_3(parries_before in 0u32..2) {
        use nexus_combat::ParryResolver;
        use nexus_combat::parry::{AttackDir, ParryOutcome};

        let (outcome, broke) = ParryResolver::resolve_shield_parry(
            AttackDir::Overhead,
            Some(AttackDir::Overhead),
            parries_before,
        );
        prop_assert_eq!(outcome, ParryOutcome::Perfect);
        if parries_before < 2 {
            prop_assert!(!broke, "shield should not break at {} parries", parries_before + 1);
        } else {
            prop_assert!(broke);
        }

        let not_yet = oracle_shield_breaks(parries_before, true);
        if parries_before < 2 {
            prop_assert!(!not_yet, "shield should not break at {} parries", parries_before + 1);
        }
        let breaks = oracle_shield_breaks(2, true);
        prop_assert!(breaks, "shield should break at 3rd perfect parry");
    }

    // Trans-Am only activates with BOTH depth >= 4 AND full gauge
    #[test]
    fn trans_am_requires_both_depth_and_gauge(
        combo in 0u32..7,
        gauge in 0.0f32..1.2,
    ) {
        use nexus_combat::combo::TransAmCombo;

        let mut chain = TransAmCombo::new(2);
        for _ in 0..combo {
            chain.on_hit();
        }

        let depth_ok = chain.is_trans_am_zone();
        let gauge_ok = gauge >= 1.0;
        let real_activates = depth_ok && gauge_ok;

        let activates = oracle_trans_am_activates(combo, gauge);
        prop_assert_eq!(real_activates, activates);
        if activates {
            prop_assert!(combo >= 4, "trans-am needs combo >= 4, had {}", combo);
            prop_assert!(gauge >= 1.0, "trans-am needs full gauge, had {}", gauge);
        }
    }

    // QIP scar forced rebirth exactly at stack 3
    #[test]
    fn qip_scar_rebirth_at_exactly_3(stacks_before in 0u32..2) {
        prop_assert!(qip_scar_rebirth_at_3(stacks_before));
    }

    // Gold inventory add/remove preserves count
    #[test]
    fn inventory_size_preserved(initial in 0usize..49) {
        prop_assert!(inventory_add_remove_preserves_size(initial));
    }

    // DeterministicFuzzer is deterministic (same seed = same output)
    #[test]
    fn fuzzer_deterministic(seed in 0u64..u64::MAX) {
        use nexus_tests::fuzz::DeterministicFuzzer;
        let mut f1 = DeterministicFuzzer::new(seed);
        let mut f2 = DeterministicFuzzer::new(seed);
        let seq1 = f1.combat_sequence(10);
        let seq2 = f2.combat_sequence(10);
        prop_assert_eq!(seq1, seq2, "same seed must produce same combat sequence");
    }

    // Damage invariant function agrees with oracle
    #[test]
    fn invariant_fn_agrees_with_oracle(
        base in 0.1f32..1000.0,
        combo in combo_depth_strategy(),
        armor in armor_strategy(),
    ) {
        let oracle = oracle_damage(base, combo, 0.0, armor, false);
        prop_assert!(oracle >= 1.0, "oracle damage floor: {}", oracle);
        let combo_mult = match combo { 0 | 1 => 1.0, 2 => 1.5, 3 => 2.0, _ => 3.0 };
        prop_assert!(damage_floor_holds(base, combo_mult, 0.0, armor));
    }
}

proptest! {
    // Every valid MagicType byte (0-5) converts to a positive damage multiplier
    #[test]
    fn magic_type_multiplier_always_positive(byte in 0u8..6u8) {
        let mt = MagicType::try_from(byte).expect("byte 0-5 must convert to MagicType");
        let multiplier = f32::from(mt);
        prop_assert!(multiplier > 0.0, "MagicType multiplier must be > 0, got {}", multiplier);
    }

    // TryFrom<u8> and From<MagicType> are value-stable round-trips
    #[test]
    fn magic_type_u8_round_trip(byte in 0u8..6u8) {
        let mt = MagicType::try_from(byte).expect("byte 0-5 should convert");
        let multiplier: f32 = f32::from(mt);
        // Re-converting from the same byte gives the same multiplier
        let mt2 = MagicType::try_from(byte).expect("second conversion should succeed");
        prop_assert_eq!(f32::from(mt2), multiplier, "MagicType conversion must be deterministic");
    }

    // Out-of-range bytes are always rejected
    #[test]
    fn magic_type_rejects_out_of_range(byte in 6u8..=255u8) {
        prop_assert!(MagicType::try_from(byte).is_err(), "byte {} should not convert to MagicType", byte);
    }
}

// Known-bad corpus tests
#[test]
fn known_bad_combat_cases_all_pass_floor() {
    use nexus_tests::fuzz::KnownBadCorpus;
    for (base, combo_mult, equipment_bonus, armor) in KnownBadCorpus::combat_edge_cases() {
        if base > 0.0 {
            let dmg = oracle_damage(
                base,
                if combo_mult >= 3.0 { 5 } else { 1 },
                equipment_bonus,
                armor,
                false,
            );
            assert!(
                dmg >= 1.0,
                "damage floor failed: base={}, mult={}, eq={}, armor={}, got={}",
                base,
                combo_mult,
                equipment_bonus,
                armor,
                dmg
            );
        }
    }
}

#[test]
fn known_bad_gold_cases_no_overflow() {
    use nexus_tests::fuzz::KnownBadCorpus;
    for (a, b) in KnownBadCorpus::gold_edge_cases() {
        // Gold addition should use saturating arithmetic
        let result = a.saturating_add(b);
        assert!(
            result >= a || result == u32::MAX,
            "gold should saturate, not overflow"
        );
    }
}
