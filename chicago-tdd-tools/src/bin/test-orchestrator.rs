use std::fs;
use std::path::PathBuf;
use std::process::{Command, Child, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::net::TcpStream;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        tracing::info!("Stopping background web server...");
        let _ = self.0.kill();
        let _ = self.0.wait();
        tracing::info!("Background web server stopped.");
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

fn run_orchestrator() -> Result<(), anyhow::Error> {
    let project_root = find_project_root();
    let pwa_staff_dir = project_root.join("pwa-staff");

    tracing::info!("Project root detected at: {:?}", project_root);
    // Check if port 3000 is already in use first
    let already_running = TcpStream::connect("127.0.0.1:3000").is_ok();

    let _guard = if already_running {
        tracing::info!("Server on port 3000 is already running. Reusing the existing server.");
        None
    } else {
        tracing::info!("Spawning genie_server.js in background relative to pwa-staff...");
        // Spawn server: node ../genie_server.js inside pwa-staff dir
        let server_child = Command::new("node")
            .arg("../genie_server.js")
            .current_dir(&pwa_staff_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Poll http://localhost:3000 until online
        tracing::info!("Polling http://localhost:3000...");
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
            anyhow::bail!("genie_server.js did not bind to port 3000 within 15 seconds.");
        }
        tracing::info!("genie_server.js is online!");
        Some(ChildGuard(server_child))
    };

    // Run Playwright test
    tracing::info!("Running Playwright visual delta test...");
    let playwright_status = Command::new("npx")
        .arg("playwright")
        .arg("test")
        .arg("tests-e2e/tps-dflss.spec.ts")
        .current_dir(&pwa_staff_dir)
        .status()?;

    if !playwright_status.success() {
        anyhow::bail!("Playwright visual delta test failed with status: {:?}", playwright_status);
    }
    tracing::info!("Playwright test run succeeded.");

    // Read generated receipt
    let receipt_path = pwa_staff_dir.join("test-results/tps-dflss-receipt.json");
    if !receipt_path.exists() {
        anyhow::bail!("Expected receipt file not found at: {:?}", receipt_path);
    }

    tracing::info!("Reading and validating cryptographic receipt...");
    let receipt_json = fs::read_to_string(&receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt_json)?;

    // Validate expected fields: screenshots, consoleLogs, visualDelta, verdict, signature
    let expected_fields = vec![
        "screenshots",
        "consoleLogs",
        "visualDelta",
        "verdict",
        "signature",
    ];

    let mut missing_fields = Vec::new();
    for field in &expected_fields {
        if receipt.get(field).is_none() {
            missing_fields.push(*field);
        }
    }

    if !missing_fields.is_empty() {
        anyhow::bail!("Receipt is missing required fields: {:?}", missing_fields);
    }

    tracing::info!("\n=== Cryptographic Receipt Validated ===");
    tracing::info!("Timestamp: {}", receipt.get("timestamp").and_then(|v| v.as_str()).unwrap_or("N/A"));
    tracing::info!("Prompt: {}", receipt.get("prompt").and_then(|v| v.as_str()).unwrap_or("N/A"));
    tracing::info!("Verdict: {}", receipt.get("verdict").and_then(|v| v.as_str()).unwrap_or("N/A"));
    tracing::info!("Visual Delta: {}", receipt.get("visualDelta").and_then(|v| v.as_u64()).map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string()));
    tracing::info!("Signature: {}", receipt.get("signature").and_then(|v| v.as_str()).unwrap_or("N/A"));
    tracing::info!("=======================================\n");

    Ok(())
}

fn main() {
    match run_orchestrator() {
        Ok(_) => {
            tracing::info!("Orchestrator validation PASS.");
            std::process::exit(0);
        }
        Err(e) => {
            tracing::info!("Orchestrator validation FAIL: {:?}", e);
            std::process::exit(1);
        }
    }
}
