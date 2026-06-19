use std::fs;
use std::path::PathBuf;
use std::process::{Command, Child, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::net::TcpStream;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("Failed to get current directory");
    loop {
        if dir.join("pwa-staff").is_dir() && dir.join("chicago-tdd-tools").is_dir() {
            return dir;
        }
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            return std::env::current_dir().unwrap();
        }
    }
}

#[test]
#[ignore = "requires real UE4 HTML5 packages in pwa-staff/ — run after Stage 5 packaging completes"]
fn validate_entire_game_suite_on_es3_pipeline() {
    let project_root = find_project_root();
    let pwa_dir = project_root.join("pwa-staff");

    // Start genie_server.js if not already running on port 3000
    let already_running = TcpStream::connect("127.0.0.1:3000").is_ok();
    let _server_guard = if already_running {
        None
    } else {
        let child = Command::new("node")
            .arg("../genie_server.js")
            .current_dir(&pwa_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start genie_server.js");

        let start_time = Instant::now();
        let timeout = Duration::from_secs(15);
        let mut online = false;
        while start_time.elapsed() < timeout {
            if TcpStream::connect("127.0.0.1:3000").is_ok() {
                online = true;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        if !online {
            panic!("genie_server.js did not bind to port 3000 within 15 seconds.");
        }
        Some(ChildGuard(child))
    };

    // 1. Discover all manufactured games
    let games: Vec<_> = fs::read_dir(&pwa_dir)
        .expect("Failed to read pwa-staff directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with("-Shipping.html"))
        .collect();

    assert!(!games.is_empty(), "No manufactured games found in pwa-staff/");

    let mut buffered_receipts = Vec::new();

    // 2. Iterate and validate each game via Playwright gate
    for game in &games {
        let game_name = game.file_name().to_string_lossy().to_string();
        println!("--- Validating Manufacturing Gate for: {} ---", game_name);

        // Run the Playwright test specifically for this game artifact
        let status = Command::new("npx")
            .arg("playwright")
            .arg("test")
            .arg("tests-e2e/tps-dflss.spec.ts")
            .env("TARGET_GAME_URL", format!("/{}", game_name))
            .current_dir(&pwa_dir)
            .status()
            .expect("Failed to execute Playwright");

        assert!(status.success(), "Manufacturing Gate FAILED for game: {}", game_name);

        // Buffer the generated receipt in memory
        let test_results_dir = pwa_dir.join("test-results");
        let src_receipt = test_results_dir.join("tps-dflss-receipt.json");
        if src_receipt.exists() {
            let content = fs::read_to_string(&src_receipt).expect("Failed to read generated receipt");
            buffered_receipts.push((game_name.clone(), content));
        }
    }

    // 3. Write all buffered receipts back to the test-results directory
    let test_results_dir = pwa_dir.join("test-results");
    if !test_results_dir.exists() {
        fs::create_dir_all(&test_results_dir).expect("Failed to create test-results directory");
    }

    for (game_name, content) in buffered_receipts {
        // 1. With .html extension (e.g., tps-dflss-receipt-Brm-HTML5-Shipping.html.json)
        let dest_with_ext = test_results_dir.join(format!("tps-dflss-receipt-{}.json", game_name));
        fs::write(&dest_with_ext, &content).expect("Failed to write receipt with extension");

        // 2. Without .html extension (e.g., tps-dflss-receipt-Brm-HTML5-Shipping.json)
        if let Some(stripped) = game_name.strip_suffix(".html") {
            let dest_no_ext = test_results_dir.join(format!("tps-dflss-receipt-{}.json", stripped));
            fs::write(&dest_no_ext, &content).expect("Failed to write receipt without extension");
        }
    }
}

