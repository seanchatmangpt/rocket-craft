use std::marker::PhantomData;

use crate::ecs::World;
use crate::systems::{CombatSystem, PhysicsSystem};

// ── Phase markers (zero-sized) ───────────────────────────────────────────────

pub struct Initializing;
pub struct Running;
pub struct Paused;
pub struct GameOver;

// ── Typestate game state ─────────────────────────────────────────────────────

/// The game's top-level state machine. The phase `S` is a zero-sized
/// `PhantomData` type parameter; illegal transitions simply don't exist as
/// methods, so they are compile errors rather than runtime panics.
///
/// Transitions:
///   Initializing → Running
///   Running → Paused | GameOver
///   Paused → Running | GameOver
///   GameOver → Initializing  (restart)
pub struct GameState<S> {
    pub world: World,
    pub tick: u64,
    pub elapsed_ms: u64,
    _phase: PhantomData<S>,
}

// ── Initializing ─────────────────────────────────────────────────────────────

impl GameState<Initializing> {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            tick: 0,
            elapsed_ms: 0,
            _phase: PhantomData,
        }
    }

    /// Transition: Initializing → Running.
    pub fn start(self) -> GameState<Running> {
        GameState {
            world: self.world,
            tick: 0,
            elapsed_ms: 0,
            _phase: PhantomData,
        }
    }
}

impl Default for GameState<Initializing> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Running ───────────────────────────────────────────────────────────────────

impl GameState<Running> {
    /// Advance the simulation by `delta_ms` milliseconds.
    pub fn tick(&mut self, delta_ms: u64) {
        self.elapsed_ms += delta_ms;
        self.tick += 1;
        PhysicsSystem::run(&mut self.world, delta_ms);
        CombatSystem::run_cleanup(&mut self.world);
    }

    /// Transition: Running → Paused.
    pub fn pause(self) -> GameState<Paused> {
        GameState {
            world: self.world,
            tick: self.tick,
            elapsed_ms: self.elapsed_ms,
            _phase: PhantomData,
        }
    }

    /// Transition: Running → GameOver.
    pub fn game_over(self) -> GameState<GameOver> {
        GameState {
            world: self.world,
            tick: self.tick,
            elapsed_ms: self.elapsed_ms,
            _phase: PhantomData,
        }
    }

    pub fn is_running(&self) -> bool {
        true
    }
}

// ── Paused ────────────────────────────────────────────────────────────────────

impl GameState<Paused> {
    /// Transition: Paused → Running.
    pub fn resume(self) -> GameState<Running> {
        GameState {
            world: self.world,
            tick: self.tick,
            elapsed_ms: self.elapsed_ms,
            _phase: PhantomData,
        }
    }

    /// Transition: Paused → GameOver.
    pub fn game_over(self) -> GameState<GameOver> {
        GameState {
            world: self.world,
            tick: self.tick,
            elapsed_ms: self.elapsed_ms,
            _phase: PhantomData,
        }
    }

    pub fn is_paused(&self) -> bool {
        true
    }
}

// ── GameOver ──────────────────────────────────────────────────────────────────

impl GameState<GameOver> {
    /// Highest score among surviving players, or 0.
    pub fn winner_score(&self) -> u64 {
        self.world
            .entities_with_health()
            .filter_map(|e| self.world.get_player(e).map(|p| p.score))
            .max()
            .unwrap_or(0)
    }

    /// Total ticks that elapsed during this game.
    pub fn total_ticks(&self) -> u64 {
        self.tick
    }

    /// Transition: GameOver → Initializing (restart with fresh world).
    pub fn restart(self) -> GameState<Initializing> {
        GameState::new()
    }
}
