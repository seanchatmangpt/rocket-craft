use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn validate_entire_game_suite_on_es3_pipeline() {
    let pwa_dir = Path::new("../pwa-staff");
    
    // 1. Discover all manufactured games
    let games: Vec<_> = fs::read_dir(pwa_dir)
        .expect("Failed to read pwa-staff directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with("-Shipping.html"))
        .collect();

    assert!(!games.is_empty(), "No manufactured games found in pwa-staff/");

    // 2. Iterate and validate each game via Playwright gate
    for game in games {
        let game_name = game.file_name().to_string_lossy().to_string();
        println!("--- Validating Manufacturing Gate for: {} ---", game_name);

        // Run the Playwright test specifically for this game artifact
        let status = Command::new("npx")
            .arg("playwright")
            .arg("test")
            .arg("tests-e2e/tps-dflss.spec.ts")
            .env("TARGET_GAME_URL", format!("/{}", game_name))
            .current_dir(pwa_dir)
            .status()
            .expect("Failed to execute Playwright");

        assert!(status.success(), "Manufacturing Gate FAILED for game: {}", game_name);
    }
}
