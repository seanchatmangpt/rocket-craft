/// Chicago-TDD style integration tests for the 11 wired TRACKED_WORKs in session.rs.
///
/// Each test uses `chicago_tdd_tools::TestEnvironment::new()` for isolation
/// (temporary directory scope), even when the test itself doesn't write files,
/// to satisfy the requirement and to guard against any future side-effects
/// (e.g., save files written by `Command::Save`).
use chicago_tdd_tools::TestEnvironment;
use ib4_integration_tests::{new_session, Command, AttackDir};
use ib4_core::types::{MagicType, Stat};
use proptest::prelude::*;
use ib4_ai::roster::spawn_enemy;
use ib4_combat::{
    damage::calc_player_damage,
    parry::{ParryResolver, ParryIntent, ParryOutcome},
    magic::resolve_magic,
};
use ib4_progression::{
    perks::PerkTree,
    xp::XPSystem,
};

// ── Test 1: Enemy spawn produces correct tier ──────────────────────────────

#[test]
fn enemy_spawn_produces_correct_hp_for_bloodline() {
    let _env = TestEnvironment::new().expect("test env");

    // Bloodline 0: LightTitan base HP = 150; scale = 1.0 + 0 * 0.15 = 1.0
    let enemy = spawn_enemy("LightTitan", 0).expect("should spawn LightTitan");
    assert_eq!(enemy.id, "LightTitan");
    assert!((enemy.base_hp - 150.0).abs() < 0.01, "BL0 HP should be 150, got {}", enemy.base_hp);
    assert_eq!(enemy.phase, 1, "Freshly spawned enemy starts at phase 1");

    // Bloodline 5: scale = 1.0 + 5 * 0.15 = 1.75 → HP = 262.5
    let enemy_bl5 = spawn_enemy("LightTitan", 5).expect("should spawn at BL5");
    let expected_hp = 150.0 * 1.75;
    assert!(
        (enemy_bl5.base_hp - expected_hp).abs() < 0.01,
        "BL5 LightTitan HP should be {}, got {}",
        expected_hp, enemy_bl5.base_hp
    );
}

#[test]
fn enemy_spawn_returns_none_for_unknown_id() {
    let _env = TestEnvironment::new().expect("test env");
    let result = spawn_enemy("FakeEnemy", 0);
    assert!(result.is_none(), "Unknown enemy ID should return None");
}

#[test]
fn session_spawn_next_enemy_uses_roster() {
    let _env = TestEnvironment::new().expect("test env");
    let mut s = new_session();

    // Attack when not in combat triggers spawn_next_enemy which calls roster::spawn_enemy
    let out = s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some(), "Enemy should be spawned after first attack");
    assert!(s.is_in_combat(), "Should be in combat after spawning");

    let enemy = s.current_enemy.as_ref().unwrap();
    // LightTitan is first in the default queue
    assert_eq!(enemy.id, "LightTitan", "First enemy should be LightTitan");
    // HP should be roster-scaled (bloodline 0 → exactly 150)
    assert!((enemy.base_hp - 150.0).abs() < 0.01, "HP should match roster value");
    let _ = out;
}

// ── Test 2: Player damage calculation uses attack stats correctly ───────────

#[test]
fn calc_player_damage_scales_with_attack_stat() {
    let _env = TestEnvironment::new().expect("test env");

    let mut player = ib4_core::player::PlayerState::new("Siris");
    let enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");

    let (dmg_base, _) = calc_player_damage(&player, &enemy, 1.0, 1.0, 0.0, 1.0); // rng=1.0 → no crit

    // Increase attack stat and verify damage increases
    player.stat_attack += 10;
    let (dmg_higher, _) = calc_player_damage(&player, &enemy, 1.0, 1.0, 0.0, 1.0);

    assert!(
        dmg_higher > dmg_base,
        "Higher attack stat should produce more damage: base={}, higher={}",
        dmg_base, dmg_higher
    );
}

#[test]
fn calc_player_damage_crits_when_rng_below_crit_chance() {
    let _env = TestEnvironment::new().expect("test env");

    let player = ib4_core::player::PlayerState::new("Siris");
    let enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");

    // rng_crit = 0.0 → always crits (default weapon crit_chance = 0.05)
    let (dmg_crit, is_crit) = calc_player_damage(&player, &enemy, 1.0, 1.0, 0.0, 0.0);
    assert!(is_crit, "Should be a crit when rng_crit=0.0");

    // rng_crit = 1.0 → never crits
    let (dmg_no_crit, not_crit) = calc_player_damage(&player, &enemy, 1.0, 1.0, 0.0, 1.0);
    assert!(!not_crit, "Should not crit when rng_crit=1.0");

    assert!(
        dmg_crit > dmg_no_crit,
        "Crit damage should be higher: crit={}, no_crit={}",
        dmg_crit, dmg_no_crit
    );
}

#[test]
fn calc_player_damage_combo_multiplier_increases_damage() {
    let _env = TestEnvironment::new().expect("test env");

    let player = ib4_core::player::PlayerState::new("Siris");
    let enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");

    let (dmg_1x, _) = calc_player_damage(&player, &enemy, 1.0, 1.0, 0.0, 1.0);
    let (dmg_2x, _) = calc_player_damage(&player, &enemy, 2.0, 1.0, 0.0, 1.0);

    assert!(
        (dmg_2x - dmg_1x * 2.0).abs() < 0.5,
        "2x combo multiplier should double damage: 1x={}, 2x={}",
        dmg_1x, dmg_2x
    );
}

// ── Test 3: Titan phase transitions at correct HP thresholds ───────────────

#[test]
fn titan_check_phase_transition_at_60_percent() {
    let _env = TestEnvironment::new().expect("test env");

    let mut enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");
    // Set HP to 59% to cross the 60% Phase 2 threshold
    enemy.current_hp = enemy.base_hp * 0.59;

    let result = ib4_ai::titan::TitanAI::check_phase_transition(&mut enemy);
    assert_eq!(result, Some(2), "Phase 2 should trigger at 59% HP");
    assert_eq!(enemy.phase, 2, "Enemy phase should be updated to 2");
    // Attack damage should scale to 1.25x
    let expected_atk = enemy.base_attack_damage * 1.25;
    assert!(
        (enemy.attack_damage - expected_atk).abs() < 0.01,
        "Phase 2 attack damage should be 1.25x: expected={}, got={}",
        expected_atk, enemy.attack_damage
    );
}

#[test]
fn titan_check_phase_transition_at_30_percent() {
    let _env = TestEnvironment::new().expect("test env");

    let mut enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");
    enemy.current_hp = enemy.base_hp * 0.29;

    let result = ib4_ai::titan::TitanAI::check_phase_transition(&mut enemy);
    assert_eq!(result, Some(3), "Phase 3 should trigger at 29% HP");
    assert_eq!(enemy.phase, 3, "Enemy phase should be updated to 3");
    let expected_atk = enemy.base_attack_damage * 1.875;
    assert!(
        (enemy.attack_damage - expected_atk).abs() < 0.01,
        "Phase 3 attack damage should be 1.875x"
    );
}

#[test]
fn titan_check_phase_transition_no_change_above_60_percent() {
    let _env = TestEnvironment::new().expect("test env");

    let mut enemy = spawn_enemy("LightTitan", 0).expect("spawn enemy");
    enemy.current_hp = enemy.base_hp * 0.80; // 80% HP → phase 1, no transition

    let result = ib4_ai::titan::TitanAI::check_phase_transition(&mut enemy);
    assert_eq!(result, None, "No transition should occur above 60% HP");
    assert_eq!(enemy.phase, 1, "Phase should remain 1");
}

// ── Test 4: Parry resolution returns the right outcome type ───────────────

#[test]
fn parry_resolver_directional_correct_direction_is_perfect() {
    let _env = TestEnvironment::new().expect("test env");

    let outcome = ParryResolver::resolve(
        AttackDir::Left,
        ParryIntent::DirectionalParry(AttackDir::Left),
    );
    assert_eq!(outcome, ParryOutcome::PerfectParry, "Matching direction should be PerfectParry");
}

#[test]
fn parry_resolver_directional_wrong_direction_is_normal() {
    let _env = TestEnvironment::new().expect("test env");

    let outcome = ParryResolver::resolve(
        AttackDir::Left,
        ParryIntent::DirectionalParry(AttackDir::Right),
    );
    assert_eq!(outcome, ParryOutcome::NormalParry, "Non-matching direction should be NormalParry");
}

#[test]
fn parry_resolver_any_parry_is_normal() {
    let _env = TestEnvironment::new().expect("test env");

    let outcome = ParryResolver::resolve(AttackDir::Overhead, ParryIntent::AnyParry);
    assert_eq!(outcome, ParryOutcome::NormalParry, "AnyParry intent should produce NormalParry");
}

#[test]
fn session_parry_command_prevents_damage() {
    let _env = TestEnvironment::new().expect("test env");

    let mut s = new_session();
    s.dispatch(Command::Attack(AttackDir::Overhead)); // spawn enemy
    assert!(s.current_enemy.is_some());

    let hp_before = s.player.health;
    s.announced_attack = Some(AttackDir::Left);
    s.dispatch(Command::Parry); // AnyParry → NormalParry → no damage
    assert_eq!(s.player.health, hp_before, "Normal parry should prevent all damage");
}

#[test]
fn session_perfect_parry_stuns_enemy() {
    let _env = TestEnvironment::new().expect("test env");

    let mut s = new_session();
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    // Perfect parry stuns the enemy; announce_next_attack then immediately
    // consumes the 1-turn stun (ticks it down to 0, clears is_stunned).
    // So after dispatch returns the stun is spent — verify instead that
    // the narrative mentions the stun and no further attack is announced
    // (enemy was "stunned and cannot act" during announce_next_attack).
    s.announced_attack = Some(AttackDir::Right);
    let _out = s.dispatch(Command::PerfectParry(AttackDir::Right));

    // The announce_next_attack path prints the stun message, not a new attack dir
    assert!(
        s.announced_attack.is_none(),
        "Perfect parry should stun enemy, preventing a new attack from being announced"
    );
    // After the 1-turn stun is consumed, the enemy is no longer stunned
    let is_stunned = s.current_enemy.as_ref().map(|e| e.is_stunned).unwrap_or(false);
    assert!(!is_stunned, "1-turn stun should be consumed within the same dispatch call");
}

// ── Test 5: Magic resolve costs mana and deals damage ─────────────────────

#[test]
fn resolve_magic_fire_has_correct_damage_and_mana_cost() {
    let _env = TestEnvironment::new().expect("test env");

    let (result, mana_cost) = resolve_magic(MagicType::Fire, 0, 1.0, 1.0);
    // magic_stat=0 → magic_bonus = 0 × 10 × 1.0 = 0; damage = 30 + 0 = 30
    assert!((result.damage - 30.0).abs() < 0.01, "Fire damage should be 30 with stat=0, got {}", result.damage);
    assert!(!result.is_heal, "Fire should not be a heal");
    assert!((mana_cost - 20.0).abs() < 0.01, "Fire mana cost should be 20, got {}", mana_cost);
}

#[test]
fn resolve_magic_light_is_a_heal() {
    let _env = TestEnvironment::new().expect("test env");

    let (result, mana_cost) = resolve_magic(MagicType::Light, 0, 1.0, 1.0);
    assert!(result.is_heal, "Light magic should be a heal");
    assert!((result.heal_amount - 40.0).abs() < 0.01, "Light heal amount should be 40 with stat=0, got {}", result.heal_amount);
    assert_eq!(result.damage, 0.0, "Light magic should deal 0 damage");
    assert!((mana_cost - 25.0).abs() < 0.01, "Light mana cost should be 25");
}

#[test]
fn resolve_magic_magic_stat_scales_damage() {
    let _env = TestEnvironment::new().expect("test env");

    let (result_0, _) = resolve_magic(MagicType::Fire, 0, 1.0, 1.0);
    let (result_5, _) = resolve_magic(MagicType::Fire, 5, 1.0, 1.0);
    // stat=5 → bonus = 5 * 10 * 1.0 = 50; damage = 30 + 50 = 80
    assert!(result_5.damage > result_0.damage, "Higher magic stat should increase damage");
    assert!((result_5.damage - 80.0).abs() < 0.01, "Expected 80 damage with stat=5, got {}", result_5.damage);
}

#[test]
fn resolve_magic_mana_cost_mult_reduces_cost() {
    let _env = TestEnvironment::new().expect("test env");

    let (_, cost_full) = resolve_magic(MagicType::Fire, 0, 1.0, 1.0);
    let (_, cost_half) = resolve_magic(MagicType::Fire, 0, 1.0, 0.5);
    assert!(cost_half < cost_full, "0.5 cost mult should reduce mana cost");
    assert!((cost_half - 10.0).abs() < 0.01, "Half cost should be 10, got {}", cost_half);
}

#[test]
fn session_magic_command_reduces_mana() {
    let _env = TestEnvironment::new().expect("test env");

    let mut s = new_session();
    // Spawn enemy first
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    let mana_before = s.player.mana;
    s.announced_attack = Some(AttackDir::Left); // so we don't skip the combat check
    s.dispatch(Command::Magic(MagicType::Fire));

    assert!(s.player.mana < mana_before, "Magic should consume mana");
}

// ── Test 6: Perk bonuses apply to player stats ────────────────────────────

#[test]
fn perk_tree_no_perks_gives_default_aggregate() {
    let _env = TestEnvironment::new().expect("test env");

    let tree = PerkTree::new();
    let agg = tree.compute_aggregate(&[]);
    assert!((agg.attack_mult - 1.0).abs() < 0.001, "No perks: attack_mult = 1.0");
    assert!((agg.gold_mult - 1.0).abs() < 0.001, "No perks: gold_mult = 1.0");
    assert!((agg.xp_mult - 1.0).abs() < 0.001, "No perks: xp_mult = 1.0");
    assert_eq!(agg.combo_extra_turns, 0, "No perks: no extra combo turns");
    assert!(!agg.has_parry_bonus, "No perks: no parry bonus");
}

#[test]
fn perk_tree_bloody_resolve_adds_attack_bonus() {
    let _env = TestEnvironment::new().expect("test env");

    let tree = PerkTree::new();
    let agg = tree.compute_aggregate(&["BloodyResolve".to_string()]);
    // BloodyResolve: +10% attack
    assert!(
        (agg.attack_mult - 1.10).abs() < 0.001,
        "BloodyResolve should give 1.10x attack, got {}",
        agg.attack_mult
    );
}

#[test]
fn perk_tree_scavenger_increases_gold_mult() {
    let _env = TestEnvironment::new().expect("test env");

    let tree = PerkTree::new();
    let agg = tree.compute_aggregate(&["Scavenger".to_string()]);
    // Scavenger: +20% gold
    assert!(
        (agg.gold_mult - 1.20).abs() < 0.001,
        "Scavenger should give 1.20x gold, got {}",
        agg.gold_mult
    );
}

#[test]
fn perk_tree_treasure_hunter_stacks_on_scavenger() {
    let _env = TestEnvironment::new().expect("test env");

    let tree = PerkTree::new();
    let perks = vec!["Scavenger".to_string(), "TreasureHunter".to_string()];
    let agg = tree.compute_aggregate(&perks);
    // Scavenger (+0.20) + TreasureHunter (+0.30) = 1.50 total
    assert!(
        (agg.gold_mult - 1.50).abs() < 0.001,
        "Scavenger+TreasureHunter should give 1.50x gold, got {}",
        agg.gold_mult
    );
}

#[test]
fn perk_tree_arcane_channeling_reduces_mana_cost() {
    let _env = TestEnvironment::new().expect("test env");

    let tree = PerkTree::new();
    let agg = tree.compute_aggregate(&["MagicSensitivity".to_string(), "ArcaneChanneling".to_string()]);
    // ArcaneChanneling: -20% mana cost → magic_cost_mult = 1.0 - 0.20 = 0.80
    assert!(
        (agg.magic_cost_mult - 0.80).abs() < 0.001,
        "ArcaneChanneling should give 0.80x mana cost, got {}",
        agg.magic_cost_mult
    );
}

// ── Test 7: XP check_level_up fires at correct XP threshold ───────────────

#[test]
fn xp_system_no_level_up_below_threshold() {
    let _env = TestEnvironment::new().expect("test env");

    let mut player = ib4_core::player::PlayerState::new("Siris");
    // Level 1 threshold = round(100 × 1^1.5) = 100 XP
    player.xp = 99;
    let events = XPSystem::add_xp(&mut player, 0, 1.0);
    assert!(events.is_empty(), "No level-up below threshold (XP=99)");
    assert_eq!(player.level, 1, "Player should remain level 1");
}

#[test]
fn xp_system_level_up_at_threshold() {
    let _env = TestEnvironment::new().expect("test env");

    let mut player = ib4_core::player::PlayerState::new("Siris");
    // Level 1 threshold = 100 XP; add exactly 100 raw XP with mult=1.0
    let events = XPSystem::add_xp(&mut player, 100, 1.0);
    assert!(!events.is_empty(), "Should level up at exactly 100 XP");
    assert_eq!(events[0].new_level, 2, "Should reach level 2");
    assert_eq!(events[0].stat_points_gained, 2, "Should gain 2 stat points");
    assert_eq!(player.level, 2, "Player level should be 2");
    assert_eq!(player.stat_points, 2, "Player should have 2 stat points");
}

#[test]
fn xp_system_xp_multiplier_scales_gained_xp() {
    let _env = TestEnvironment::new().expect("test env");

    let mut player = ib4_core::player::PlayerState::new("Siris");
    // Add 60 raw XP with 1.5x mult → 90 XP gained; not enough to level at threshold=100
    let events = XPSystem::add_xp(&mut player, 60, 1.5);
    assert!(events.is_empty(), "60 × 1.5 = 90 XP should not trigger level-up");
    assert_eq!(player.xp, 90, "Player XP should be 90");

    // Add 7 more raw XP with 1.5x → 10.5 → rounded to 11; total 101 → level up
    let events2 = XPSystem::add_xp(&mut player, 7, 1.5);
    assert!(!events2.is_empty(), "90 + 11 = 101 XP should trigger level-up");
    assert_eq!(events2[0].new_level, 2);
}

#[test]
fn xp_system_multiple_level_ups_in_one_call() {
    let _env = TestEnvironment::new().expect("test env");

    let mut player = ib4_core::player::PlayerState::new("Siris");
    // Add enough XP to hit levels 2 and 3 in one shot.
    // Level 1 threshold = 100, Level 2 threshold = round(100×2^1.5) = round(282.84) = 283
    let events = XPSystem::add_xp(&mut player, 300, 1.0);
    assert!(events.len() >= 2, "Should gain at least 2 levels with 300 XP, got {} events", events.len());
    assert!(player.level >= 3, "Player should be at least level 3");
}

#[test]
fn session_enemy_defeat_awards_xp_and_triggers_level_up() {
    let _env = TestEnvironment::new().expect("test env");

    let mut s = new_session();
    // Spawn LightTitan
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    // Give enough XP that the next enemy defeat will push us over the level threshold
    // Level 1 threshold = 100; LightTitan gives 50 XP; set player XP to 51 first
    s.player.xp = 51;

    // Kill the enemy (clearing announced_attack to avoid player death)
    let mut rounds = 0;
    while s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
        s.announced_attack = None;
        s.dispatch(Command::Attack(AttackDir::Overhead));
        rounds += 1;
        assert!(rounds < 100, "Should resolve combat in < 100 rounds");
    }

    // 51 + 50 (LightTitan XP) = 101 ≥ 100 → level up
    assert!(s.player.level >= 2, "Player should have levelled up after defeat: level={}", s.player.level);
    assert!(s.player.xp >= 100, "Player should have XP >= threshold");
}

// ---------------------------------------------------------------------------
// Property-based tests: stat parse invariants
// ---------------------------------------------------------------------------

proptest! {
    // Every canonical stat name parses successfully
    #[test]
    fn stat_canonical_names_parse(idx in 0usize..4usize) {
        let names = ["health", "attack", "defense", "magic"];
        let result: Result<Stat, _> = names[idx].parse();
        prop_assert!(result.is_ok(), "canonical stat name '{}' should parse", names[idx]);
    }

    // Every short alias also parses successfully
    #[test]
    fn stat_aliases_parse(idx in 0usize..4usize) {
        let aliases = ["hp", "atk", "def", "mag"];
        let result: Result<Stat, _> = aliases[idx].parse();
        prop_assert!(result.is_ok(), "stat alias '{}' should parse", aliases[idx]);
    }

    // Case-insensitive: uppercase canonical names parse to the same variant
    #[test]
    fn stat_uppercase_parses(idx in 0usize..4usize) {
        let names = ["HEALTH", "ATTACK", "DEFENSE", "MAGIC"];
        let lc_names = ["health", "attack", "defense", "magic"];
        let upper: Result<Stat, _> = names[idx].parse();
        let lower: Result<Stat, _> = lc_names[idx].parse();
        prop_assert!(upper.is_ok(), "uppercase stat '{}' should parse", names[idx]);
        prop_assert_eq!(upper.unwrap(), lower.unwrap(), "case should not affect parsed variant");
    }

    // Damage from an attack never increases player HP (attack always deals damage)
    #[test]
    fn attack_never_heals_enemy(initial_hp in 500.0f32..2000.0f32) {
        let mut s = new_session();
        // Spawn enemy with known high HP by dispatching first attack
        s.dispatch(Command::Attack(AttackDir::Overhead));
        if let Some(enemy) = s.current_enemy.as_mut() {
            enemy.current_hp = initial_hp;
        }
        let hp_before = s.current_enemy.as_ref().map(|e| e.current_hp).unwrap_or(0.0);
        s.announced_attack = None;
        s.dispatch(Command::Attack(AttackDir::Overhead));
        // After an attack, enemy HP should be <= before (never increases)
        let hp_after = s.current_enemy.as_ref().map(|e| e.current_hp).unwrap_or(0.0);
        prop_assert!(hp_after <= hp_before, "attack must not increase enemy HP: before={}, after={}", hp_before, hp_after);
    }
}
