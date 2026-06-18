pub mod domain;
pub mod cli;
pub mod logging;
pub mod discovery;
pub mod coordinate;
pub mod aimbot;

pub use domain::account::Account;
pub use domain::transfer::TransferService;
pub use domain::environment::TestEnvironment;
pub use cli::ClapNoun;
pub use discovery::{discover_games, DiscoveredGame};
pub use coordinate::{
    GameCoordinateSystem,
    InfinityBladeCoordinateSystem,
    GundamSessionSimulation,
    SessionState,
    GundamMove,
    GundamCoordinateSystem,
};
pub use aimbot::{explore_state_space, TraversalResult};

pub use logging::{Logger, LogLevel, LogSink, StdoutSink, FileSink, TuiBufferSink};
