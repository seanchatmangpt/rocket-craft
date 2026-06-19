use crate::errors::GenieError;
use crate::spec::WorldSpec;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Manages deployment logging and operations for manufactured worlds.
pub struct DeploymentManager;

impl DeploymentManager {
    /// Executes the headless UE4 pipeline to build the HTML5 world from the generated .t3d artifact.
    pub fn deploy(spec: &WorldSpec, log_path: &Path) -> Result<(), GenieError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| GenieError::Deployment(format!("Failed to open log file: {}", e)))?;

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        writeln!(file, "Genie 26 Deployment Log")
            .map_err(|e| GenieError::Deployment(e.to_string()))?;
        writeln!(file, "Timestamp MS: {}", timestamp_ms)
            .map_err(|e| GenieError::Deployment(e.to_string()))?;
        writeln!(file, "Places: {}", spec.places.len())
            .map_err(|e| GenieError::Deployment(e.to_string()))?;
        writeln!(file, "Actors: {}", spec.actors.len())
            .map_err(|e| GenieError::Deployment(e.to_string()))?;
        for place in &spec.places {
            writeln!(file, "Place: {}", place.id)
                .map_err(|e| GenieError::Deployment(e.to_string()))?;
        }
        writeln!(file, "Initiating Headless Unreal Engine 4 Build...")
            .map_err(|e| GenieError::Deployment(e.to_string()))?;

        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let ue4_root = std::env::var("UE4_ROOT")
            .unwrap_or_else(|_| "/Applications/UnrealEngine-4.27".to_string());

        let t3d_path = project_root.join("map.t3d");
        let project_uproject = project_root
            .join("versions")
            .join("v4_27_0")
            .join("Brm.uproject");
        let destination_dir = project_root.join("pwa-staff").join("manufactured");

        // Use unify-wasm packager instead of bash script
        let packager =
            unify_wasm::packager::WasmPackager::new(PathBuf::from(ue4_root), project_uproject);

        let options = unify_wasm::packager::PackageOptions {
            source_t3d: t3d_path.clone(),
            destination_dir: destination_dir.clone(),
            map_name: "ManufacturedWorld".to_string(),
        };

        let status = packager.package_html5(&options);
        if status.is_err() {
            writeln!(file, "Pipeline Status: FAILED")
                .map_err(|e| GenieError::Deployment(e.to_string()))?;
            return Err(GenieError::Deployment(
                "Headless UE4 HTML5 Pipeline failed. Please check UE4_ROOT and logs.".to_string(),
            ));
        }

        let status_win = packager.package_windows(&options);
        if status_win.is_err() {
            writeln!(file, "Pipeline Status: FAILED")
                .map_err(|e| GenieError::Deployment(e.to_string()))?;
            return Err(GenieError::Deployment(
                "Headless UE4 Win64 Pipeline failed. Please check UE4_ROOT and logs.".to_string(),
            ));
        }

        let status_linux = packager.package_linux(&options);
        if status_linux.is_err() {
            writeln!(file, "Pipeline Status: FAILED")
                .map_err(|e| GenieError::Deployment(e.to_string()))?;
            return Err(GenieError::Deployment(
                "Headless UE4 Linux Pipeline failed. Please check UE4_ROOT and logs.".to_string(),
            ));
        }

        // Record a receipt
        let receipt_file = destination_dir.join("receipt.json");
        let receipt_json = format!(
            r#"{{
  "status": "success",
  "engine": "UE4.27-ES3",
  "timestamp": "{}",
  "artifact": "{}",
  "url": "/manufactured/Brm-HTML5-Shipping.html"
}}"#,
            timestamp_ms / 1000,
            t3d_path.display()
        );

        std::fs::write(&receipt_file, receipt_json)
            .map_err(|e| GenieError::Deployment(e.to_string()))?;

        // Write spec.json
        let spec_json_path = destination_dir.join("spec.json");
        let spec_json = serde_json::to_string_pretty(spec)
            .map_err(|e| GenieError::Deployment(format!("Failed to serialize spec: {}", e)))?;
        std::fs::write(&spec_json_path, spec_json)
            .map_err(|e| GenieError::Deployment(format!("Failed to write spec.json: {}", e)))?;

        writeln!(file, "Pipeline Status: SUCCESS")
            .map_err(|e| GenieError::Deployment(e.to_string()))?;

        tracing::info!("World Factory completed. A Playable packaged browser world is now available in pwa-staff/manufactured.");
        tracing::info!("Launch a local server in pwa-staff/manufactured to enter the world.");

        Ok(())
    }
}
