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

    println!("{}", "=== Android Keystore Generation Guide ===".bold().cyan());
    println!("Native Rust PKCS12 generation is a work-in-progress.");
    println!("Please use the following commands to generate your signing keys:\n");

    for (name, alias) in targets {
        if Path::new(name).exists() {
            println!("{}: {}", name.yellow(), "PRESENT".green());
        } else {
            let cmd = format!(
                "keytool -genkey -v -keystore {} -alias {} -keyalg RSA -keysize 2048 -validity 10000",
                name, alias
            );
            
            println!("{}: {}", name.yellow(), "MISSING".red());
            println!("  Command: {}\n", cmd.green());
            
            // Create placeholder if not exists
            manage_placeholder(name)?;
        }
    }

    Ok(())
}

pub fn generate_keystore(_path: &str, _alias: &str, _password: &str) -> Result<()> {
    // Placeholder for future native implementation
    Ok(())
}

pub fn manage_placeholder(keystore_path: &str) -> Result<()> {
    let placeholder_path = format!("{}.placeholder", keystore_path);
    if !Path::new(&placeholder_path).exists() {
        fs::write(&placeholder_path, "This is a placeholder for the actual keystore file. The real keystore should NOT be committed to version control.")?;
        println!("{} Created placeholder '{}'.", "INFO".blue(), placeholder_path);
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

    println!("\n{} Keystore Status:", "Crypto".bold());
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
        
        println!("  - {}: {}", name, status);
    }
    Ok(())
}
