use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use chicago_tdd_tools::coordinate::{GameCoordinateSystem, InfinityBladeCoordinateSystem};
use ib4_mud::command::Command;
use ib4_mud::session::GameSession;

/// Defines a point allocation across player stats (health, attack, defense, magic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatAllocation {
    pub health: u32,
    pub attack: u32,
    pub defense: u32,
    pub magic: u32,
}

impl StatAllocation {
    /// Prune imbalanced coordinates to ensure structurally valid game configurations.
    pub fn is_imbalanced(&self, total_points: u32) -> bool {
        if total_points == 0 {
            return false;
        }
        
        // Prevent stats from being completely ignored if there are enough points
        if total_points >= 4 {
            if self.health == 0 || self.attack == 0 {
                return true;
            }
        }
        
        // Prevent all-in-one allocations as they are mathematically degenerate
        let max_stat = self.health.max(self.attack).max(self.defense).max(self.magic);
        if max_stat == total_points && total_points > 1 {
            return true;
        }

        false
    }
}

/// The aggregated results of running a batch of Monte Carlo battle simulations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub allocation: StatAllocation,
    pub player_win_rate: f64,
    pub avg_turns: f64,
    pub average_player_final_hp: f64,
}

/// Run Monte Carlo simulation battles with a specific stat allocation.
pub fn simulate_battles(alloc: &StatAllocation, num_blank_battles: usize) -> SimulationResult {
    let mut player_wins = 0;
    let mut total_turns = 0;
    let mut total_final_hp = 0.0;

    let coords = InfinityBladeCoordinateSystem;

    for _ in 0..num_blank_battles {
        let mut session = GameSession::new("Siris");

        session.player.stat_health = alloc.health;
        session.player.stat_attack = alloc.attack;
        session.player.stat_defense = alloc.defense;
        session.player.stat_magic = alloc.magic;
        session.player.recalculate_stats();
        session.player.health = session.player.max_health;

        session.dispatch(Command::Explore);

        let mut turns = 0;
        while session.is_in_combat() && turns < 100 {
            turns += 1;
            let legal_moves = coords.get_legal_moves(&session);
            if legal_moves.is_empty() {
                break;
            }

            let chosen_move = if session.announced_attack.is_some() {
                if legal_moves.contains(&Command::Parry) {
                    Command::Parry
                } else {
                    legal_moves[0].clone()
                }
            } else {
                if let Some(atk_move) =
                    legal_moves.iter().find(|m| matches!(m, Command::Attack(_)))
                {
                    atk_move.clone()
                } else {
                    legal_moves[0].clone()
                }
            };

            session.dispatch(chosen_move);
        }

        if session.player.health > 0.0 {
            player_wins += 1;
            total_final_hp += session.player.health;
        }
        total_turns += turns;
    }

    SimulationResult {
        allocation: alloc.clone(),
        player_win_rate: player_wins as f64 / num_blank_battles as f64,
        avg_turns: total_turns as f64 / num_blank_battles as f64,
        average_player_final_hp: if player_wins > 0 {
            total_final_hp as f64 / player_wins as f64
        } else {
            0.0
        },
    }
}

/// Optimize stat allocations to reach closest to a target win rate.
/// Prunes imbalanced coordinates and selects the mathematically perfect game configurations.
pub fn optimize_balance(
    total_points: u32,
    target_win_rate: f64,
    sims_per_config: usize,
) -> Result<SimulationResult> {
    let mut best_result: Option<SimulationResult> = None;
    let mut min_diff = f64::MAX;

    for h in 0..=total_points {
        for a in 0..=(total_points - h) {
            for d in 0..=(total_points - h - a) {
                let m = total_points - h - a - d;
                let alloc = StatAllocation {
                    health: h,
                    attack: a,
                    defense: d,
                    magic: m,
                };

                if alloc.is_imbalanced(total_points) {
                    continue;
                }

                let res = simulate_battles(&alloc, sims_per_config);
                let diff = (res.player_win_rate - target_win_rate).abs();

                if diff < min_diff {
                    min_diff = diff;
                    best_result = Some(res);
                }
            }
        }
    }

    best_result.context("Failed to find any valid allocation result after pruning imbalanced coordinates")
}
