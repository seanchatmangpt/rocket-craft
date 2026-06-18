# Rust API and Typestate Specifications

This reference manual provides key API definitions, structures, traits, and typestate representations for the core crates of the Rocket-Craft ecosystem: `unify-automl`, `chicago-tdd-tools`, and `nexus-types`.

---

## 1. `unify-automl` Crate

The `unify-automl` crate manages component dynamic discovery and Monte Carlo balancing simulation.

### Discovery Registry Structures

```rust
pub mod discovery {
    use std::path::Path;
    use anyhow::Result;

    /// A representation of a component found during dynamic discovery.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub struct DiscoveredComponent {
        /// The name of the discovered component.
        pub name: String,
        /// The file path to the source file where the component is defined.
        pub file_path: String,
        /// The programming language of the source file (e.g. "Rust", "C++").
        pub language: String,
        /// The binding tag annotation found (e.g. "@UnifyAutoBind: CombatSystem" or "#[derive(AutoBind)]").
        pub binding_tag: String,
    }

    /// The registry containing all discovered components and active workspace games.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ComponentRegistry {
        /// List of all discovered game components.
        pub components: Vec<DiscoveredComponent>,
        /// List of all detected workspace games.
        pub workspace_games: Vec<String>,
    }

    /// Recursively scans a directory for files containing `@UnifyAutoBind` tags or `#[derive(AutoBind)]` macros.
    pub fn scan_directory<P: AsRef<Path>>(dir: P) -> Result<ComponentRegistry>;

    /// Extract component name from comment annotations.
    pub fn extract_name_from_tag(tag: &str) -> Option<String>;
}
```

### Balancer Structures & Functions

```rust
pub mod balancer {
    use anyhow::Result;

    /// Defines a point allocation across player stats.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct StatAllocation {
        pub health: u32,
        pub attack: u32,
        pub defense: u32,
        pub magic: u32,
    }

    /// Aggregated results of running Monte Carlo battle simulations.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SimulationResult {
        /// The stat allocation that was simulated.
        pub allocation: StatAllocation,
        /// The ratio of battles won by the player (0.0 to 1.0).
        pub player_win_rate: f64,
        /// The average number of turns elapsed per battle.
        pub avg_turns: f64,
        /// The average final health of the player in won battles.
        pub average_player_final_hp: f64,
    }

    /// Simulates `num_blank_battles` combat rounds with a specific stat configuration.
    pub fn simulate_battles(alloc: &StatAllocation, num_blank_battles: usize) -> SimulationResult;

    /// Iteratively evaluates stat combinations totaling `total_points` to find the win rate closest to `target_win_rate`.
    pub fn optimize_balance(
        total_points: u32,
        target_win_rate: f64,
        sims_per_config: usize,
    ) -> Result<SimulationResult>;
}
```

---

## 2. `chicago-tdd-tools` Crate

The `chicago-tdd-tools` crate contains the formal state-space modeling traits and the aimbot BFS state-space search engine.

### `GameCoordinateSystem` Trait
All game states must implement this trait to enable exploration by the combinatorial engine.

```rust
pub trait GameCoordinateSystem {
    /// The concrete state representation struct.
    type State;
    /// The discrete actions or moves that can transition the state.
    type Move: std::fmt::Debug + Clone;

    /// Encodes a state to its chess-coordinate string representation.
    fn state_to_coordinate(&self, state: &Self::State) -> String;

    /// Evaluates the current state and returns a list of legal actions.
    fn get_legal_moves(&self, state: &Self::State) -> Vec<Self::Move>;

    /// Applies a move to the current state, returning the next state or an error.
    fn apply_move(&self, state: &Self::State, mv: &Self::Move) -> anyhow::Result<Self::State>;

    /// Returns the notation string for the given action.
    fn move_to_notation(&self, mv: &Self::Move) -> String;
}
```

### Discovery Registry Interface

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredGame {
    pub name: String,
    pub crate_name: String,
    pub details: String,
}

/// Discovers active game session profiles by instantiating them and verifying compilation linkage.
pub fn discover_games() -> Vec<DiscoveredGame>;
```

### State-Space Explorer (Aimbot)

```rust
pub struct TraversalResult {
    /// Number of unique state coordinates visited.
    pub visited_states_count: usize,
    /// Vector of state transitions: (source_coord, move_notation, target_coord).
    pub transitions: Vec<(String, String, String)>,
    /// Any execution errors encountered during state transitions.
    pub errors: Vec<String>,
}

/// Runs a breadth-first search traversal up to `max_states` to explore the state space of a game coordinate system.
pub fn explore_state_space<S: GameCoordinateSystem>(
    system: &S,
    initial_state: S::State,
    max_states: usize,
) -> TraversalResult;
```

---

## 3. `nexus-types` Crate

The `nexus-types` crate contains the foundational types, math units, strongly-typed IDs, and sealed typestate markers.

### Sealed Trait Pattern
Downstream crates are prevented from introducing unauthorized typestates by sealing marker traits.

```rust
mod private {
    pub trait Sealed {}
    impl Sealed for super::Idle {}
    impl Sealed for super::Attacking {}
    impl Sealed for super::Parrying {}
    impl Sealed for super::Dodging {}
    impl Sealed for super::Stunned {}
    impl Sealed for super::Dead {}
    
    impl Sealed for super::Connecting {}
    impl Sealed for super::Authenticated {}
    impl Sealed for super::InLobby {}
    impl Sealed for super::InMatch {}
    impl Sealed for super::Spectating {}
    impl Sealed for super::Disconnected {}

    impl Sealed for super::PendingBid {}
    impl Sealed for super::BidAccepted {}
    impl Sealed for super::BidRejected {}
    impl Sealed for super::AuctionClosed {}
}
```

### Combat Typestates
Enforces combat phase correctness at compile time.

```rust
pub trait CombatStateMarker: private::Sealed {}

pub struct Idle;
pub struct Attacking;
pub struct Parrying;
pub struct Dodging;
pub struct Stunned;
pub struct Dead;

impl CombatStateMarker for Idle {}
impl CombatStateMarker for Attacking {}
impl CombatStateMarker for Parrying {}
impl CombatStateMarker for Dodging {}
impl CombatStateMarker for Stunned {}
impl CombatStateMarker for Dead {}
```

### Session Typestates
Represents multiplayer socket session phases.

```rust
pub trait SessionStateMarker: private::Sealed {}

pub struct Connecting;
pub struct Authenticated;
pub struct InLobby;
pub struct InMatch;
pub struct Spectating;
pub struct Disconnected;

impl SessionStateMarker for Connecting {}
impl SessionStateMarker for Authenticated {}
impl SessionStateMarker for InLobby {}
impl SessionStateMarker for InMatch {}
impl SessionStateMarker for Spectating {}
impl SessionStateMarker for Disconnected {}
```

### Economy Typestates
Represents transaction states in the auction house.

```rust
pub trait EconomyStateMarker: private::Sealed {}

pub struct PendingBid;
pub struct BidAccepted;
pub struct BidRejected;
pub struct AuctionClosed;

impl EconomyStateMarker for PendingBid {}
impl EconomyStateMarker for BidAccepted {}
impl EconomyStateMarker for BidRejected {}
impl EconomyStateMarker for AuctionClosed {}
```

### Runtime Enums (Wire Format & Matchers)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttackDir {
    Overhead,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ParryOutcome {
    Miss,
    Normal,
    Perfect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MagicType {
    Fire,
    Lightning,
    Ice,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Infinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TitanType {
    Warrior,
    Mage,
    Archer,
    Heavy,
    GodKing,
}
```

### Foundational Primitive Units
Tuple structures implementing numeric types with phantom parameters to restrict mixed algebra.

```rust
pub struct Hp(pub f32);
pub struct Mana(pub f32);
pub struct Damage(pub f32);
pub struct Armor(pub f32);
pub struct Gold(pub u32);
pub struct Xp(pub u64);
pub struct ComboMultiplier(pub f32);
pub struct TimeDilation(pub f32);

/// A wrapper type that attaches a phantom marker type parameter for strict compile-time checking.
pub struct Typed<T, Marker> {
    pub value: T,
    _marker: std::marker::PhantomData<Marker>,
}
```
