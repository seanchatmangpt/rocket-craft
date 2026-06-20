use ib4_integration_tests::{new_session, AttackDir, Command, GameSession};

/// Clear the arena queue and place the GodKing as the only enemy, then spawn it.
/// Note: Command::Explore calls cmd_look(), not spawn. The spawn happens on Attack
/// when not already in combat. So we clear the queue, push GodKing, then Attack.
fn spawn_godking(s: &mut GameSession) {
    s.arena.clear();
    s.arena.push_back("CorruptedGalath".to_string());
    // Attack when not in combat triggers spawn_next_enemy
    s.dispatch(Command::Attack(AttackDir::Overhead));
}

#[test]
fn should_spawn_godking_with_shield_active() {
    let mut s = new_session();
    // Clear queue so only GodKing spawns
    s.arena.clear();
    s.arena.push_back("CorruptedGalath".to_string());
    s.dispatch(Command::Attack(AttackDir::Overhead));

    assert!(s.current_enemy.is_some(), "GodKing should be spawned");
    let enemy = s.current_enemy.as_ref().unwrap();
    assert_eq!(enemy.id, "CorruptedGalath");
    assert!(enemy.shield_active, "Shield should be active at Phase 1");
    assert_eq!(enemy.phase, 1);
}

#[test]
fn should_block_normal_attacks_with_godking_shield() {
    let mut s = new_session();
    spawn_godking(&mut s);
    // The spawn attack may have hit the shield already — reset state for clean test
    // Ensure shield is still active
    if let Some(e) = s.current_enemy.as_mut() {
        e.shield_active = true;
    }

    let hp_before = s
        .current_enemy
        .as_ref()
        .map(|e| e.current_hp)
        .unwrap_or(0.0);
    s.announced_attack = None;
    s.dispatch(Command::Attack(AttackDir::Overhead));
    let hp_after = s
        .current_enemy
        .as_ref()
        .map(|e| e.current_hp)
        .unwrap_or(0.0);

    assert_eq!(
        hp_before, hp_after,
        "Normal attack should deal 0 damage through hard-light shield"
    );
}

#[test]
fn should_break_godking_shield_after_3_perfect_parries() {
    let mut s = new_session();
    spawn_godking(&mut s);

    // Ensure shield is active
    if let Some(e) = s.current_enemy.as_mut() {
        e.shield_active = true;
        e.perfect_parries_received = 0;
    }
    assert!(s.current_enemy.as_ref().unwrap().shield_active);

    for _ in 0..3 {
        // Ensure there's an announced attack to parry
        if s.announced_attack.is_none() {
            s.announced_attack = Some(AttackDir::Left);
        }
        let dir = s.announced_attack.clone().unwrap();
        s.dispatch(Command::PerfectParry(dir));
    }

    let shield_active = s
        .current_enemy
        .as_ref()
        .map(|e| e.shield_active)
        .unwrap_or(false);
    assert!(
        !shield_active,
        "Shield should be broken after 3 perfect parries"
    );

    let phase = s.current_enemy.as_ref().map(|e| e.phase).unwrap_or(1);
    assert_eq!(
        phase, 2,
        "GodKing should advance to Phase 2 after shield breaks"
    );
}

#[test]
fn should_apply_qip_scar_stack_on_normal_parry_in_phase2() {
    let mut s = new_session();
    spawn_godking(&mut s);

    // Force Phase 2 directly (shield broken, phase=2)
    if let Some(e) = s.current_enemy.as_mut() {
        e.shield_active = false;
        e.phase = 2;
        e.perfect_parries_received = 3;
    }

    // Normal parry (no direction match) in Phase 2 should apply QIP scar
    let scars_before = s.player.qip_scar_stacks;
    s.announced_attack = Some(AttackDir::Right);
    s.dispatch(Command::Parry); // Normal parry — no directional match
    let scars_after = s.player.qip_scar_stacks;

    // QIP scars accumulate in Phase 2 on normal parries (or forced rebirth at 3 stacks)
    assert!(
        scars_after > scars_before || s.player.bloodline > 0,
        "QIP scar should be applied or forced rebirth triggered"
    );
}

#[test]
fn should_transition_godking_to_phase3_at_30_percent_hp() {
    let mut s = new_session();
    spawn_godking(&mut s);

    // Force Phase 2 (shield broken)
    if let Some(e) = s.current_enemy.as_mut() {
        e.shield_active = false;
        e.phase = 2;
        e.perfect_parries_received = 3;
        // Set HP to 29% to trigger Phase 3 on next attack
        e.current_hp = e.base_hp * 0.29;
    }

    s.announced_attack = None;
    s.dispatch(Command::Attack(AttackDir::Overhead));

    let phase = s.current_enemy.as_ref().map(|e| e.phase).unwrap_or(0);
    assert!(
        phase >= 3 || s.current_enemy.is_none(),
        "Phase 3 should trigger at 29% HP"
    );
}
