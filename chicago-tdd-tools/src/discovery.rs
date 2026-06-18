use ib4_mud::session::GameSession;
use nexus_session::PlayerSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredGame {
    pub name: String,
    pub crate_name: String,
    pub details: String,
}

/// Discovers the active game systems by instantiating and verifying their session profiles.
/// This acts as a compiler and linkage check for the entire workspace game suites.
pub fn discover_games() -> Vec<DiscoveredGame> {
    let mut discovered = Vec::new();

    // 1. Discover Infinity Blade 4 MUD
    // Verify that the game session can be initialized and we can query player name.
    let ib4_session = GameSession::new("DiscoveryTester");
    discovered.push(DiscoveredGame {
        name: "Infinity Blade 4 MUD".to_string(),
        crate_name: "ib4-mud".to_string(),
        details: format!("Discovered player session for: {}", ib4_session.player.name),
    });

    // 2. Discover Gundam Nexus
    // Verify that the PlayerSession can be instantiated in Connecting state.
    let nexus_sess = PlayerSession::new(1001, "NexusTester".to_string());
    discovered.push(DiscoveredGame {
        name: "Gundam Nexus".to_string(),
        crate_name: "nexus-session".to_string(),
        details: format!(
            "Discovered Gundam pilot session: {} (ID: {})",
            nexus_sess.username, nexus_sess.player_id
        ),
    });

    discovered
}
