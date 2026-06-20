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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_games_returns_two_entries() {
        let games = discover_games();
        assert_eq!(games.len(), 2);
    }

    #[test]
    fn infinity_blade_entry_has_correct_crate_name() {
        let games = discover_games();
        let ib4 = games.iter().find(|g| g.crate_name == "ib4-mud").unwrap();
        assert_eq!(ib4.name, "Infinity Blade 4 MUD");
    }

    #[test]
    fn gundam_nexus_entry_has_correct_crate_name() {
        let games = discover_games();
        let nexus = games.iter().find(|g| g.crate_name == "nexus-session").unwrap();
        assert_eq!(nexus.name, "Gundam Nexus");
    }

    #[test]
    fn details_contain_player_name() {
        let games = discover_games();
        let ib4 = games.iter().find(|g| g.crate_name == "ib4-mud").unwrap();
        assert!(ib4.details.contains("DiscoveryTester"));
        let nexus = games.iter().find(|g| g.crate_name == "nexus-session").unwrap();
        assert!(nexus.details.contains("NexusTester"));
    }

    #[test]
    fn discovered_game_is_clone_and_eq() {
        let g = DiscoveredGame {
            name: "Test".into(), crate_name: "test".into(), details: "d".into(),
        };
        assert_eq!(g.clone(), g);
    }
}
