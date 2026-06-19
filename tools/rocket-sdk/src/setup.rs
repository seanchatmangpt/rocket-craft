use crate::config::RocketConfig;
use inquire::{Confirm, Select, Text};
use std::env;
use std::path::{Path, PathBuf};
use tracing::{error, info, instrument, warn};

/// Locate the UE4 engine root from `.rocket.json` / `UE4_ROOT` / common paths.
pub fn find_engine_root() -> anyhow::Result<PathBuf> {
    let config = RocketConfig::load()?;
    find_ue4_root(&config)?
        .ok_or_else(|| anyhow::anyhow!(
            "UE4 engine root not found. Set UE4_ROOT or add '{{\"ue4_root\": \"...\"}}' to .rocket.json"
        ))
}

#[instrument]
pub fn run_setup() -> anyhow::Result<()> {
    info!("Starting Rocket Craft Project Setup");

    let mut config = RocketConfig::load()?;

    let ue4_root = find_ue4_root(&config)?;

    if let Some(root) = ue4_root {
        info!("Found Unreal Engine 4.27 HTML5 ES3 at: {:?}", root);
        config.ue4_root = Some(root);
        config.save()?;
        info!("Configuration saved to .rocket.json");
    } else {
        error!("Could not find Unreal Engine 4.27 HTML5 ES3 root.");
        return Err(anyhow::anyhow!("UE4 root not found"));
    }

    Ok(())
}

fn find_ue4_root(config: &RocketConfig) -> anyhow::Result<Option<PathBuf>> {
    let mut candidates = Vec::new();

    // 1. Check config
    if let Some(ref root) = config.ue4_root {
        if validate_ue4_root(root) {
            candidates.push(root.clone());
        }
    }

    // 2. Check environment variable
    if let Ok(env_root) = env::var("UE4_ROOT") {
        let path = PathBuf::from(env_root);
        if validate_ue4_root(&path) && !candidates.contains(&path) {
            candidates.push(path);
        }
    }

    // 3. Search common locations
    let common_paths = if cfg!(windows) {
        vec![
            PathBuf::from("UnrealEngine-HTML5-ES3"),
            PathBuf::from("C:\\Program Files\\Epic Games\\UE_4.27"),
            PathBuf::from("D:\\ue-engines\\4.27-html\\myengine"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("UnrealEngine-HTML5-ES3"),
            PathBuf::from("/Users/Shared/Epic Games/UE_4.27"),
        ]
    } else {
        // Linux
        vec![
            PathBuf::from("UnrealEngine-HTML5-ES3"),
            PathBuf::from("/opt/UnrealEngine"),
        ]
    };

    for path in common_paths {
        if validate_ue4_root(&path) && !candidates.contains(&path) {
            candidates.push(path.clone());
        }

        // Also check relative to project root
        if let Ok(cwd) = env::current_dir() {
            let abs_path = cwd.join(&path);
            if validate_ue4_root(&abs_path) && !candidates.contains(&abs_path) {
                candidates.push(abs_path);
            }
        }
    }

    if !candidates.is_empty() {
        let mut options: Vec<String> = candidates
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        options.push("Enter path manually...".to_string());

        let selection =
            Select::new("Select Unreal Engine 4.27 HTML5 ES3 root:", options).prompt()?;

        if selection == "Enter path manually..." {
            prompt_manual_path()
        } else {
            Ok(Some(PathBuf::from(selection)))
        }
    } else {
        prompt_manual_path()
    }
}

fn prompt_manual_path() -> anyhow::Result<Option<PathBuf>> {
    let input =
        Text::new("Please enter the path to your Unreal Engine 4.27 HTML5 ES3 root:").prompt()?;

    let path = PathBuf::from(input);
    if validate_ue4_root(&path) {
        Ok(Some(path))
    } else {
        warn!(
            "Provided path does not seem to contain a valid RunUAT script: {:?}",
            path
        );
        let confirm = Confirm::new("Use this path anyway?")
            .with_default(false)
            .prompt()?;

        if confirm {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
}

fn validate_ue4_root(path: &Path) -> bool {
    let uat_name = if cfg!(windows) {
        "RunUAT.bat"
    } else {
        "RunUAT.sh"
    };
    let uat_path = path
        .join("Engine")
        .join("Build")
        .join("BatchFiles")
        .join(uat_name);
    uat_path.exists()
}
