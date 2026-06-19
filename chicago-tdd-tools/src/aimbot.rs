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
