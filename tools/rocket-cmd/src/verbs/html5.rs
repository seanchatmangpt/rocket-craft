//! HTML5/WebGL2 pipeline commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn ue4_root() -> std::path::PathBuf {
    if let Ok(v) = std::env::var("UE4_ROOT") {
        return std::path::PathBuf::from(v);
    }
    // Canonical location for the html5-es3 source build
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join("ue-4.27-html5-es3")
}

fn do_html5_setup() -> Result<Value> {
    let engine = ue4_root();
    rocket_sdk::Html5Setup::new(&engine)
        .run()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;
    Ok(serde_json::json!({"status": "ok", "engine": engine.display().to_string()}))
}

fn do_html5_serve(dir: Option<String>, port: Option<u16>) -> Result<Value> {
    use std::path::PathBuf;
    use std::process::Command;

    let dir = dir.unwrap_or_else(|| "/tmp/brm-html5-archive/HTML5".to_string());
    let port = port.unwrap_or(8080);
    let path = PathBuf::from(&dir);
    if !path.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "HTML5 package directory not found: {dir}"
        )));
    }
    println!("Serving {dir} on http://0.0.0.0:{port}");
    let status = Command::new("python3")
        .args(["-m", "http.server", &port.to_string(), "--bind", "0.0.0.0"])
        .current_dir(&path)
        .status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "http.server exited with {status}"
        )));
    }
    Ok(serde_json::json!({"status": "ok"}))
}

fn do_html5_cook(
    project: String,
    archive: Option<String>,
    config: Option<String>,
) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    let ctx = rocket_sdk::RocketContext::load(&root)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    let proj = ctx
        .manifest
        .projects()
        .iter()
        .find(|p| p.name == project)
        .ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "project '{project}' not found in project-manifest.json"
            ))
        })?;

    let uproject = root.join(&proj.uproject_path);
    let archive_dir = archive
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/brm-html5-archive"));
    let engine = ue4_root();

    println!(
        "HTML5 cook: {} → {}",
        uproject.display(),
        archive_dir.display()
    );
    println!("Engine: {}", engine.display());

    let mut cook = rocket_sdk::Html5Cook::new(&engine, &uproject, &archive_dir);
    cook.client_config = config.unwrap_or_else(|| "Development".to_string());
    cook.run()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;

    Ok(serde_json::json!({
        "status": "ok",
        "project": proj.name,
        "archive": archive_dir.display().to_string(),
    }))
}

/// Build third-party HTML5 libraries via emscripten
#[verb("setup", "html5")]
fn setup_html5() -> Result<Value> {
    do_html5_setup()
}

/// Serve a packaged HTML5 build over HTTP
///
/// # Arguments
/// * `dir` - Directory containing the HTML5 package (default: /tmp/brm-html5-archive/HTML5)
/// * `port` - Port to listen on (default: 8080)
#[verb("serve", "html5")]
fn serve_html5(dir: Option<String>, port: Option<u16>) -> Result<Value> {
    do_html5_serve(dir, port)
}

/// Cook + package a UE4 project for HTML5 via RunUAT BuildCookRun
///
/// # Arguments
/// * `project` - Project name as declared in project-manifest.json (e.g. Brm)
/// * `archive` - Output directory for the packaged HTML5 build (default: /tmp/brm-html5-archive)
/// * `config` - Build configuration: Development or Shipping (default: Development)
#[verb("cook", "html5")]
fn cook_html5(project: String, archive: Option<String>, config: Option<String>) -> Result<Value> {
    do_html5_cook(project, archive, config)
}
