use anyhow::Result;
use chicago_tdd_tools::aimbot::explore_state_space;
use chicago_tdd_tools::coordinate::{
    GameCoordinateSystem, GundamCoordinateSystem, GundamSessionSimulation,
    InfinityBladeCoordinateSystem, SessionState,
};
use ib4_mud::session::GameSession;
use nexus_session::player::PlayerProfile;

// A simple grid coordinate system for testing the traversal engine without errors.
struct SimpleGridSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
struct GridState {
    x: i32,
    y: i32,
}

impl GameCoordinateSystem for SimpleGridSystem {
    type State = GridState;
    type Move = &'static str;

    fn state_to_coordinate(&self, state: &Self::State) -> String {
        format!("{},{}", state.x, state.y)
    }

    fn get_legal_moves(&self, state: &Self::State) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        if state.x < 2 {
            moves.push("right");
        }
        if state.y < 2 {
            moves.push("up");
        }
        moves
    }

    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> Result<Self::State> {
        let mut next = state.clone();
        match *mv {
            "right" => next.x += 1,
            "up" => next.y += 1,
            _ => return Err(anyhow::anyhow!("Invalid move")),
        }
        Ok(next)
    }

    fn move_to_notation(&self, mv: &Self::Move) -> String {
        mv.to_string()
    }
}

#[test]
fn test_explore_state_space_visits_multiple_states_without_errors() {
    let system = SimpleGridSystem;
    let initial = GridState { x: 0, y: 0 };

    // Grid from (0,0) to (2,2) has 3 * 3 = 9 states.
    let result = explore_state_space(&system, initial, 100);

    assert_eq!(result.visited_states_count, 9);
    assert!(!result.transitions.is_empty());
    assert!(
        result.errors.is_empty(),
        "Expected no errors during exploration, but found: {:?}",
        result.errors
    );
}

#[test]
fn test_explore_state_space_with_infinity_blade() {
    let system = InfinityBladeCoordinateSystem;
    let initial = GameSession::new("SirisAimbot");

    let result = explore_state_space(&system, initial, 100);

    assert!(
        result.visited_states_count >= 20,
        "Expected at least 20 states, got {}",
        result.visited_states_count
    );
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_explore_state_space_with_gundam_nexus() {
    let system = GundamCoordinateSystem;
    let mut initial = GundamSessionSimulation {
        state: SessionState::Connecting,
        profile: PlayerProfile::new(1001, "PilotAimbot".to_string()),
        inventory: Vec::new(),
    };
    initial.profile.gold = 20;

    let result = explore_state_space(&system, initial.clone(), 1000);

    assert!(result.visited_states_count > 1);
    // There should be auth:false failures which are expected/handled
    assert!(!result.errors.is_empty());

    let mut visited_coords = std::collections::HashSet::new();
    visited_coords.insert(system.state_to_coordinate(&initial));
    for (from, _, to) in &result.transitions {
        visited_coords.insert(from.clone());
        visited_coords.insert(to.clone());
    }

    let mut visited_level_2 = false;
    let mut visited_gold_depleted = false;
    let mut visited_non_empty_inventory = false;

    for coord in &visited_coords {
        let parts: Vec<&str> = coord.split(':').collect();
        if parts.len() == 6 {
            if parts[2] == "lv2" {
                visited_level_2 = true;
            }
            if parts[4] != "i0" {
                visited_non_empty_inventory = true;
            }
            if parts[5] == "g0" {
                visited_gold_depleted = true;
            }
        }
    }

    assert!(
        visited_level_2,
        "Should have visited level 2 state. Visited: {:?}",
        visited_coords
    );
    assert!(
        visited_gold_depleted,
        "Should have visited gold depleted state (gold = 0). Visited: {:?}",
        visited_coords
    );
    assert!(
        visited_non_empty_inventory,
        "Should have visited non-empty inventory state. Visited: {:?}",
        visited_coords
    );
}
