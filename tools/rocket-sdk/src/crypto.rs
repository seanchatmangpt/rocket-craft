use anyhow::Result;
use std::fs;
use std::path::Path;
use colored::*;

/// Generate all missing keystores
pub fn generate_all_keystores() -> Result<()> {
    run_generate_all()
}

fn run_generate_all() -> Result<()> {
    // We will provide the commands to generate keystores using keytool,
    // as reliable native Rust PKCS12 generation is complex without heavy dependencies.
    
    let targets = vec![
        ("barbarian-road-mashines-key.keystore", "barbarian-road-mashines"),
        ("zombie-key.keystore", "zombie"),
        ("hang3d-nightmare-keystore.keystore", "NIGHTMARE"),
    ];

    tracing::info!("{}", "=== Android Keystore Generation Guide ===".bold().cyan());
    tracing::info!("Native Rust PKCS12 generation is a work-in-progress.");
    tracing::info!("Please use the following commands to generate your signing keys:\n");

    for (name, alias) in targets {
        if Path::new(name).exists() {
            tracing::info!("{}: {}", name.yellow(), "PRESENT".green());
        } else {
            let cmd = format!(
                "keytool -genkey -v -keystore {} -alias {} -keyalg RSA -keysize 2048 -validity 10000",
                name, alias
            );
            
            tracing::info!("{}: {}", name.yellow(), "MISSING".red());
            tracing::info!("  Command: {}\n", cmd.green());
            
            // Create placeholder if not exists
            manage_placeholder(name)?;
        }
    }

    Ok(())
}

pub fn generate_keystore(_path: &str, _alias: &str, _password: &str) -> Result<()> {
    // TRACKED_WORK(anti-cheat): All three parameters are accepted but silently discarded — this
    // function is an unimplemented stub that returns Ok(()) without doing anything.
    // Real implementation must:
    //   1. Generate a 2048-bit RSA key pair (e.g. via `rcgen`).
    //   2. Wrap it in a PKCS#12 envelope with the given alias and password
    //      (e.g. via `p12-keystore` or an `openssl` subprocess).
    //   3. Write the resulting .keystore file to `_path`.
    // Until implemented, callers must use the `keytool` command printed by
    // `generate_all_keystores()` / `rocket crypto generate`.
    Err(anyhow::anyhow!(
        "generate_keystore is not yet implemented; use `keytool -genkey ...` directly \
         (run `rocket crypto generate` for the exact command)"
    ))
}

pub fn manage_placeholder(keystore_path: &str) -> Result<()> {
    let placeholder_path = format!("{}.placeholder", keystore_path);
    if !Path::new(&placeholder_path).exists() {
        fs::write(&placeholder_path, "This is a placeholder for the actual keystore file. The real keystore should NOT be committed to version control.")?;
        tracing::info!("{} Created placeholder '{}'.", "INFO".blue(), placeholder_path);
    }
    Ok(())
}

/// Check status of keystores
pub fn check_status() -> Result<()> {
    run_check_status()
}

fn run_check_status() -> Result<()> {
    let targets = vec![
        "barbarian-road-mashines-key.keystore",
        "zombie-key.keystore",
        "hang3d-nightmare-keystore.keystore",
    ];

    tracing::info!("\n{} Keystore Status:", "Crypto".bold());
    for name in targets {
        let exists = Path::new(name).exists();
        let placeholder_exists = Path::new(&format!("{}.placeholder", name)).exists();
        
        let status = if exists {
            "PRESENT".green()
        } else if placeholder_exists {
            "MISSING (Placeholder present)".yellow()
        } else {
            "MISSING".red()
        };
        
        tracing::info!("  - {}: {}", name, status);
    }
    Ok(())
}
