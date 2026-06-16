// Shared helpers for integration tests
pub use ib4_mud::session::GameSession;
pub use ib4_mud::command::Command;
pub use ib4_core::types::AttackDir;

pub fn new_session() -> GameSession {
    GameSession::new("Siris")
}
