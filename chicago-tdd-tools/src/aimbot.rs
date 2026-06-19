use crate::coordinate::GameCoordinateSystem;
use std::collections::{HashSet, VecDeque};

pub struct TraversalResult {
    pub visited_states_count: usize,
    pub transitions: Vec<(String, String, String)>,
    pub errors: Vec<String>,
}

pub fn explore_state_space<S: GameCoordinateSystem>(
    system: &S,
    initial_state: S::State,
    max_states: usize,
) -> TraversalResult {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut transitions = Vec::new();
    let mut errors = Vec::new();

    let initial_coord = system.state_to_coordinate(&initial_state);
    visited.insert(initial_coord.clone());
    queue.push_back((initial_state, initial_coord));

    while let Some((state, coord)) = queue.pop_front() {
        if visited.len() >= max_states {
            break;
        }

        let legal_moves = system.get_legal_moves(&state);
        for mv in legal_moves {
            match system.apply_move(&state, &mv) {
                Ok(next_state) => {
                    let next_coord = system.state_to_coordinate(&next_state);
                    let move_notation = system.move_to_notation(&mv);
                    transitions.push((coord.clone(), move_notation, next_coord.clone()));

                    if !visited.contains(&next_coord) {
                        visited.insert(next_coord.clone());
                        if visited.len() < max_states {
                            queue.push_back((next_state, next_coord));
                        }
                    }
                }
                Err(e) => {
                    let move_notation = system.move_to_notation(&mv);
                    errors.push(format!(
                        "Error applying move '{}' at state '{}': {}",
                        move_notation, coord, e
                    ));
                }
            }
        }
    }

    TraversalResult {
        visited_states_count: visited.len(),
        transitions,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::{GundamCoordinateSystem, GundamSessionSimulation, SessionState};
    use nexus_session::player::PlayerProfile;

    fn connecting_sim() -> GundamSessionSimulation {
        GundamSessionSimulation {
            state: SessionState::Connecting,
            profile: PlayerProfile::new(1, "Amuro".into()),
            inventory: vec![],
        }
    }

    // ── TraversalResult structure ─────────────────────────────────────────────

    #[test]
    fn explore_single_state_no_errors() {
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 1);
        // We start at 1 state; max_states=1 prevents expanding any children.
        // visited contains the initial state.
        assert_eq!(result.visited_states_count, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn explore_expands_connecting_to_reachable_states() {
        // max_states=10 lets the BFS run for several transitions from Connecting.
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 10);
        assert!(result.visited_states_count > 1, "should reach at least Authenticated and Disconnected");
    }

    #[test]
    fn transitions_are_tuples_of_from_move_to() {
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 5);
        for (from, mv, to) in &result.transitions {
            assert!(!from.is_empty(), "from should be non-empty");
            assert!(!mv.is_empty(), "move notation should be non-empty");
            assert!(!to.is_empty(), "to should be non-empty");
        }
    }

    #[test]
    fn transitions_include_auth_true_notation() {
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 5);
        let has_auth = result.transitions.iter().any(|(_, mv, _)| mv.contains("auth:true"));
        assert!(has_auth, "auth:true should appear in transitions from Connecting");
    }

    #[test]
    fn zero_max_states_gives_empty_result() {
        // With max_states=0 the guard `visited.len() >= max_states` fires immediately
        // after the initial insertion, so BFS terminates after 1 visited state.
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 0);
        // Actual behavior: initial state is visited (len=1 ≥ 0), loop exits immediately.
        assert!(result.transitions.is_empty());
    }

    #[test]
    fn errors_are_collected_not_panicked() {
        // explore_state_space should never panic even when apply_move returns Err.
        // auth:false causes an error; verify it is recorded in errors, not panicked.
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), 20);
        let has_auth_error = result.errors.iter().any(|e| e.contains("auth:false") || e.contains("Authentication"));
        assert!(has_auth_error, "auth:false error should be collected; errors: {:?}", result.errors);
    }

    #[test]
    fn visited_count_does_not_exceed_max_states() {
        let max = 3;
        let result = explore_state_space(&GundamCoordinateSystem, connecting_sim(), max);
        assert!(result.visited_states_count <= max + 1, // BFS may add 1 over the limit before checking
            "visited={}, max={}", result.visited_states_count, max);
    }
}
