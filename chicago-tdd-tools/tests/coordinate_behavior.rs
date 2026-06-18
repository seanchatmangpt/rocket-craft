use anyhow::Result;
use chicago_tdd_tools::coordinate::{
    GameCoordinateSystem,
    InfinityBladeCoordinateSystem,
    GundamSessionSimulation,
    SessionState,
    GundamMove,
    GundamCoordinateSystem,
};
use ib4_mud::session::GameSession;
use ib4_mud::command::Command;
use ib4_core::types::{AttackDir, MagicType};
use nexus_session::player::PlayerProfile;

#[test]
fn test_infinity_blade_coordinate_system_not_in_combat() {
    let system = InfinityBladeCoordinateSystem;
    let mut session = GameSession::new("Siris");
    
    // Set status/stat points to 0 to verify legal moves do not contain AllocStat
    session.player.stat_points = 0;
    
    // Initial state: bloodline=0, health=280/280, max_health=280, current_enemy=None, announced_attack=None, in_combat=false, combo=0
    let coord = system.state_to_coordinate(&session);
    assert_eq!(coord, "b0:Full:None:None:ep0:aNone:cF:cb0");
    
    let legal_moves = system.get_legal_moves(&session);
    assert!(legal_moves.contains(&Command::Explore));
    assert!(legal_moves.contains(&Command::Attack(AttackDir::Overhead)));
    assert_eq!(legal_moves.len(), 2);
    
    // Give player stat points and check legal moves
    session.player.stat_points = 3;
    let legal_moves_with_stats = system.get_legal_moves(&session);
    assert!(legal_moves_with_stats.contains(&Command::Explore));
    assert!(legal_moves_with_stats.contains(&Command::Attack(AttackDir::Overhead)));
    assert!(legal_moves_with_stats.contains(&Command::AllocStat("health".to_string())));
    assert!(legal_moves_with_stats.contains(&Command::AllocStat("attack".to_string())));
    assert_eq!(legal_moves_with_stats.len(), 4);
    
    // Test notation
    assert_eq!(system.move_to_notation(&Command::Explore), "explore");
    assert_eq!(system.move_to_notation(&Command::AllocStat("health".to_string())), "alloc:health");
}

#[test]
fn test_infinity_blade_coordinate_system_in_combat() -> Result<()> {
    let system = InfinityBladeCoordinateSystem;
    let mut session = GameSession::new("Siris");
    
    // Start combat by dispatching an attack command
    session = system.apply_move(&session, &Command::Attack(AttackDir::Right))?;
    
    // Verify combat state
    assert!(session.is_in_combat());
    assert!(session.current_enemy.is_some());
    
    let enemy = session.current_enemy.as_ref().unwrap();
    let expected_enemy_id = match enemy.id.as_str() {
        "LightTitan" => "LT",
        "HeavyTitan" => "HT",
        "DarkKnight" => "DK",
        "CorruptedGalath" => "CG",
        other => other,
    };
    
    let coord = system.state_to_coordinate(&session);
    // Format: b{bloodline}:{hp_class}:{enemy_id}:{enemy_hp_class}:{enemy_phase}:{announced_attack}:{in_combat}:{combo}
    // Player and enemy should start full
    let announced_attack = match &session.announced_attack {
        Some(AttackDir::Overhead) => "aO",
        Some(AttackDir::Left) => "aL",
        Some(AttackDir::Right) => "aR",
        None => "aNone",
    };
    let expected_coord = format!("b0:Full:{}:Full:ep1:{}:cT:cb0", expected_enemy_id, announced_attack);
    assert_eq!(coord, expected_coord);
    
    // Check legal moves in combat
    let legal_moves = system.get_legal_moves(&session);
    assert!(legal_moves.contains(&Command::Attack(AttackDir::Overhead)));
    assert!(legal_moves.contains(&Command::Attack(AttackDir::Left)));
    assert!(legal_moves.contains(&Command::Attack(AttackDir::Right)));
    
    // Since an attack is announced, we should have Parry, PerfectParry, Dodge
    if let Some(announced) = &session.announced_attack {
        assert!(legal_moves.contains(&Command::Parry));
        assert!(legal_moves.contains(&Command::PerfectParry(announced.clone())));
        assert!(legal_moves.contains(&Command::Dodge));
    }
    
    // Player starts with 60 mana, which is >= 25, so magic fire and magic light should be legal
    assert!(legal_moves.contains(&Command::Magic(MagicType::Fire)));
    assert!(legal_moves.contains(&Command::Magic(MagicType::Light)));
    
    // Test notations
    assert_eq!(system.move_to_notation(&Command::Attack(AttackDir::Left)), "attack:left");
    assert_eq!(system.move_to_notation(&Command::Parry), "parry");
    assert_eq!(system.move_to_notation(&Command::PerfectParry(AttackDir::Overhead)), "perfect_parry:overhead");
    assert_eq!(system.move_to_notation(&Command::Magic(MagicType::Fire)), "magic:fire");
    
    Ok(())
}

#[test]
fn test_gundam_coordinate_system_transitions() -> Result<()> {
    let system = GundamCoordinateSystem;
    let profile = PlayerProfile::new(101, "Heero".to_string());
    
    let mut sim = GundamSessionSimulation {
        state: SessionState::Connecting,
        profile,
        inventory: Vec::new(),
    };
    
    // 1. Initial Connecting state
    // s{state}:{match_id}:lv{level}:xp{xp}:i{inv_size}:g{gold}
    // Level: 1, inventory: empty (0), gold: 100
    assert_eq!(system.state_to_coordinate(&sim), "sC:m0:lv1:xp0:i0:g100");
    
    let legal_moves = system.get_legal_moves(&sim);
    assert_eq!(legal_moves, vec![
        GundamMove::Authenticate(true),
        GundamMove::Authenticate(false),
        GundamMove::Reject,
    ]);
    
    // Test notation
    assert_eq!(system.move_to_notation(&GundamMove::Authenticate(true)), "auth:true");
    assert_eq!(system.move_to_notation(&GundamMove::Reject), "reject");
    
    // Authenticate failed move should return error
    let fail_res = system.apply_move(&sim, &GundamMove::Authenticate(false));
    assert!(fail_res.is_err());
    
    // Reject move should transition to Disconnected
    let reject_sim = system.apply_move(&sim, &GundamMove::Reject)?;
    assert_eq!(reject_sim.state, SessionState::Disconnected);
    assert_eq!(system.state_to_coordinate(&reject_sim), "sD:m0:lv1:xp0:i0:g100");
    
    // 2. Authenticate successful move -> Authenticated state
    sim = system.apply_move(&sim, &GundamMove::Authenticate(true))?;
    assert_eq!(sim.state, SessionState::Authenticated);
    assert_eq!(system.state_to_coordinate(&sim), "sA:m0:lv1:xp0:i0:g100");
    assert_eq!(system.get_legal_moves(&sim), vec![GundamMove::EnterLobby, GundamMove::Disconnect]);
    
    // 3. EnterLobby -> InLobby state
    sim = system.apply_move(&sim, &GundamMove::EnterLobby)?;
    assert_eq!(sim.state, SessionState::InLobby);
    assert_eq!(system.state_to_coordinate(&sim), "sL:m0:lv1:xp0:i0:g100");
    
    // Legal moves in Lobby: EnterMatch, Spectate, Disconnect, ApplyXP, SpendGold (10 since gold=100), InventoryAdd
    let lobby_moves = system.get_legal_moves(&sim);
    assert!(lobby_moves.contains(&GundamMove::EnterMatch(42)));
    assert!(lobby_moves.contains(&GundamMove::Spectate(42)));
    assert!(lobby_moves.contains(&GundamMove::Disconnect));
    assert!(lobby_moves.contains(&GundamMove::ApplyXP(100)));
    assert!(lobby_moves.contains(&GundamMove::SpendGold(10)));
    assert!(lobby_moves.contains(&GundamMove::InventoryAdd));
    
    // 4. ApplyXP -> increases XP, checks level/coordinate
    sim = system.apply_move(&sim, &GundamMove::ApplyXP(400))?;
    // XP 400 is enough to level up (xp required for level 2 is 100 * 2^2 = 400)
    assert_eq!(sim.profile.level, 2);
    assert_eq!(system.state_to_coordinate(&sim), "sL:m0:lv2:xp400:i0:g100");
    
    // 5. SpendGold -> decreases gold
    sim = system.apply_move(&sim, &GundamMove::SpendGold(10))?;
    assert_eq!(sim.profile.gold, 90);
    assert_eq!(system.state_to_coordinate(&sim), "sL:m0:lv2:xp400:i0:g90");
    
    // 6. EnterMatch(42) -> InMatch { match_id: 42 }
    sim = system.apply_move(&sim, &GundamMove::EnterMatch(42))?;
    assert_eq!(sim.state, SessionState::InMatch { match_id: 42 });
    assert_eq!(system.state_to_coordinate(&sim), "sM:m42:lv2:xp400:i0:g90");
    assert_eq!(system.get_legal_moves(&sim), vec![GundamMove::MatchComplete, GundamMove::Disconnect]);
    
    // 7. MatchComplete -> InLobby
    sim = system.apply_move(&sim, &GundamMove::MatchComplete)?;
    assert_eq!(sim.state, SessionState::InLobby);
    
    // 8. Spectate(77) -> Spectating { match_id: 77 }
    sim = system.apply_move(&sim, &GundamMove::Spectate(77))?;
    assert_eq!(sim.state, SessionState::Spectating { match_id: 77 });
    assert_eq!(system.state_to_coordinate(&sim), "sS:m77:lv2:xp400:i0:g90");
    assert_eq!(system.get_legal_moves(&sim), vec![GundamMove::LeaveSpectate, GundamMove::Disconnect]);
    
    // 9. LeaveSpectate -> InLobby
    sim = system.apply_move(&sim, &GundamMove::LeaveSpectate)?;
    assert_eq!(sim.state, SessionState::InLobby);
    
    // 10. Disconnect -> Disconnected
    sim = system.apply_move(&sim, &GundamMove::Disconnect)?;
    assert_eq!(sim.state, SessionState::Disconnected);
    assert_eq!(system.state_to_coordinate(&sim), "sD:m0:lv2:xp400:i0:g90");
    assert_eq!(system.get_legal_moves(&sim), vec![GundamMove::Reconnect]);
    
    // 11. Reconnect -> Connecting
    sim = system.apply_move(&sim, &GundamMove::Reconnect)?;
    assert_eq!(sim.state, SessionState::Connecting);
    
    Ok(())
}
