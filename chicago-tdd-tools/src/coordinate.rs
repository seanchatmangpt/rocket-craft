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
pub struct GundamSessionSimulation {
    pub state: SessionState,
    pub profile: PlayerProfile,
    pub inventory: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GundamMove {
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

pub struct GundamCoordinateSystem;

impl GameCoordinateSystem for GundamCoordinateSystem {
    type State = GundamSessionSimulation;
    type Move = GundamMove;

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
                moves.push(GundamMove::Authenticate(true));
                moves.push(GundamMove::Authenticate(false));
                moves.push(GundamMove::Reject);
            }
            SessionState::Authenticated => {
                moves.push(GundamMove::EnterLobby);
                moves.push(GundamMove::Disconnect);
            }
            SessionState::InLobby => {
                moves.push(GundamMove::EnterMatch(42));
                moves.push(GundamMove::Spectate(42));
                moves.push(GundamMove::Disconnect);
                moves.push(GundamMove::ApplyXP(100));
                if state.profile.gold >= 10 {
                    moves.push(GundamMove::SpendGold(10));
                } else if state.profile.gold > 0 {
                    moves.push(GundamMove::SpendGold(state.profile.gold));
                }
                if state.inventory.len() < 5 {
                    moves.push(GundamMove::InventoryAdd);
                }
                for i in 0..state.inventory.len() {
                    moves.push(GundamMove::InventoryRemove(i));
                }
            }
            SessionState::InMatch { .. } => {
                moves.push(GundamMove::MatchComplete);
                moves.push(GundamMove::Disconnect);
            }
            SessionState::Spectating { .. } => {
                moves.push(GundamMove::LeaveSpectate);
                moves.push(GundamMove::Disconnect);
            }
            SessionState::Disconnected => {
                moves.push(GundamMove::Reconnect);
            }
        }
        moves
    }

    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> Result<Self::State> {
        let mut next = state.clone();
        match (&state.state, mv) {
            (SessionState::Connecting, GundamMove::Authenticate(true)) => {
                next.state = SessionState::Authenticated;
            }
            (SessionState::Connecting, GundamMove::Authenticate(false)) => {
                return Err(anyhow::anyhow!("Authentication failed"));
            }
            (SessionState::Connecting, GundamMove::Reject) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Authenticated, GundamMove::EnterLobby) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::Authenticated, GundamMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::InLobby, GundamMove::EnterMatch(match_id)) => {
                next.state = SessionState::InMatch {
                    match_id: *match_id,
                };
            }
            (SessionState::InLobby, GundamMove::Spectate(match_id)) => {
                next.state = SessionState::Spectating {
                    match_id: *match_id,
                };
            }
            (SessionState::InLobby, GundamMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::InLobby, GundamMove::ApplyXP(amount)) => {
                next.profile.apply_xp_gain(*amount);
            }
            (SessionState::InLobby, GundamMove::SpendGold(amount)) => {
                next.profile
                    .spend_gold(*amount)
                    .map_err(|e| anyhow::anyhow!("Spend gold failed: {}", e))?;
            }
            (SessionState::InLobby, GundamMove::InventoryAdd) => {
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
            (SessionState::InLobby, GundamMove::InventoryRemove(slot)) => {
                if *slot < next.inventory.len() {
                    next.inventory.remove(*slot);
                } else {
                    return Err(anyhow::anyhow!("Invalid inventory slot"));
                }
            }
            (SessionState::InMatch { .. }, GundamMove::MatchComplete) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::InMatch { .. }, GundamMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Spectating { .. }, GundamMove::LeaveSpectate) => {
                next.state = SessionState::InLobby;
            }
            (SessionState::Spectating { .. }, GundamMove::Disconnect) => {
                next.state = SessionState::Disconnected;
            }
            (SessionState::Disconnected, GundamMove::Reconnect) => {
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
            GundamMove::Authenticate(val) => format!("auth:{}", val),
            GundamMove::Reject => "reject".to_string(),
            GundamMove::EnterLobby => "enter_lobby".to_string(),
            GundamMove::EnterMatch(match_id) => format!("enter_match:{}", match_id),
            GundamMove::Spectate(match_id) => format!("spectate:{}", match_id),
            GundamMove::Disconnect => "disconnect".to_string(),
            GundamMove::ApplyXP(amount) => format!("apply_xp:{}", amount),
            GundamMove::SpendGold(amount) => format!("spend_gold:{}", amount),
            GundamMove::MatchComplete => "match_complete".to_string(),
            GundamMove::LeaveSpectate => "leave_spectate".to_string(),
            GundamMove::Reconnect => "reconnect".to_string(),
            GundamMove::InventoryAdd => "inventory_add".to_string(),
            GundamMove::InventoryRemove(slot) => format!("inventory_remove:{}", slot),
        }
    }
}
