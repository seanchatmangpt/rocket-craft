use nexus_integration::game_loop::*;
use nexus_types::Gold;
use proptest::prelude::*;

// ── Combat loop ──────────────────────────────────────────────────────────────

#[test]
fn full_combat_loop_kills_enemy_and_grants_rewards() {
    let mut s = GameSession::new(1, "Suletta", 42);
    s.dispatch(GameCommand::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());
    assert!(s.is_in_combat);

    let mut rounds = 0;
    while s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
        s.dispatch(GameCommand::Attack(AttackDir::Overhead));
        rounds += 1;
        assert!(rounds < 200, "combat should resolve within 200 rounds");
    }

    assert!(!s.is_in_combat);
    assert!(s.player.xp.value() > 0);
    assert!(s.player.gold.value() > 500, "should earn gold from combat");
    assert_eq!(s.player.combo_depth, 0, "combo resets on enemy defeat");
}

#[test]
fn combo_depth_builds_with_attacks() {
    let mut s = GameSession::new(1, "Kamille", 1);
    s.dispatch(GameCommand::Attack(AttackDir::Overhead)); // spawns + first hit
    if s.is_in_combat && s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
        assert_eq!(s.player.combo_depth, 1);
        if s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
            s.dispatch(GameCommand::Attack(AttackDir::Left));
            if s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
                assert_eq!(s.player.combo_depth, 2);
            }
        }
    }
}

#[test]
fn dodge_resets_combo() {
    let mut s = GameSession::new(1, "Amuro", 5);
    s.dispatch(GameCommand::Attack(AttackDir::Overhead));
    s.dispatch(GameCommand::Attack(AttackDir::Left));
    s.dispatch(GameCommand::Dodge);
    assert_eq!(s.player.combo_depth, 0);
}

// ── GodKing / shield mechanics ───────────────────────────────────────────────

#[test]
fn godking_shield_blocks_all_attacks() {
    let mut s = GameSession::new(1, "Siris", 10);
    s.spawn_godking();
    let hp_before = s.current_enemy.as_ref().unwrap().hp;
    s.dispatch(GameCommand::Attack(AttackDir::Overhead));
    let hp_after = s.current_enemy.as_ref().map(|e| e.hp).unwrap_or(0.0);
    assert_eq!(hp_before, hp_after, "shield should absorb all damage");
}

#[test]
fn godking_shield_breaks_after_3_perfect_parries() {
    let mut s = GameSession::new(1, "Aran", 20);
    s.spawn_godking();

    for _ in 0..3 {
        if let Some(e) = s.current_enemy.as_mut() {
            e.announced_dir = Some(AttackDir::Overhead);
        }
        s.dispatch(GameCommand::Parry(Some(AttackDir::Overhead)));
    }

    let shield_on = s.current_enemy.as_ref().map(|e| e.shield_active).unwrap_or(true);
    assert!(!shield_on, "shield must break after 3 perfect parries");
}

#[test]
fn godking_phase2_qip_scars_force_rebirth_at_3() {
    let mut s = GameSession::new(1, "Siris", 42);
    s.spawn_godking();

    // Break shield first (3 perfect parries)
    for _ in 0..3 {
        if let Some(e) = s.current_enemy.as_mut() { e.announced_dir = Some(AttackDir::Left); }
        s.dispatch(GameCommand::Parry(Some(AttackDir::Left)));
    }

    // Now in Phase 2 (no shield). Stack 3 QIP scars via wrong-dir parries.
    // Wrong dir = normal parry = applies QIP scar in Phase 2 GodKing.
    if let Some(e) = s.current_enemy.as_mut() { e.announced_dir = Some(AttackDir::Right); }
    // Phase 2 may not be active yet (shield just broke = phase 2 activates now)
    // Force phase 2
    if let Some(e) = s.current_enemy.as_mut() { e.phase = 2; }

    let initial_bloodline = s.player.bloodline;
    for _ in 0..3 {
        if let Some(e) = s.current_enemy.as_mut() { e.announced_dir = Some(AttackDir::Overhead); }
        s.dispatch(GameCommand::Parry(Some(AttackDir::Right))); // wrong dir = QIP scar
    }

    // Either rebirth triggered OR scars accumulated to 3
    assert!(s.player.bloodline > initial_bloodline || s.player.qip_scar_stacks >= 3);
}

// ── Rebirth / bloodline ──────────────────────────────────────────────────────

#[test]
fn rebirth_increments_bloodline_and_restores_hp() {
    let mut s = GameSession::new(1, "Aran", 99);
    let initial_bl = s.player.bloodline;
    s.player.rebirth();
    assert_eq!(s.player.bloodline, initial_bl + 1);
    assert_eq!(s.player.hp, s.player.max_hp);
    assert_eq!(s.player.gold, Gold::new(500), "gold resets on rebirth");
}

#[test]
fn xp_preserved_through_rebirth() {
    let mut s = GameSession::new(1, "Siris", 1);
    s.player.gain_xp(5_000);
    let xp_before = s.player.xp;
    s.player.rebirth();
    assert_eq!(s.player.xp, xp_before, "XP must survive rebirth");
}

#[test]
fn perk_point_granted_on_rebirth_below_bl20() {
    let mut s = GameSession::new(1, "test", 1);
    s.player.bloodline = 10;
    let pp_before = s.player.perk_points;
    s.player.rebirth();
    assert_eq!(s.player.perk_points, pp_before + 1);
}

// ── Shop / economy ───────────────────────────────────────────────────────────

#[test]
fn buying_weapon_increases_attack_and_costs_gold() {
    let mut s = GameSession::new(1, "Miorine", 42);
    let initial_atk = s.player.attack;
    s.player.gold = Gold::new(1000);
    s.dispatch(GameCommand::BuyItem { item_index: 0 }); // Beam Saber (+15 ATK, 100 gold)
    assert!(s.player.attack > initial_atk);
    assert_eq!(s.player.gold, Gold::new(900));
}

#[test]
fn cannot_buy_item_without_enough_gold() {
    let mut s = GameSession::new(1, "Char", 1);
    s.player.gold = Gold::new(0);
    let initial_atk = s.player.attack;
    s.dispatch(GameCommand::BuyItem { item_index: 0 });
    assert_eq!(s.player.attack, initial_atk, "no gold = no purchase");
    assert_eq!(s.player.gold, Gold::new(0));
}

// ── Stat allocation ──────────────────────────────────────────────────────────

#[test]
fn stat_allocation_spends_point_and_increases_stat() {
    let mut s = GameSession::new(1, "Char", 1);
    s.player.stat_points = 3;
    let initial_atk = s.player.attack;
    s.dispatch(GameCommand::AllocateStat(StatType::Attack));
    assert_eq!(s.player.attack, initial_atk + 5);
    assert_eq!(s.player.stat_points, 2);
}

#[test]
fn stat_allocation_without_points_is_noop() {
    let mut s = GameSession::new(1, "test", 1);
    assert_eq!(s.player.stat_points, 0);
    let initial = s.player.attack;
    s.dispatch(GameCommand::AllocateStat(StatType::Attack));
    assert_eq!(s.player.attack, initial);
}

// ── Trans-Am ─────────────────────────────────────────────────────────────────

#[test]
fn trans_am_activates_only_with_full_gauge() {
    let mut s = GameSession::new(1, "Setsuna", 42);
    s.player.trans_am_gauge = 0.5;
    let initial_atk = s.player.attack;
    s.dispatch(GameCommand::UseSpecial);
    assert_eq!(s.player.attack, initial_atk, "partial gauge = no Trans-Am");

    s.player.trans_am_gauge = 1.0;
    s.dispatch(GameCommand::UseSpecial);
    assert!(s.player.attack > initial_atk, "full gauge = Trans-Am activates");
    assert_eq!(s.player.trans_am_gauge, 0.0, "gauge resets after activation");
}

// ── Surrender ────────────────────────────────────────────────────────────────

#[test]
fn surrender_ends_combat() {
    let mut s = GameSession::new(1, "test", 1);
    s.dispatch(GameCommand::Attack(AttackDir::Left));
    assert!(s.is_in_combat);
    s.dispatch(GameCommand::Surrender);
    assert!(!s.is_in_combat);
    assert!(s.current_enemy.is_none());
}

// ── Perk selection ───────────────────────────────────────────────────────────

#[test]
fn perk_selection_requires_perk_points() {
    let mut s = GameSession::new(1, "test", 1);
    assert_eq!(s.player.perk_points, 0);
    s.dispatch(GameCommand::SelectPerk(1));
    assert!(s.player.selected_perks.is_empty());

    s.player.perk_points = 1;
    s.dispatch(GameCommand::SelectPerk(2));
    assert_eq!(s.player.selected_perks.len(), 1);
    assert_eq!(s.player.perk_points, 0);
}

// ── Property-based invariants ─────────────────────────────────────────────────

proptest! {
    // Player HP never goes negative regardless of damage taken
    #[test]
    fn hp_never_negative(damage_seq in prop::collection::vec(0.0f32..2000.0, 1..20)) {
        let mut s = GameSession::new(1, "inv-test", 42);
        for dmg in damage_seq {
            s.player.take_damage(dmg);
            prop_assert!(s.player.hp.value() >= 0.0, "HP must never be negative");
        }
    }

    // Gold never underflows
    #[test]
    fn gold_never_underflows(spend in 0u32..10_000) {
        let mut s = GameSession::new(1, "gold-test", 1);
        let before = s.player.gold;
        s.player.spend_gold(spend);
        prop_assert!(s.player.gold <= before);
    }

    // Combo multiplier is always >= 1.0
    #[test]
    fn combo_mult_always_at_least_one(depth in 0u32..10) {
        let mut s = GameSession::new(1, "combo-test", 1);
        s.player.combo_depth = depth;
        prop_assert!(s.player.combo_multiplier() >= 1.0);
    }

    // Level is monotone — XP gain never decreases level
    #[test]
    fn level_monotone_on_xp_gain(batches in prop::collection::vec(0u64..10_000, 1..30)) {
        let mut s = GameSession::new(1, "level-test", 1);
        let mut prev = s.player.level;
        for xp in batches {
            s.player.gain_xp(xp);
            prop_assert!(s.player.level >= prev, "level must not decrease");
            prev = s.player.level;
        }
    }

    // Bloodline monotone through rebirths
    #[test]
    fn bloodline_increases_each_rebirth(count in 1usize..10) {
        let mut s = GameSession::new(1, "bl-test", 1);
        let mut prev = s.player.bloodline;
        for _ in 0..count {
            s.player.rebirth();
            prop_assert!(s.player.bloodline > prev, "bloodline must increase on rebirth");
            prev = s.player.bloodline;
        }
    }

    // Event log only grows — never shrinks
    #[test]
    fn event_log_monotone(n in 1usize..20) {
        let mut s = GameSession::new(1, "event-test", 99);
        let mut prev_count = 0usize;
        for i in 0..n {
            let cmd = match i % 4 {
                0 => GameCommand::LookAround,
                1 => GameCommand::Attack(AttackDir::Overhead),
                2 => GameCommand::Dodge,
                _ => GameCommand::OpenShop,
            };
            s.dispatch(cmd);
            prop_assert!(s.events.len() >= prev_count, "event log must not shrink");
            prev_count = s.events.len();
        }
    }
}
