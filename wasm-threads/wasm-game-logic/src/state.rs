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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Initializing → Running ────────────────────────────────────────────────

    #[test]
    fn new_state_starts_at_tick_zero() {
        let s = GameState::<Initializing>::new();
        assert_eq!(s.tick, 0);
        assert_eq!(s.elapsed_ms, 0);
    }

    #[test]
    fn default_equals_new() {
        let s: GameState<Initializing> = Default::default();
        assert_eq!(s.tick, 0);
    }

    #[test]
    fn start_transitions_to_running() {
        let running = GameState::<Initializing>::new().start();
        assert!(running.is_running());
        assert_eq!(running.tick, 0);
    }

    // ── Running ticks ─────────────────────────────────────────────────────────

    #[test]
    fn tick_increments_counter_and_elapsed() {
        let mut s = GameState::<Initializing>::new().start();
        s.tick(16);
        assert_eq!(s.tick, 1);
        assert_eq!(s.elapsed_ms, 16);
        s.tick(16);
        assert_eq!(s.tick, 2);
        assert_eq!(s.elapsed_ms, 32);
    }

    // ── Running → Paused ─────────────────────────────────────────────────────

    #[test]
    fn pause_preserves_tick_and_elapsed() {
        let mut running = GameState::<Initializing>::new().start();
        running.tick(100);
        let paused = running.pause();
        assert!(paused.is_paused());
        assert_eq!(paused.tick, 1);
        assert_eq!(paused.elapsed_ms, 100);
    }

    // ── Paused → Running ─────────────────────────────────────────────────────

    #[test]
    fn resume_from_paused_is_running() {
        let mut s = GameState::<Initializing>::new().start();
        s.tick(50);
        let resumed = s.pause().resume();
        assert!(resumed.is_running());
        assert_eq!(resumed.elapsed_ms, 50);
    }

    // ── Running → GameOver ────────────────────────────────────────────────────

    #[test]
    fn game_over_from_running_preserves_tick() {
        let mut s = GameState::<Initializing>::new().start();
        s.tick(200);
        let over = s.game_over();
        assert_eq!(over.total_ticks(), 1);
    }

    #[test]
    fn winner_score_is_zero_with_no_players() {
        let over = GameState::<Initializing>::new().start().game_over();
        assert_eq!(over.winner_score(), 0);
    }

    // ── GameOver → Initializing (restart) ────────────────────────────────────

    #[test]
    fn restart_gives_fresh_initializing_state() {
        let mut s = GameState::<Initializing>::new().start();
        s.tick(999);
        let restarted = s.game_over().restart();
        assert_eq!(restarted.tick, 0);
        assert_eq!(restarted.elapsed_ms, 0);
    }

    // ── Paused → GameOver ─────────────────────────────────────────────────────

    #[test]
    fn game_over_from_paused_state() {
        let mut s = GameState::<Initializing>::new().start();
        s.tick(10);
        let over = s.pause().game_over();
        assert_eq!(over.total_ticks(), 1);
    }
}
