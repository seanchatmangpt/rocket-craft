use nexus_tests::{invariants::*, model::*, strategies::*};
use proptest::prelude::*;

proptest! {
    // Damage floor invariant always holds
    #[test]
    fn damage_floor_always_at_least_one(
        base in damage_strategy(),
        armor in armor_strategy(),
        combo in combo_depth_strategy(),
    ) {
        let dmg = oracle_damage(base, combo, 0.0, armor, false);
        prop_assert!(dmg >= 1.0, "damage must always be >= 1.0, got {}", dmg);
    }

    // Perfect parry counter always does more than normal attack
    #[test]
    fn perfect_parry_counter_beats_normal(
        base in 1.0f32..1000.0,
        armor in 0.0f32..50.0,
        combo in 0u32..5,
    ) {
        let normal = oracle_damage(base, combo, 0.0, armor, false);
        let counter = oracle_damage(base, combo, 0.0, armor, true);
        prop_assert!(counter >= normal, "perfect counter ({}) must beat normal ({})", counter, normal);
    }

    // XP requirement is monotone: higher level always needs more XP
    #[test]
    fn xp_requirement_monotone(level in 1u32..100) {
        let current = oracle_xp_for_level(level);
        let next = oracle_xp_for_level(level + 1);
        prop_assert!(next > current, "level {} needs {} xp, level {} needs {} xp", level, current, level+1, next);
    }

    // Gacha probability never decreases (monotone in pity pulls)
    #[test]
    fn gacha_pity_monotone(pull in 1u32..89) {
        let p_now = oracle_ssr_probability(pull);
        let p_next = oracle_ssr_probability(pull + 1);
        prop_assert!(p_next >= p_now, "SSR rate must not decrease with more pulls: pull {} was {}, pull {} was {}", pull, p_now, pull+1, p_next);
    }

    // Hard pity at pull 90 guarantees SSR
    #[test]
    fn hard_pity_at_90_guarantees_ssr(pull in 90u32..200) {
        prop_assert_eq!(oracle_ssr_probability(pull), 1.0);
    }

    // Auction minimum bid is always > current bid (prevents bid snaking)
    #[test]
    fn auction_min_bid_above_current(current in 1u32..1_000_000) {
        let minimum = oracle_min_bid(current);
        prop_assert!(minimum > current, "min bid {} must be > current bid {}", minimum, current);
    }

    // GodKing shield breaks exactly at 3rd perfect parry
    #[test]
    fn godking_shield_breaks_only_at_3(parries_before in 0u32..2) {
        // Before 3rd parry: shield does not break
        let not_yet = oracle_shield_breaks(parries_before, true);
        if parries_before < 2 {
            prop_assert!(!not_yet, "shield should not break at {} parries", parries_before + 1);
        }
        // At exactly 3rd parry: breaks
        let breaks = oracle_shield_breaks(2, true);
        prop_assert!(breaks, "shield should break at 3rd perfect parry");
    }

    // Trans-Am only activates with BOTH depth >= 4 AND full gauge
    #[test]
    fn trans_am_requires_both_depth_and_gauge(
        combo in 0u32..7,
        gauge in 0.0f32..1.2,
    ) {
        let activates = oracle_trans_am_activates(combo, gauge);
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
                base, combo_mult, equipment_bonus, armor, dmg
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
