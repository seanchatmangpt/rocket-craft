use anyhow::Result;
use ib4_core::types::{AttackDir, MagicType};
use ib4_mud::command::Command;
use ib4_mud::session::GameSession;
use nexus_session::inventory::Item;
use nexus_session::player::PlayerProfile;

pub trait GameCoordinateSystem {
    type State;
    type Move: std::fmt::Debug + Clone;

    fn state_to_coordinate(&self, state: &Self::State) -> String;
    fn get_legal_moves(&self, state: &Self::State) -> Vec<Self::Move>;
    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> Result<Self::State>;
    fn move_to_notation(&self, mv: &Self::Move) -> String;
}

fn get_hp_class(hp: f32, max_hp: f32) -> &'static str {
    if hp <= 0.0 {
        "Dead"
    } else {
        let ratio = hp / max_hp;
        if ratio >= 1.0 {
            "Full"
        } else if ratio >= 0.25 {
            "Mid"
        } else {
            "Low"
        }
    }
}

fn map_enemy_id(id: &str) -> &str {
    match id {
        "LightTitan" => "LT",
        "HeavyTitan" => "HT",
        "DarkKnight" => "DK",
        "CorruptedGalath" => "CG",
        "MageTitan" => "MT",
        "GiantTitan" => "GT",
        "BloodSlave" => "BS",
        "KuroShino" => "KS",
        "DeathlessSoldier" => "DS",
        "ElementalTitan" => "ET",
        "ShadowTitan" => "ST",
        "TwinBladeTitan" => "TBT",
        "CrystalGolem" => "CrG",
        "QuantumSoldier" => "QS",
        "Kuero" => "K",
        other => other,
    }
}

pub struct InfinityBladeCoordinateSystem;

impl GameCoordinateSystem for InfinityBladeCoordinateSystem {
    type State = GameSession;
    type Move = Command;

    fn state_to_coordinate(&self, state: &Self::State) -> String {
        let bloodline = state.player.bloodline;
        let hp_class = get_hp_class(state.player.health, state.player.max_health);
        let enemy_id = state
            .current_enemy
            .as_ref()
            .map(|e| map_enemy_id(&e.id))
            .unwrap_or("None");
        let enemy_hp_class = state
            .current_enemy
            .as_ref()
            .map(|e| get_hp_class(e.current_hp, e.base_hp))
            .unwrap_or("None");
        let enemy_phase = state
            .current_enemy
            .as_ref()
            .map(|e| format!("ep{}", e.phase))
            .unwrap_or_else(|| "ep0".to_string());
        let announced_attack = match &state.announced_attack {
            Some(AttackDir::Overhead) => "aO",
            Some(AttackDir::Left) => "aL",
            Some(AttackDir::Right) => "aR",
            None => "aNone",
        };
        let in_combat = if state.is_in_combat() { "cT" } else { "cF" };
        let combo = format!("cb{}", state.combo_depth);

        format!(
            "b{}:{}:{}:{}:{}:{}:{}:{}",
            bloodline,
            hp_class,
            enemy_id,
            enemy_hp_class,
            enemy_phase,
            announced_attack,
            in_combat,
            combo
        )
    }

    fn get_legal_moves(&self, state: &Self::State) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        if !state.is_in_combat() {
            moves.push(Command::Explore);
            moves.push(Command::Attack(AttackDir::Overhead));
            if state.player.stat_points > 0 {
                moves.push(Command::AllocStat("health".to_string()));
                moves.push(Command::AllocStat("attack".to_string()));
            }
        } else {
            moves.push(Command::Attack(AttackDir::Overhead));
            moves.push(Command::Attack(AttackDir::Left));
            moves.push(Command::Attack(AttackDir::Right));
            if let Some(announced) = &state.announced_attack {
                moves.push(Command::Parry);
                moves.push(Command::PerfectParry(announced.clone()));
                moves.push(Command::Dodge);
            }
            if state.player.mana >= 20.0 {
                moves.push(Command::Magic(MagicType::Fire));
            }
            if state.player.mana >= 25.0 {
                moves.push(Command::Magic(MagicType::Light));
            }
        }
        moves
    }

    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> Result<Self::State> {
        let mut next_state = state.clone();
        let _narrative = next_state.dispatch(mv.clone());
        Ok(next_state)
    }

    fn move_to_notation(&self, mv: &Self::Move) -> String {
        match mv {
            Command::Explore => "explore".to_string(),
            Command::Attack(dir) => match dir {
                AttackDir::Overhead => "attack:overhead".to_string(),
                AttackDir::Left => "attack:left".to_string(),
                AttackDir::Right => "attack:right".to_string(),
            },
            Command::Parry => "parry".to_string(),
            Command::PerfectParry(dir) => match dir {
                AttackDir::Overhead => "perfect_parry:overhead".to_string(),
                AttackDir::Left => "perfect_parry:left".to_string(),
                AttackDir::Right => "perfect_parry:right".to_string(),
            },
            Command::Dodge => "dodge".to_string(),
            Command::Magic(magic) => match magic {
                MagicType::Fire => "magic:fire".to_string(),
                MagicType::Lightning => "magic:lightning".to_string(),
                MagicType::Ice => "magic:ice".to_string(),
                MagicType::Dark => "magic:dark".to_string(),
                MagicType::Light => "magic:light".to_string(),
            },
            Command::AllocStat(stat) => format!("alloc:{}", stat.to_lowercase()),
            Command::Look => "look".to_string(),
            Command::Status => "status".to_string(),
            Command::Inventory => "inventory".to_string(),
            Command::Perks => "perks".to_string(),
            Command::SelectPerk(perk) => format!("select_perk:{}", perk),
            Command::Shop => "shop".to_string(),
            Command::Buy(item) => format!("buy:{}", item),
            Command::Sell(item) => format!("sell:{}", item),
            Command::Equip(item) => format!("equip:{}", item),
            Command::Save => "save".to_string(),
            Command::Help => "help".to_string(),
            Command::Quit => "quit".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Connecting,
    Authenticated,
    InLobby,
    InMatch { match_id: u64 },
    Spectating { match_id: u64 },
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct MechaSessionSimulation {
    pub state: SessionState,
    pub profile: PlayerProfile,
    pub inventory: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechaMove {
    Authenticate(bool),
    Reject,
    EnterLobby,
    EnterMatch(u64),
    Spectate(u64),
    Disconnect,
    ApplyXP(u64),
    SpendGold(u32),
    MatchComplete,
    LeaveSpectate,
    Reconnect,
    InventoryAdd,
    InventoryRemove(usize),
}

pub struct MechaCoordinateSystem;

impl GameCoordinateSystem for MechaCoordinateSystem {
    type State = MechaSessionSimulation;
    type Move = MechaMove;

    fn state_to_coordinate(&self, state: &Self::State) -> String {
        let state_char = match &state.state {
            SessionState::Connecting => "C",
            SessionState::Authenticated => "A",
            SessionState::InLobby => "L",
            SessionState::InMatch { .. } => "M",
            SessionState::Spectating { .. } => "S",
            SessionState::Disconnected => "D",
        };
        let match_id_str = match &state.state {
            SessionState::InMatch { match_id } => format!("m{}", match_id),
            SessionState::Spectating { match_id } => format!("m{}", match_id),
            _ => "m0".to_string(),
        };
        format!(
            "s{}:{}:lv{}:xp{}:i{}:g{}",
            state_char,
            match_id_str,
            state.profile.level,
            state.profile.xp,
            state.inventory.len(),
            state.profile.gold
        )
    }

    fn get_legal_moves(&self, state: &Self::State) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        match &state.state {
            SessionState::Connecting => {
                moves.push(MechaMove::Authenticate(true));
                moves.push(MechaMove::Authenticate(false));
                moves.push(MechaMove::Reject);
            }
            SessionState::Authenticated => {
                moves.push(MechaMove::EnterLobby);
                moves.push(MechaMove::Disconnect);
            }
            SessionState::InLobby => {
                moves.push(MechaMove::EnterMatch(42));
                moves.push(MechaMove::Spectate(42));
                moves.push(MechaMove::Disconnect);
                moves.push(MechaMove::ApplyXP(100));
                if state.profile.gold >= 10 {
                    moves.push(MechaMove::SpendGold(10));
                } else if state.profile.gold > 0 {
                    moves.push(MechaMove::SpendGold(state.profile.gold));
                }
                if state.inventory.len() < 5 {
                    moves.push(MechaMove::InventoryAdd);
                }
                for i in 0..state.inventory.len() {
                    moves.push(MechaMove::InventoryRemove(i));
                }
            }
            SessionState::InMatch { .. } => {
                moves.push(MechaMove::MatchComplete);
                moves.push(MechaMove::Disconnect);
            }
            SessionState::Spectating { .. } => {
                moves.push(MechaMove::LeaveSpectate);
                moves.push(MechaMove::Disconnect);
            }
            SessionState::Disconnected => {
                moves.push(MechaMove::Reconnect);
            }
        }
        moves
    }

    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> Result<Self::State> {
        let mut next = state.clone();
        match (&state.state, mv) {
            (SessionState::Connecting, MechaMove::Authenticate(true)) => {
                next.state = SessionState::Authenticated;
            }
            (SessionState::Connecting, MechaMove::Authenticate(false)) => {
                return Err(anyhow::anyhow!("Authentication failed"));
            }
            (SessionState::Connecting, MechaMove::Reject) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Authenticated, MechaMove::EnterLobby) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::Authenticated, MechaMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::InLobby, MechaMove::EnterMatch(match_id)) => {
                next.state = SessionState::InMatch {
                    match_id: *match_id,
                };
            }
            (SessionState::InLobby, MechaMove::Spectate(match_id)) => {
                next.state = SessionState::Spectating {
                    match_id: *match_id,
                };
            }
            (SessionState::InLobby, MechaMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::InLobby, MechaMove::ApplyXP(amount)) => {
                next.profile.apply_xp_gain(*amount);
            }
            (SessionState::InLobby, MechaMove::SpendGold(amount)) => {
                next.profile
                    .spend_gold(*amount)
                    .map_err(|e| anyhow::anyhow!("Spend gold failed: {}", e))?;
            }
            (SessionState::InLobby, MechaMove::InventoryAdd) => {
                if next.inventory.len() < 5 {
                    let count = next.inventory.len() as u64;
                    next.inventory.push(Item {
                        id: count,
                        name: "Shield".to_string(),
                        ..Default::default()
                    });
                } else {
                    return Err(anyhow::anyhow!("Inventory full"));
                }
            }
            (SessionState::InLobby, MechaMove::InventoryRemove(slot)) => {
                if *slot < next.inventory.len() {
                    next.inventory.remove(*slot);
                } else {
                    return Err(anyhow::anyhow!("Invalid inventory slot"));
                }
            }
            (SessionState::InMatch { .. }, MechaMove::MatchComplete) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::InMatch { .. }, MechaMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Spectating { .. }, MechaMove::LeaveSpectate) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::Spectating { .. }, MechaMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Disconnected, MechaMove::Reconnect) => {
                next.state = SessionState::Connecting;
            }
            (current_state, invalid_move) => {
                return Err(anyhow::anyhow!(
                    "Invalid move {:?} in state {:?}",
                    invalid_move,
                    current_state
                ));
            }
        }
        Ok(next)
    }

    fn move_to_notation(&self, mv: &Self::Move) -> String {
        match mv {
            MechaMove::Authenticate(val) => format!("auth:{}", val),
            MechaMove::Reject => "reject".to_string(),
            MechaMove::EnterLobby => "enter_lobby".to_string(),
            MechaMove::EnterMatch(match_id) => format!("enter_match:{}", match_id),
            MechaMove::Spectate(match_id) => format!("spectate:{}", match_id),
            MechaMove::Disconnect => "disconnect".to_string(),
            MechaMove::ApplyXP(amount) => format!("apply_xp:{}", amount),
            MechaMove::SpendGold(amount) => format!("spend_gold:{}", amount),
            MechaMove::MatchComplete => "match_complete".to_string(),
            MechaMove::LeaveSpectate => "leave_spectate".to_string(),
            MechaMove::Reconnect => "reconnect".to_string(),
            MechaMove::InventoryAdd => "inventory_add".to_string(),
            MechaMove::InventoryRemove(slot) => format!("inventory_remove:{}", slot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_session::player::PlayerProfile;

    // ── get_hp_class (private, tested via state_to_coordinate indirectly) ──────
    // We test the observable behavior through MechaCoordinateSystem::state_to_coordinate.

    fn profile() -> PlayerProfile {
        PlayerProfile::new(1, "Amuro".into())
    }

    fn connecting_sim() -> MechaSessionSimulation {
        MechaSessionSimulation {
            state: SessionState::Connecting,
            profile: profile(),
            inventory: vec![],
        }
    }

    fn lobby_sim() -> MechaSessionSimulation {
        MechaSessionSimulation {
            state: SessionState::InLobby,
            profile: profile(),
            inventory: vec![],
        }
    }

    // ── MechaCoordinateSystem::state_to_coordinate ───────────────────────────

    #[test]
    fn connecting_coordinate_starts_with_sc() {
        let sys = MechaCoordinateSystem;
        let coord = sys.state_to_coordinate(&connecting_sim());
        assert!(coord.starts_with("sC:"), "got: {coord}");
    }

    #[test]
    fn lobby_coordinate_starts_with_sl() {
        let sys = MechaCoordinateSystem;
        let coord = sys.state_to_coordinate(&lobby_sim());
        assert!(coord.starts_with("sL:"), "got: {coord}");
    }

    #[test]
    fn in_match_coordinate_includes_match_id() {
        let sys = MechaCoordinateSystem;
        let sim = MechaSessionSimulation {
            state: SessionState::InMatch { match_id: 99 },
            profile: profile(),
            inventory: vec![],
        };
        let coord = sys.state_to_coordinate(&sim);
        assert!(coord.contains("m99"), "got: {coord}");
    }

    #[test]
    fn coordinate_includes_level_and_gold() {
        let sys = MechaCoordinateSystem;
        let coord = sys.state_to_coordinate(&lobby_sim());
        assert!(coord.contains("lv1"), "got: {coord}");
        assert!(coord.contains("g100"), "PlayerProfile starts with gold=100; got: {coord}");
    }

    // ── MechaCoordinateSystem::get_legal_moves ───────────────────────────────

    #[test]
    fn connecting_legal_moves_include_authenticate() {
        let sys = MechaCoordinateSystem;
        let moves = sys.get_legal_moves(&connecting_sim());
        assert!(moves.contains(&MechaMove::Authenticate(true)));
        assert!(moves.contains(&MechaMove::Authenticate(false)));
        assert!(moves.contains(&MechaMove::Reject));
    }

    #[test]
    fn lobby_legal_moves_include_enter_match_and_spectate() {
        let sys = MechaCoordinateSystem;
        let moves = sys.get_legal_moves(&lobby_sim());
        assert!(moves.contains(&MechaMove::EnterMatch(42)));
        assert!(moves.contains(&MechaMove::Spectate(42)));
        assert!(moves.contains(&MechaMove::Disconnect));
    }

    #[test]
    fn disconnected_legal_moves_only_reconnect() {
        let sys = MechaCoordinateSystem;
        let sim = MechaSessionSimulation {
            state: SessionState::Disconnected,
            profile: profile(),
            inventory: vec![],
        };
        let moves = sys.get_legal_moves(&sim);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], MechaMove::Reconnect);
    }

    // ── MechaCoordinateSystem::apply_move ────────────────────────────────────

    #[test]
    fn authenticate_true_transitions_to_authenticated() {
        let sys = MechaCoordinateSystem;
        let next = sys.apply_move(&connecting_sim(), &MechaMove::Authenticate(true)).unwrap();
        assert_eq!(next.state, SessionState::Authenticated);
    }

    #[test]
    fn authenticate_false_returns_error() {
        let sys = MechaCoordinateSystem;
        let result = sys.apply_move(&connecting_sim(), &MechaMove::Authenticate(false));
        assert!(result.is_err());
    }

    #[test]
    fn reject_transitions_to_disconnected() {
        let sys = MechaCoordinateSystem;
        let next = sys.apply_move(&connecting_sim(), &MechaMove::Reject).unwrap();
        assert_eq!(next.state, SessionState::Disconnected);
    }

    #[test]
    fn enter_match_from_lobby() {
        let sys = MechaCoordinateSystem;
        let next = sys.apply_move(&lobby_sim(), &MechaMove::EnterMatch(42)).unwrap();
        assert_eq!(next.state, SessionState::InMatch { match_id: 42 });
    }

    #[test]
    fn match_complete_returns_to_lobby() {
        let sys = MechaCoordinateSystem;
        let in_match = MechaSessionSimulation {
            state: SessionState::InMatch { match_id: 1 },
            profile: profile(),
            inventory: vec![],
        };
        let next = sys.apply_move(&in_match, &MechaMove::MatchComplete).unwrap();
        assert_eq!(next.state, SessionState::InLobby);
    }

    #[test]
    fn invalid_move_for_state_returns_error() {
        let sys = MechaCoordinateSystem;
        // MatchComplete is invalid in Connecting state
        let result = sys.apply_move(&connecting_sim(), &MechaMove::MatchComplete);
        assert!(result.is_err());
    }

    // ── MechaCoordinateSystem::move_to_notation ──────────────────────────────

    #[test]
    fn notation_for_authenticate_true() {
        let sys = MechaCoordinateSystem;
        assert_eq!(sys.move_to_notation(&MechaMove::Authenticate(true)), "auth:true");
    }

    #[test]
    fn notation_for_enter_match() {
        let sys = MechaCoordinateSystem;
        assert_eq!(sys.move_to_notation(&MechaMove::EnterMatch(7)), "enter_match:7");
    }

    #[test]
    fn notation_for_inventory_remove() {
        let sys = MechaCoordinateSystem;
        assert_eq!(sys.move_to_notation(&MechaMove::InventoryRemove(2)), "inventory_remove:2");
    }

    #[test]
    fn notation_for_disconnect() {
        let sys = MechaCoordinateSystem;
        assert_eq!(sys.move_to_notation(&MechaMove::Disconnect), "disconnect");
    }

    // ── SessionState enum properties ──────────────────────────────────────────

    #[test]
    fn session_states_are_distinct() {
        assert_ne!(SessionState::Connecting, SessionState::Authenticated);
        assert_ne!(SessionState::InLobby, SessionState::Disconnected);
        assert_ne!(
            SessionState::InMatch { match_id: 1 },
            SessionState::InMatch { match_id: 2 }
        );
    }
}
