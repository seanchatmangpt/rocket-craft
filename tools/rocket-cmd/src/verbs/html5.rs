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

    let cook = rocket_sdk::Html5Cook::new(&engine, &uproject, &archive_dir)
        .with_client_config(config.unwrap_or_else(|| "Development".to_string()));
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

fn do_html5_verify(archive: Option<String>, min_mb: Option<f64>) -> Result<Value> {
    let dir = archive.unwrap_or_else(|| "/tmp/brm-html5-archive/HTML5".to_string());
    let min_bytes = min_mb
        .map(|mb| (mb * 1_048_576.0) as u64)
        .unwrap_or(10 * 1024 * 1024); // default 10 MB

    let mut verifier = rocket_sdk::Html5PackageVerifier::new(&dir);
    verifier.min_wasm_bytes = min_bytes;

    let report = verifier.verify()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;

    let verdict = if report.is_real_package { "PASS" } else { "FAIL" };
    println!("[{verdict}] {}", report.summary());

    let wasm_list: Vec<serde_json::Value> = report.wasm_files.iter().map(|f| {
        let (verdict_str, size) = match &f.verdict {
            rocket_sdk::WasmVerdict::Real { size_bytes } => ("real", *size_bytes),
            rocket_sdk::WasmVerdict::Stub { size_bytes } => ("stub", *size_bytes),
            rocket_sdk::WasmVerdict::NotWasm { .. } => ("not_wasm", 0),
            rocket_sdk::WasmVerdict::Unreadable { .. } => ("unreadable", 0),
        };
        serde_json::json!({
            "path": f.path.display().to_string(),
            "verdict": verdict_str,
            "size_bytes": size,
        })
    }).collect();

    Ok(serde_json::json!({
        "verdict": verdict,
        "is_real_package": report.is_real_package,
        "summary": report.summary(),
        "archive_dir": dir,
        "wasm_files": wasm_list,
        "companions": {
            "has_js": report.companions.has_js,
            "has_html": report.companions.has_html,
            "has_data_or_pak": report.companions.has_data_or_pak,
        },
    }))
}

/// Verify an HTML5 package directory contains a real UE4 WASM build
///
/// Checks WASM magic bytes, minimum file size (stub detection), and
/// companion files (.js, .html, .data/.pak).
///
/// # Arguments
/// * `archive` - Directory to verify (default: /tmp/brm-html5-archive/HTML5)
/// * `min_mb`  - Minimum WASM size in MB to count as real (default: 10.0)
#[verb("verify", "html5")]
fn verify_html5(archive: Option<String>, min_mb: Option<f64>) -> Result<Value> {
    do_html5_verify(archive, min_mb)
}

fn do_html5_status() -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // 1. Engine presence
    let engine = ue4_root();
    let uat = engine.join("Engine/Build/BatchFiles/RunUAT.sh");
    let engine_ok = uat.exists();

    // 2. emsdk presence
    let emsdk = engine.join("Engine/Platforms/HTML5/Build/emsdk");
    let emsdk_ok = emsdk.exists();

    // 3. Package verification
    let archive = "/tmp/brm-html5-archive/HTML5";
    let pkg_report = rocket_sdk::Html5PackageVerifier::new(archive)
        .verify()
        .ok();

    let pkg_verdict = pkg_report
        .as_ref()
        .map(|r| if r.is_real_package { "REAL" } else { "STUB" })
        .unwrap_or("MISSING");

    let wasm_mb = pkg_report.as_ref().and_then(|r| {
        r.wasm_files.iter().find_map(|f| {
            if let rocket_sdk::WasmVerdict::Real { size_bytes } = f.verdict {
                Some(size_bytes as f64 / 1_048_576.0)
            } else {
                None
            }
        })
    });

    // 4. Serve port availability
    let port_free = std::net::TcpListener::bind("0.0.0.0:8080").is_ok();

    // 5. Manifest projects
    let manifest_result = rocket_sdk::RocketContext::load(&root);
    let (total_projects, present_projects) = match &manifest_result {
        Ok(ctx) => {
            let total = ctx.projects().len();
            let present = ctx.projects().iter().filter(|p| p.absolute_uproject_path().exists()).count();
            (total, present)
        }
        Err(_) => (0, 0),
    };

    let overall = if engine_ok && pkg_verdict == "REAL" { "READY" } else { "NOT READY" };

    println!("=== HTML5 Pipeline Status ===");
    println!("[{}] Engine: {}", if engine_ok { "PASS" } else { "FAIL" }, engine.display());
    println!("[{}] emsdk: {}", if emsdk_ok { "PASS" } else { "WARN" }, emsdk.display());
    println!("[{}] Package: {} ({})", pkg_verdict, archive,
        wasm_mb.map(|mb| format!("{mb:.1} MB")).unwrap_or_else(|| "n/a".into()));
    println!("[{}] Port 8080: {}", if port_free { "FREE" } else { "IN USE" }, if port_free { "available for serve" } else { "already bound" });
    println!("[INFO] Projects: {present_projects}/{total_projects} present on disk");
    println!("\n[{overall}] Pipeline is {}", overall.to_lowercase());

    Ok(serde_json::json!({
        "overall": overall,
        "engine": {
            "root": engine.display().to_string(),
            "uat_present": engine_ok,
            "emsdk_present": emsdk_ok,
        },
        "package": {
            "archive": archive,
            "verdict": pkg_verdict,
            "wasm_mb": wasm_mb,
        },
        "port_8080_free": port_free,
        "manifest": {
            "total_projects": total_projects,
            "present_projects": present_projects,
        },
    }))
}

/// Show the current state of the HTML5 pipeline in one shot
///
/// Reports: engine root, emsdk, last cooked package verdict, serve port availability,
/// and project manifest presence. Use before running `html5 cook` or `html5 serve`.
#[verb("status", "html5")]
fn status_html5() -> Result<Value> {
    do_html5_status()
}

fn do_html5_preflight(project: Option<String>) -> Result<Value> {
    use std::path::Path;

    let mut checks: Vec<serde_json::Value> = Vec::new();
    let mut overall_ok = true;

    let mut add = |name: &str, ok: bool, msg: String| {
        let status = if ok { "PASS" } else { "FAIL" };
        println!("[{status}] {name}: {msg}");
        checks.push(serde_json::json!({ "name": name, "status": status, "message": msg }));
        if !ok { overall_ok = false; }
    };

    // 1. Engine root
    let engine = ue4_root();
    let uat = engine.join("Engine/Build/BatchFiles/RunUAT.sh");
    add("Engine Root", uat.exists(), format!("{} — RunUAT.sh {}", engine.display(),
        if uat.exists() { "present" } else { "MISSING" }));

    // 2. emsdk
    let emsdk = engine.join("Engine/Platforms/HTML5/Build/emsdk");
    add("emsdk", emsdk.exists(), format!("{}", emsdk.display()));

    // 3. Python 3
    let python_ok = std::process::Command::new("python3")
        .arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    add("Python 3", python_ok, if python_ok { "python3 in PATH".into() } else { "python3 not found".into() });

    // 4. Disk space (require ≥ 50 GB free in /tmp for archive)
    let free_gb = {
        use std::process::Command;
        Command::new("df").args(["-g", "/tmp"]).output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                s.lines().nth(1).and_then(|l| l.split_whitespace().nth(3).and_then(|n| n.parse::<u64>().ok()))
            })
            .unwrap_or(0)
    };
    add("Disk Space (/tmp)", free_gb >= 50,
        format!("{free_gb} GB free in /tmp (need ≥ 50 GB)"));

    // 5. Project .uproject file
    if let Some(proj_name) = &project {
        let root = std::env::current_dir()
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
        let ctx = rocket_sdk::RocketContext::load(&root).ok();
        let uproject_ok = ctx.as_ref()
            .and_then(|c| c.manifest.projects().iter().find(|p| &p.name == proj_name).map(|p| root.join(&p.uproject_path).exists()))
            .unwrap_or(false);
        add(&format!("Project '{proj_name}'"), uproject_ok,
            if uproject_ok { format!("{proj_name}.uproject found") } else { format!("{proj_name} not found in manifest") });
    }

    // 6. arch -x86_64 (Rosetta required for UE4 HTML5 on Apple Silicon)
    let arch_ok = std::process::Command::new("arch")
        .args(["-x86_64", "true"]).status().map(|s| s.success()).unwrap_or(false);
    add("Rosetta (arch -x86_64)", arch_ok,
        if arch_ok { "Rosetta present".into() } else { "Rosetta not available — required on Apple Silicon".into() });

    let verdict = if overall_ok { "READY" } else { "NOT READY" };
    println!("\n[{verdict}] Preflight complete");

    Ok(serde_json::json!({
        "verdict": verdict,
        "all_pass": overall_ok,
        "checks": checks,
    }))
}

/// Run preflight checks before starting an HTML5 cook
///
/// Verifies: engine root + RunUAT.sh, emsdk, Python 3, disk space (≥50 GB),
/// project .uproject presence, and Rosetta (arch -x86_64) on Apple Silicon.
/// Run this before `html5 cook` to catch blockers early.
///
/// # Arguments
/// * `project` - Optional project name to verify .uproject exists
#[verb("preflight", "html5")]
fn preflight_html5(project: Option<String>) -> Result<Value> {
    do_html5_preflight(project)
}

/// Open the served HTML5 game in the default browser
///
/// Requires `html5 serve` to already be running. Opens the first `.html`
/// file found in the archive directory in the system browser.
///
/// # Arguments
/// * `archive` - Package directory (default: /tmp/brm-html5-archive/HTML5)
/// * `port` - Port the server is listening on (default: 8080)
#[verb("open", "html5")]
fn open_html5(archive: Option<String>, port: Option<u16>) -> Result<Value> {
    let dir = archive.unwrap_or_else(|| "/tmp/brm-html5-archive/HTML5".to_string());
    let port = port.unwrap_or(8080);

    // Find the first .html file in the archive dir
    let html_file = std::fs::read_dir(&dir)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|e| {
                let e = e.ok()?;
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".html") { Some(name) } else { None }
            })
        });

    let url = match html_file {
        Some(file) => format!("http://localhost:{port}/{file}"),
        None => format!("http://localhost:{port}/"),
    };

    println!("Opening: {url}");

    let status = std::process::Command::new("open")
        .arg(&url)
        .status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Failed to open browser — try opening the URL manually".to_string(),
        ));
    }

    Ok(serde_json::json!({ "url": url }))
}

fn do_html5_log(lines: Option<u32>) -> Result<Value> {
    // UAT cook logs land in ~/ue4-cook*.log — find the most recent one
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let log_dir = std::path::Path::new(&home);

    let mut candidates: Vec<(std::time::SystemTime, std::path::PathBuf)> = std::fs::read_dir(log_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let e = e.ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("ue4-cook") && name.ends_with(".log") {
                let mtime = e.metadata().ok()?.modified().ok()?;
                Some((mtime, e.path()))
            } else {
                None
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.0.cmp(&a.0));

    let log_path = candidates
        .into_iter()
        .map(|(_, p)| p)
        .next()
        .ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(
                "No ue4-cook*.log found in $HOME — start a cook first".to_string(),
            )
        })?;

    let n = lines.unwrap_or(50);
    println!("[log] {}", log_path.display());

    let status = std::process::Command::new("tail")
        .args(["-n", &n.to_string()])
        .arg(&log_path)
        .status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if !status.success() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "tail exited non-zero".to_string(),
        ));
    }

    Ok(serde_json::json!({
        "log": log_path.display().to_string(),
        "lines": n,
    }))
}

/// Show the tail of the most recent HTML5 cook log
///
/// Finds the latest ue4-cook*.log in $HOME and prints the last N lines.
/// Use this to check cook progress or diagnose a failure without hunting
/// for the log file path.
///
/// # Arguments
/// * `lines` - Number of tail lines to show (default: 50)
#[verb("log", "html5")]
fn log_html5(lines: Option<u32>) -> Result<Value> {
    do_html5_log(lines)
}
