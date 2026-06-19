//! HTML5/WebGL2 pipeline commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

/// Push a verified package report to Supabase.
///
/// Sequence:
///   1. create_cook_session → get UUID
///   2. push_to_supabase (events + receipt, tagged with UUID)
///   3. close_cook_session (is_alive=false, receipt_hash recorded)
///
/// Reads SUPABASE_URL + SUPABASE_ANON_KEY from the environment. Silently no-ops
/// when Supabase is not configured — offline cooks still write local receipts.
fn push_report_to_supabase(report: &rocket_sdk::Html5PackageReport) {
    let url = std::env::var("SUPABASE_URL").unwrap_or_default();
    let key = std::env::var("SUPABASE_ANON_KEY")
        .or_else(|_| std::env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();
    if url.is_empty() || key.is_empty() {
        return;
    }
    let svc = rocket_sdk::supabase::SupabaseService::new(url, key);
    let Ok(rt) = tokio::runtime::Runtime::new() else {
        println!("[supabase] warn: could not create tokio runtime");
        return;
    };
    rt.block_on(async {
        // Open a game_sessions row so OCEL events can be chain-verified.
        let project_name = report.archive_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let session_id = match svc.create_cook_session(
            project_name,
            &report.archive_dir.display().to_string(),
        ).await {
            Ok(id) => {
                println!("[supabase] cook session opened → {id}");
                Some(id)
            }
            Err(e) => {
                println!("[supabase] warn: create_cook_session failed (non-fatal) — {e:#}");
                None
            }
        };

        // Attach session_id before pushing events + receipt.
        let mut owned = report.clone();
        owned.cook_session_id = session_id;
        match owned.push_to_supabase(&svc).await {
            Ok(()) => println!("[supabase] cook receipt pushed → game_receipts"),
            Err(e) => println!("[supabase] warn: push failed (non-fatal) — {e:#}"),
        }
    });
}

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

// Inline Python server with COOP/COEP headers (required for SharedArrayBuffer/wasm-threads).
// This is a background ASSET server only — Nuxt shell is the entry point, not this server.
// The Nuxt dev server proxies /manufactured/** → this port.
const COEP_SERVER_SCRIPT: &str = r#"
import http.server, sys

class CoepHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()
    def log_message(self, fmt, *args):
        pass  # suppress per-request noise

port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
print(f"[asset-server] UE4 assets on http://0.0.0.0:{port} (proxied via Nuxt /manufactured/)", flush=True)
with http.server.HTTPServer(("0.0.0.0", port), CoepHandler) as httpd:
    httpd.serve_forever()
"#;

fn pid_file_for_port(port: u16) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/tmp/rocket-html5-serve-{port}.pid"))
}

fn do_html5_serve(dir: Option<String>, port: Option<u16>, project: Option<String>, background: bool) -> Result<Value> {
    use std::path::PathBuf;
    use std::process::Command;

    let dir = dir.unwrap_or_else(|| {
        let name = project.as_deref().unwrap_or("brm").to_lowercase();
        format!("/tmp/{name}-html5-archive/HTML5")
    });
    let port = port.unwrap_or(8080);
    let path = PathBuf::from(&dir);
    if !path.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "HTML5 package directory not found: {dir}"
        )));
    }

    // Verify python3 is available before attempting to spawn it — gives a clear error
    // instead of a cryptic "No such file or directory" from spawn().
    let python_ok = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !python_ok {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "python3 not found in PATH — required for COOP/COEP serve. Install python3 and retry.".to_string()
        ));
    }

    println!("Serving {dir} on http://0.0.0.0:{port} (COOP/COEP headers enabled)");

    if background {
        let child = Command::new("python3")
            .args(["-c", COEP_SERVER_SCRIPT, &port.to_string()])
            .current_dir(&path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
        let pid = child.id();
        let pid_file = pid_file_for_port(port);
        std::fs::write(&pid_file, pid.to_string())
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("could not write PID file: {e}")))?;
        println!("[background] PID {pid} — stop with: rocket html5 stop --port {port}");
        return Ok(serde_json::json!({ "status": "background", "pid": pid, "pid_file": pid_file.display().to_string() }));
    }

    let status = Command::new("python3")
        .args(["-c", COEP_SERVER_SCRIPT, &port.to_string()])
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
        .unwrap_or_else(|| std::path::PathBuf::from(format!("/tmp/{}-html5-archive", proj.name.to_lowercase())));
    let engine = ue4_root();

    let cook = rocket_sdk::Html5Cook::new(&engine, &uproject, &archive_dir)
        .with_client_config(config.unwrap_or_else(|| "Development".to_string()));

    // Use Html5Cook::preflight_check() to surface all blockers before wasting 30 min on UAT
    let blockers = cook.preflight_check();
    if !blockers.is_empty() {
        let msg = format!(
            "Cook preflight failed for '{}' — fix these issues first:\n{}",
            project,
            blockers.iter().map(|b| format!("  • {b}")).collect::<Vec<_>>().join("\n")
        );
        return Err(clap_noun_verb::NounVerbError::execution_error(msg));
    }

    println!(
        "HTML5 cook: {} → {}",
        uproject.display(),
        archive_dir.display()
    );
    println!("Engine: {}", engine.display());
    cook.run()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;

    // Auto-verify and write cook receipt after a successful cook
    let archive_html5 = archive_dir.join("HTML5");
    let verify_dir = if archive_html5.exists() { &archive_html5 } else { &archive_dir };
    let (pkg_verdict, receipt_path) = match rocket_sdk::Html5PackageVerifier::new(verify_dir).verify() {
        Ok(report) => {
            let v = if report.is_real_package { "PASS" } else { "FAIL" };
            println!("[{}] {}", v, report.summary());
            let rp = report.write_receipt().ok().map(|p| p.display().to_string());
            if let Some(ref p) = rp { println!("[receipt] {p}"); }
            push_report_to_supabase(&report);
            (v.to_string(), rp)
        }
        Err(e) => {
            println!("[WARN] post-cook verify failed: {e:#}");
            ("UNKNOWN".to_string(), None)
        }
    };

    Ok(serde_json::json!({
        "status": "ok",
        "project": proj.name,
        "archive": archive_dir.display().to_string(),
        "verify_verdict": pkg_verdict,
        "receipt": receipt_path,
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
/// * `background` - Daemonize the server and write a PID file; use `html5 stop` to kill it
#[verb("serve", "html5")]
fn serve_html5(dir: Option<String>, port: Option<u16>, project: Option<String>, background: Option<bool>) -> Result<Value> {
    do_html5_serve(dir, port, project, background.unwrap_or(false))
}

/// Cook + package a UE4 project for HTML5 via RunUAT BuildCookRun
///
/// # Arguments
/// * `project` - Project name as declared in project-manifest.json (e.g. Brm)
/// * `archive` - Output directory for the packaged HTML5 build (default: /tmp/brm-html5-archive)
/// * `config` - Build configuration: Development or Shipping (default: Development)
/// * `dry_run` - Print the UAT command that would be run without executing it
#[verb("cook", "html5")]
fn cook_html5(project: String, archive: Option<String>, config: Option<String>, dry_run: Option<bool>) -> Result<Value> {
    if dry_run.unwrap_or(false) {
        return do_html5_cook_dry_run(project, archive, config);
    }
    do_html5_cook(project, archive, config)
}

fn do_html5_cook_dry_run(project: String, archive: Option<String>, config: Option<String>) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    let ctx = rocket_sdk::RocketContext::load(&root)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
    let proj = ctx.manifest.projects().iter()
        .find(|p| p.name == project)
        .ok_or_else(|| clap_noun_verb::NounVerbError::execution_error(format!("project '{project}' not found")))?;
    let uproject = root.join(&proj.uproject_path);
    let archive_dir = archive
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(format!("/tmp/{}-html5-archive", proj.name.to_lowercase())));
    let engine = ue4_root();

    let cook = rocket_sdk::Html5Cook::new(&engine, &uproject, &archive_dir)
        .with_client_config(config.unwrap_or_else(|| "Development".to_string()));
    let args = cook.command_args();
    let cmd_str = args.join(" \\\n  ");
    println!("[dry-run] Would execute:\n  {cmd_str}");
    Ok(serde_json::json!({ "dry_run": true, "command": args }))
}

/// Patch a UE4-generated HTML file to suppress pointer lock so UMG widget
/// clicks reach UE4's input system on first press (no "focus-capture" eat).
fn patch_html_for_ui_input(html_dir: &std::path::Path) {
    let inject = r#"
<script>
// rocket-html5: suppress UE4 pointer lock so UMG widget clicks register immediately.
(function(){
  var noop=function(){return Promise.resolve();};
  try{Object.defineProperty(HTMLElement.prototype,'requestPointerLock',{value:noop,writable:false,configurable:false});}catch(e){HTMLElement.prototype.requestPointerLock=noop;}
  try{Object.defineProperty(Document.prototype,'exitPointerLock',{value:noop,writable:false,configurable:false});}catch(e){document.exitPointerLock=noop;}
  var _p=setInterval(function(){var c=document.getElementById('canvas');if(c&&c.style.display!=='none'){c.focus();clearInterval(_p);}},500);
  document.addEventListener('click',function(){var c=document.getElementById('canvas');if(c)c.focus();});
})();
</script>
"#;
    let html_files: Vec<_> = std::fs::read_dir(html_dir)
        .ok().into_iter().flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
        .map(|e| e.path())
        .collect();
    for path in html_files {
        if let Ok(src) = std::fs::read_to_string(&path) {
            if src.contains("rocket-html5: suppress") {
                continue; // already patched
            }
            let patched = src.replace("</body>", &format!("{inject}</body>"));
            if std::fs::write(&path, patched).is_ok() {
                println!("[patch] UI input fix applied → {}", path.display());
            }
        }
    }
}

fn do_html5_verify(archive: Option<String>, min_mb: Option<f64>, project: Option<String>) -> Result<Value> {
    let dir = archive.unwrap_or_else(|| {
        let name = project.as_deref().unwrap_or("brm").to_lowercase();
        format!("/tmp/{name}-html5-archive/HTML5")
    });
    let min_bytes = min_mb
        .map(|mb| (mb * 1_048_576.0) as u64)
        .unwrap_or(10 * 1024 * 1024); // default 10 MB

    // Patch UE4-generated HTML to suppress pointer-lock so UMG widget clicks work.
    patch_html_for_ui_input(std::path::Path::new(&dir));

    let mut verifier = rocket_sdk::Html5PackageVerifier::new(&dir);
    verifier.min_wasm_bytes = min_bytes;

    let report = verifier.verify()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{:#}", e)))?;

    let verdict = if report.is_real_package { "PASS" } else { "FAIL" };
    println!("[{verdict}] {}", report.summary());

    // Write cook-receipt.json alongside the archive for pipeline traceability
    match report.write_receipt() {
        Ok(path) => println!("[receipt] {}", path.display()),
        Err(e) => println!("[receipt] warning: could not write receipt — {e:#}"),
    }

    // Push to Supabase if configured — non-fatal (cook is already done)
    push_report_to_supabase(&report);

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
fn verify_html5(archive: Option<String>, min_mb: Option<f64>, project: Option<String>) -> Result<Value> {
    do_html5_verify(archive, min_mb, project)
}

fn do_html5_status(project: Option<String>) -> Result<Value> {
    let root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    // 1. Engine presence
    let engine = ue4_root();
    let uat = engine.join("Engine/Build/BatchFiles/RunUAT.sh");
    let engine_ok = uat.exists();

    // 2. emsdk presence
    let emsdk = engine.join("Engine/Platforms/HTML5/Build/emsdk");
    let emsdk_ok = emsdk.exists();

    // 3. Package verification — derive archive path from project name if given
    let project_name = project.as_deref().unwrap_or("brm");
    let archive = format!("/tmp/{}-html5-archive/HTML5", project_name.to_lowercase());
    let archive = archive.as_str();
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

    // 4. Serve port availability + background serve PID
    let port_free = std::net::TcpListener::bind("0.0.0.0:8080").is_ok();
    let pid_file = pid_file_for_port(8080);
    let background_pid: Option<u32> = std::fs::read_to_string(&pid_file)
        .ok()
        .and_then(|s| s.trim().parse().ok());

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

    // 6. Most recent cook log
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let cook_log = {
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
        candidates.into_iter().map(|(_, p)| p).next()
    };

    // 7. Cook receipt presence
    let receipt_path = std::path::Path::new(archive).join("cook-receipt.json");
    let receipt = if receipt_path.exists() {
        std::fs::read_to_string(&receipt_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
    } else {
        None
    };
    let receipt_verdict = receipt.as_ref()
        .and_then(|v| v["verdict"].as_str().map(str::to_string))
        .unwrap_or_else(|| "NONE".to_string());

    // 8. Nuxt shell status (port 3000) — the real game entry point
    let nuxt_port = 3000u16;
    let nuxt_up = std::net::TcpStream::connect(format!("127.0.0.1:{nuxt_port}")).is_ok();
    let nuxt_game_url = format!("http://localhost:{nuxt_port}/game");

    let overall = if engine_ok && pkg_verdict == "REAL" && nuxt_up { "READY" }
        else if engine_ok && pkg_verdict == "REAL" { "ASSETS READY — start Nuxt shell" }
        else { "NOT READY" };

    println!("=== HTML5 Pipeline Status ===");
    println!("[{}] Engine: {}", if engine_ok { "PASS" } else { "FAIL" }, engine.display());
    println!("[{}] emsdk: {}", if emsdk_ok { "PASS" } else { "WARN" }, emsdk.display());
    println!("[{}] Package: {} ({})", pkg_verdict, archive,
        wasm_mb.map(|mb| format!("{mb:.1} MB")).unwrap_or_else(|| "n/a".into()));
    println!("[{}] Receipt: {}", receipt_verdict, if receipt.is_some() { receipt_path.display().to_string() } else { "not found (run 'rocket html5 verify')".into() });
    println!("[{}] UE4 asset server port 8080: {}{}",
        if port_free { "FREE" } else { "SERVING" },
        if port_free { "start with: rocket html5 serve --project Brm" } else { "running" },
        background_pid.map(|pid| format!(" (PID {pid})")).unwrap_or_default());
    println!("[{}] Nuxt shell port {nuxt_port}: {}",
        if nuxt_up { "UP" } else { "DOWN" },
        if nuxt_up { nuxt_game_url.as_str() } else { "start with: cd nuxt-shell && pnpm dev" });
    println!("[INFO] Projects: {present_projects}/{total_projects} present on disk");
    if let Some(ref log) = cook_log {
        println!("[INFO] Last cook log: {}  (run 'rocket html5 log' to tail)", log.display());
    }
    println!("\n[{overall}]");
    if nuxt_up {
        println!("  Game URL: {nuxt_game_url}");
    }

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
        "receipt": {
            "path": receipt_path.display().to_string(),
            "verdict": receipt_verdict,
            "present": receipt.is_some(),
        },
        "nuxt_shell": {
            "port": nuxt_port,
            "up": nuxt_up,
            "game_url": if nuxt_up { Some(nuxt_game_url) } else { None },
        },
        "ue4_asset_server": {
            "port": 8080,
            "port_free": port_free,
            "background_pid": background_pid,
        },
        "manifest": {
            "total_projects": total_projects,
            "present_projects": present_projects,
        },
        "cook_log": cook_log.as_ref().map(|p| p.display().to_string()),
    }))
}

/// Show the current state of the HTML5 pipeline in one shot
///
/// Reports: engine root, emsdk, last cooked package verdict, serve port availability,
/// and project manifest presence. Use before running `html5 cook` or `html5 serve`.
#[verb("status", "html5")]
fn status_html5(project: Option<String>) -> Result<Value> {
    do_html5_status(project)
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

    // 7. emsdk bundled Python — path: <emsdk>/emsdk-*/python/<ver>/bin/python3
    //    UBT reads PYTHON env var first; this is what HTML5Setup.sh activates via
    //    `emsdk activate`. Verify it exists so cook doesn't fail silently.
    let emsdk_python = {
        walkdir::WalkDir::new(&emsdk)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| {
                let name = e.file_name().to_string_lossy();
                name == "python3" && e.path().parent().map(|p| p.ends_with("bin")).unwrap_or(false)
            })
            .map(|e| e.into_path())
    };
    let emsdk_python_ok = emsdk_python.as_ref().map(|p| p.exists()).unwrap_or(false);
    add("emsdk Python", emsdk_python_ok,
        emsdk_python.as_ref().map(|p| format!("{}", p.display()))
            .unwrap_or_else(|| "not found — emsdk may need setup: run 'rocket html5 setup'".into()));

    // 8. UHT symlink — UHT computes CURRENT_FILE_ID from the directory name.
    //    If the project is in versions/4.27.0/, UHT produces invalid C macros (start with digit).
    //    Html5Cook auto-creates the symlink, but preflight can verify it already exists.
    //    Only check when a project is specified.
    if let Some(proj_name) = &project {
        let root = std::env::current_dir()
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;
        let ctx = rocket_sdk::RocketContext::load(&root).ok();
        if let Some(uproject_path) = ctx.as_ref()
            .and_then(|c| c.manifest.projects().iter().find(|p| &p.name == proj_name)
                .map(|p| root.join(&p.uproject_path)))
        {
            // Check if parent dir starts with digit, and if so whether the symlink exists
            let parent_name = uproject_path.parent()
                .and_then(|d| d.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if parent_name.starts_with(|c: char| c.is_ascii_digit()) {
                // Look for a sibling symlink that starts with a letter
                let sibling_dir = uproject_path.parent()
                    .and_then(|d| d.parent())
                    .unwrap_or(Path::new("."));
                let symlink_exists = std::fs::read_dir(sibling_dir)
                    .ok()
                    .map(|rd| rd.filter_map(|e| e.ok())
                        .any(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            name.starts_with(|c: char| c.is_alphabetic())
                                && e.path().is_symlink()
                        }))
                    .unwrap_or(false);
                add("UHT symlink", symlink_exists,
                    if symlink_exists {
                        format!("letter-starting symlink found in {}", sibling_dir.display())
                    } else {
                        format!("WARNING: {} starts with digit — UHT will generate invalid macros. \
                            'rocket html5 cook' creates the symlink automatically.", parent_name)
                    });
            }
        }
    }

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

/// Cross-platform browser open: macOS `open`, Linux `xdg-open`, Windows `start`.
fn open_in_browser(url: &str) -> std::result::Result<(), String> {
    let (cmd, args): (&str, Vec<&str>) = if cfg!(target_os = "macos") {
        ("open", vec![url])
    } else if cfg!(target_os = "windows") {
        ("cmd", vec!["/c", "start", url])
    } else {
        ("xdg-open", vec![url])
    };
    let status = std::process::Command::new(cmd)
        .args(&args)
        .status()
        .map_err(|e| format!("failed to spawn '{cmd}': {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("'{cmd}' exited with non-zero status — open {url} manually"))
    }
}

/// Open the Nuxt shell game in the default browser.
///
/// The Nuxt shell (port 3000, /game route) is the entry point — NOT the raw UE4 asset
/// server. This command verifies the Nuxt shell is actually responding before opening.
///
/// # Arguments
/// * `nuxt_port` - Nuxt dev server port (default: 3000)
#[verb("open", "html5")]
fn open_html5(nuxt_port: Option<u16>, project: Option<String>) -> Result<Value> {
    let _ = project; // reserved for future multi-project support
    let port = nuxt_port.unwrap_or(3000);
    let url = format!("http://localhost:{port}/game");

    // Verify the Nuxt shell is actually up before opening — never give a URL that doesn't work
    let reachable = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok();
    if !reachable {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Nuxt shell is not running on port {port}. \
            Start it with: cd nuxt-shell && pnpm dev"
        )));
    }

    println!("[open] Nuxt shell verified on port {port}");
    println!("[open] Opening: {url}");

    open_in_browser(&url)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;

    Ok(serde_json::json!({ "url": url, "verified": true }))
}

/// Send SIGTERM to a background process by PID, removing the PID file on success.
fn kill_background_serve(pid: u32, pid_file: &std::path::Path, port: u16) -> Result<Value> {
    #[cfg(unix)]
    {
        let status = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("kill failed: {e}")))?;
        if !status.success() {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!("kill non-zero for PID {pid}")));
        }
    }
    #[cfg(not(unix))]
    println!("Non-Unix: manually terminate PID {pid}");

    let _ = std::fs::remove_file(pid_file);
    println!("[stopped] HTML5 serve on port {port} (PID {pid})");
    Ok(serde_json::json!({ "stopped": true, "pid": pid, "port": port }))
}

/// Stop a background HTML5 serve process started with `html5 serve --background`
///
/// # Arguments
/// * `port` - Port the background server is running on (default: 8080)
#[verb("stop", "html5")]
fn stop_html5(port: Option<u16>) -> Result<Value> {
    let port = port.unwrap_or(8080);
    let pid_file = pid_file_for_port(port);
    if !pid_file.exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "No background serve for port {port} (PID file missing: {})", pid_file.display()
        )));
    }
    let raw = std::fs::read_to_string(&pid_file)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("read PID file: {e}")))?;
    let pid: u32 = raw.trim().parse()
        .map_err(|_| clap_noun_verb::NounVerbError::execution_error(format!("invalid PID: {raw}")))?;
    kill_background_serve(pid, &pid_file, port)
}

fn build_pipeline_result(
    project: &str, preflight: Value, cook: Value, verify: Value,
    preflight_secs: f64, cook_secs: f64, verify_secs: f64, total_secs: f64,
) -> Value {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0);
    serde_json::json!({
        "project": project,
        "preflight": preflight,
        "cook": cook,
        "verify": verify,
        "pipeline_verdict": "PASS",
        "timing_secs": { "preflight": preflight_secs, "cook": cook_secs, "verify": verify_secs, "total": total_secs },
        "completed_at_unix_secs": ts,
    })
}

fn write_pipeline_receipt(project: &str, result: &Value) {
    let dir = std::path::PathBuf::from(format!("/tmp/{}-html5-archive/HTML5", project.to_lowercase()));
    if dir.exists() {
        let path = dir.join("pipeline-receipt.json");
        if let Ok(s) = serde_json::to_string_pretty(result) {
            if std::fs::write(&path, s).is_ok() {
                println!("[receipt] {}", path.display());
            }
        }
    }
}

fn do_html5_log(lines: Option<u32>, follow: bool) -> Result<Value> {
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

    let mut cmd = std::process::Command::new("tail");
    cmd.args(["-n", &n.to_string()]);
    if follow {
        cmd.arg("-f");
        println!("[following — Ctrl-C to stop]");
    }
    cmd.arg(&log_path);

    let status = cmd
        .status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("{e}")))?;

    if !status.success() && !follow {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "tail exited non-zero".to_string(),
        ));
    }

    // Surface cook errors without requiring --follow or a full tail read
    let errors = extract_cook_errors(&log_path);
    if !errors.is_empty() {
        println!("[errors detected]");
        for e in &errors {
            println!("  {e}");
        }
    }

    Ok(serde_json::json!({
        "log": log_path.display().to_string(),
        "lines": n,
        "follow": follow,
        "cook_errors": errors,
    }))
}

/// Scan a cook log for lines that signal failure without reading the entire file.
/// Returns up to 20 matching lines (ERROR/FAILED/Fatal/PackagingError patterns).
fn extract_cook_errors(log_path: &std::path::Path) -> Vec<String> {
    use std::io::{BufRead, BufReader};
    let Ok(file) = std::fs::File::open(log_path) else { return vec![]; };
    let reader = BufReader::new(file);
    let patterns = ["Error:", "ERROR:", "FAILED:", "Fatal:", "PackagingError", "LogCook: Error"];
    reader.lines()
        .filter_map(|l| l.ok())
        .filter(|line| patterns.iter().any(|p| line.contains(p)))
        .take(20)
        .collect()
}

/// Show the tail of the most recent HTML5 cook log
///
/// Finds the latest ue4-cook*.log in $HOME and prints the last N lines.
/// Use this to check cook progress or diagnose a failure without hunting
/// for the log file path. Pass --follow to stream new lines as they arrive.
///
/// # Arguments
/// * `lines` - Number of tail lines to show (default: 50)
/// * `follow` - If true, stream new lines (like tail -f)
#[verb("log", "html5")]
fn log_html5(lines: Option<u32>, follow: Option<bool>) -> Result<Value> {
    do_html5_log(lines, follow.unwrap_or(false))
}

/// Run the full HTML5 pipeline: preflight → cook → verify in sequence.
/// Exits immediately on any failure. Equivalent to running each step manually.
///
/// # Arguments
/// * `project` - UE4 project name (e.g. Brm)
/// * `config` - Build configuration (default: Shipping)
/// * `archive` - Override the archive directory
#[verb("pipeline", "html5")]
fn pipeline_html5(project: String, config: Option<String>, archive: Option<String>) -> Result<Value> {
    let pipeline_start = std::time::Instant::now();

    // Step 1: preflight
    println!("[1/3] Running preflight checks for {}...", project);
    let t0 = std::time::Instant::now();
    let preflight = do_html5_preflight(Some(project.clone()))?;
    let preflight_secs = t0.elapsed().as_secs_f64();
    let all_pass = preflight["all_pass"].as_bool().unwrap_or(false);
    if !all_pass {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("Preflight failed for {}. Fix the above issues before cooking.", project)
        ));
    }
    println!("[1/3] Preflight PASS ({:.1}s)\n", preflight_secs);

    // Step 2: cook — default to Shipping for pipeline (production quality)
    let effective_config = config.unwrap_or_else(|| "Shipping".to_string());
    println!("[2/3] Cooking {} ({})...", project, effective_config);
    let t0 = std::time::Instant::now();
    let cook_result = do_html5_cook(project.clone(), archive.clone(), Some(effective_config))?;
    let cook_secs = t0.elapsed().as_secs_f64();
    println!("[2/3] Cook complete ({:.0}s)\n", cook_secs);

    // Step 3: verify (do_html5_cook already auto-verifies, but run explicitly for clean output)
    println!("[3/3] Verifying package...");
    let t0 = std::time::Instant::now();
    let verify = do_html5_verify(archive, None, Some(project.clone()))?;
    let verify_secs = t0.elapsed().as_secs_f64();
    let verdict = verify["verdict"].as_str().unwrap_or("UNKNOWN");
    if verdict != "PASS" {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("Package verification failed: verdict={verdict}")
        ));
    }
    println!("[3/3] Verification PASS ({:.1}s)\n", verify_secs);

    let total_secs = pipeline_start.elapsed().as_secs_f64();
    println!("[DONE] HTML5 pipeline complete for {project} ({:.0}s total)", total_secs);
    println!("  → serve + open in one shot:");
    println!("      rocket html5 serve --project {project} --background && rocket html5 open --project {project}");
    println!("  → stop the background server:");
    println!("      rocket html5 stop");

    let result = build_pipeline_result(
        &project, preflight, cook_result, verify,
        preflight_secs, cook_secs, verify_secs, total_secs,
    );
    write_pipeline_receipt(&project, &result);
    Ok(result)
}

/// Run the automated OCEL+OTel game loop test without human interaction.
///
/// Starts UE4 asset server (:8080) + Nuxt shell (:3000) if not already running,
/// then executes Playwright `game-loop.spec.ts` and reports the OCEL verdict.
///
/// # Arguments
/// * `project`  - UE4 project name for asset archive (default: Brm)
/// * `nuxt_dir` - Path to nuxt-shell/ directory
/// * `headed`   - Run Playwright with visible browser (default: headless)
#[verb("e2e", "html5")]
fn e2e_html5(project: Option<String>, nuxt_dir: Option<String>, headed: Option<bool>) -> Result<Value> {
    do_html5_e2e(project, nuxt_dir, headed)
}

fn do_html5_e2e(
    project: Option<String>,
    nuxt_dir: Option<String>,
    headed: Option<bool>,
) -> Result<Value> {
    let project = project.unwrap_or_else(|| "Brm".to_string());
    let repo_root = std::env::current_dir()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("cwd: {e}")))?;
    let nuxt_path = nuxt_dir
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| repo_root.join("nuxt-shell"));
    let headed = headed.unwrap_or(false);

    let archive = format!("/tmp/{}-html5-archive/HTML5", project.to_lowercase());
    if !std::path::Path::new(&archive).exists() {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "UE4 archive missing: {archive} — run: rocket html5 cook --project {project}"
        )));
    }

    let asset_child = start_asset_server_if_needed(&archive)?;
    let nuxt_child = start_nuxt_server_if_needed(&nuxt_path)?;

    let pw_result = run_playwright_game_loop(&nuxt_path, headed);

    drop(asset_child);
    drop(nuxt_child);

    pw_result
}

struct KillOnDrop(Option<std::process::Child>);
impl Drop for KillOnDrop {
    fn drop(&mut self) { if let Some(mut c) = self.0.take() { let _ = c.kill(); } }
}

fn start_asset_server_if_needed(archive: &str) -> Result<KillOnDrop> {
    if std::net::TcpListener::bind("0.0.0.0:8080").is_err() {
        println!("[e2e] Asset server already on :8080");
        return Ok(KillOnDrop(None));
    }
    println!("[e2e] Starting UE4 asset server on :8080...");
    let script_file = std::env::temp_dir().join("coep_server.py");
    std::fs::write(&script_file, COEP_SERVER_SCRIPT)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("write server: {e}")))?;
    let child = std::process::Command::new("python3")
        .arg(&script_file).arg("8080")
        .current_dir(archive)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("spawn asset server: {e}")))?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("[e2e] Asset server started");
    Ok(KillOnDrop(Some(child)))
}

fn start_nuxt_server_if_needed(nuxt_path: &std::path::Path) -> Result<KillOnDrop> {
    if std::net::TcpListener::bind("0.0.0.0:3000").is_err() {
        println!("[e2e] Nuxt shell already on :3000");
        return Ok(KillOnDrop(None));
    }
    println!("[e2e] Starting Nuxt shell dev server on :3000...");
    let child = std::process::Command::new("pnpm")
        .arg("dev").current_dir(nuxt_path)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("spawn nuxt: {e}")))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if std::net::TcpStream::connect("127.0.0.1:3000").is_ok() { break; }
        if std::time::Instant::now() > deadline {
            return Err(clap_noun_verb::NounVerbError::execution_error("Nuxt did not start within 60s".to_string()));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    println!("[e2e] Nuxt shell is up — settling 3s...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(KillOnDrop(Some(child)))
}

fn run_playwright_game_loop(nuxt_path: &std::path::Path, headed: bool) -> Result<Value> {
    // Headed mode runs the real-UE4 suite (no synthetic injection, long timeout).
    // Headless mode runs the fast OCEL pipeline CI suite (synthetic EngineReady, <60s).
    let (spec, config_arg, receipt_name, suite_label) = if headed {
        (
            "e2e/real-ue4-game-loop.spec.ts",
            Some("--config=playwright.ue4.config.ts"),
            "real-ue4-receipt.json",
            "real-UE4",
        )
    } else {
        (
            "e2e/game-loop.spec.ts",
            None,
            "game-loop-receipt.json",
            "OCEL-pipeline",
        )
    };

    println!("[e2e] Running Playwright {suite_label} suite (headed={headed})...");

    let mut cmd = std::process::Command::new("npx");
    let mut args = vec!["playwright", "test", spec, "--reporter=line"];
    if let Some(cfg) = config_arg { args.push(cfg); }
    if headed { args.push("--headed"); }

    let status = cmd.args(&args).current_dir(nuxt_path).status()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("playwright: {e}")))?;

    let receipt_path = nuxt_path.join(format!("playwright-report/{receipt_name}"));
    let receipt: Option<serde_json::Value> = receipt_path.exists()
        .then(|| std::fs::read_to_string(&receipt_path).ok()
            .and_then(|s| serde_json::from_str(&s).ok()))
        .flatten();
    let verdict = receipt.as_ref().and_then(|v| v["verdict"].as_str()).unwrap_or("UNKNOWN");
    let engine_source = receipt.as_ref()
        .and_then(|v| v["engine_source"].as_str())
        .unwrap_or("unknown");

    if headed && engine_source == "synthetic" {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "real-UE4 suite produced engine_source='synthetic' — \
             EngineReady was not received from the real WASM. \
             Ensure the UE4 asset server is running: rocket html5 serve --project Brm --background"
                .to_string(),
        ));
    }

    if status.success() {
        println!("[e2e] PASS — verdict={verdict} engine_source={engine_source}");
        println!("[e2e] Receipt: {}", receipt_path.display());
        Ok(serde_json::json!({
            "verdict": verdict,
            "engine_source": engine_source,
            "suite": suite_label,
            "receipt_path": receipt_path.display().to_string(),
            "receipt": receipt,
        }))
    } else {
        Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "{suite_label} FAILED verdict={verdict} — see nuxt-shell/playwright-report/"
        )))
    }
}

/// Resolve the HTML5 archive directory for a project name.
fn resolve_archive(project: &str) -> std::path::PathBuf {
    let lower = std::path::PathBuf::from(format!("/tmp/{}-html5-archive/HTML5", project.to_lowercase()));
    if lower.exists() { lower } else {
        std::path::PathBuf::from(format!("/tmp/{project}-html5-archive/HTML5"))
    }
}

/// Wasm size in MB from a package report (first Real verdict found).
fn wasm_mb_of(report: &rocket_sdk::Html5PackageReport) -> Option<f64> {
    report.wasm_files.iter().find_map(|f| {
        if let rocket_sdk::WasmVerdict::Real { size_bytes } = f.verdict {
            Some(size_bytes as f64 / 1_048_576.0)
        } else {
            None
        }
    })
}

/// Fetch the most recent cook receipt, trying Supabase first then the local JSON file.
fn fetch_prior_receipt(archive: &std::path::Path) -> Option<serde_json::Value> {
    let url = std::env::var("SUPABASE_URL").unwrap_or_default();
    let key = std::env::var("SUPABASE_ANON_KEY")
        .or_else(|_| std::env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();
    if !url.is_empty() && !key.is_empty() {
        let svc = rocket_sdk::supabase::SupabaseService::new(url, key);
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            return rt.block_on(svc.last_cook_receipt(1)).ok()
                .and_then(|mut v| v.pop());
        }
    }
    let local = archive.join("cook-receipt.json");
    if local.exists() {
        std::fs::read_to_string(&local).ok().and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    }
}

fn do_html5_diff(project: &str) -> clap_noun_verb::Result<Value> {
    let archive = resolve_archive(project);
    let current = rocket_sdk::Html5PackageVerifier::new(&archive).verify()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("verify failed: {e} — run rocket html5 verify first")))?;
    let current_verdict = if current.is_real_package { "PASS" } else { "FAIL" };
    let current_mb = wasm_mb_of(&current);
    let prior = fetch_prior_receipt(&archive);
    let prior_verdict = prior.as_ref().and_then(|p| p["verdict"].as_str()).unwrap_or("none");
    let prior_mb = prior.as_ref().and_then(|p| p["payload"]["wasm_mb"].as_f64());
    let regression = prior_verdict == "PASS" && current_verdict == "FAIL";
    let delta_mb = current_mb.zip(prior_mb).map(|(c, p)| c - p);

    println!("[html5 diff] project={project}  archive={}", archive.display());
    println!("  current : verdict={current_verdict}  wasm={current_mb:.1?} MB");
    println!("  prior   : verdict={prior_verdict}  wasm={prior_mb:.1?} MB");
    if let Some(d) = delta_mb { println!("  Δ wasm  : {:+.2} MB", d); }
    if regression { println!("  REGRESSION: prior=PASS current=FAIL"); }

    let result = serde_json::json!({
        "project": project,
        "current": { "verdict": current_verdict, "wasm_mb": current_mb,
            "has_js": current.companions.has_js, "has_html": current.companions.has_html,
            "has_data_or_pak": current.companions.has_data_or_pak },
        "prior": prior, "size_delta_mb": delta_mb, "regression": regression,
    });
    if regression {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            format!("[html5 diff] REGRESSION: {project} was PASS, now FAIL")));
    }
    Ok(result)
}

/// Compare the current package against the last cook receipt (Supabase or local JSON).
///
/// Reports WASM size delta, companion file changes, verdict transition (PASS→FAIL).
/// Exits non-zero on regression so CI can catch broken cooks automatically.
#[verb("diff", "html5")]
fn html5_diff(project: Option<String>) -> clap_noun_verb::Result<Value> {
    do_html5_diff(project.as_deref().unwrap_or("Brm"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn html_with_body(body_content: &str) -> String {
        format!("<html><body>{body_content}</body></html>")
    }

    #[test]
    fn patch_injects_script_before_body_close() {
        let dir = TempDir::new().unwrap();
        let html_path = dir.path().join("Game.html");
        fs::write(&html_path, html_with_body("<canvas id='canvas'></canvas>")).unwrap();

        patch_html_for_ui_input(dir.path());

        let patched = fs::read_to_string(&html_path).unwrap();
        assert!(patched.contains("rocket-html5: suppress"), "must contain the patch sentinel");
        assert!(patched.contains("requestPointerLock"), "must patch requestPointerLock");
        assert!(patched.ends_with("</html>"), "file structure preserved");
    }

    #[test]
    fn patch_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let html_path = dir.path().join("Game.html");
        fs::write(&html_path, html_with_body("<canvas></canvas>")).unwrap();

        patch_html_for_ui_input(dir.path());
        let after_first = fs::read_to_string(&html_path).unwrap();

        patch_html_for_ui_input(dir.path());
        let after_second = fs::read_to_string(&html_path).unwrap();

        assert_eq!(after_first, after_second, "second patch must not change the file");
    }

    #[test]
    fn patch_skips_non_html_files() {
        let dir = TempDir::new().unwrap();
        let wasm_path = dir.path().join("Game.wasm");
        let js_path = dir.path().join("Game.js");
        let wasm_content = b"\x00asm";
        let js_content = b"Module = {}";
        fs::write(&wasm_path, wasm_content).unwrap();
        fs::write(&js_path, js_content).unwrap();

        patch_html_for_ui_input(dir.path()); // must not crash or modify non-HTML

        assert_eq!(fs::read(&wasm_path).unwrap(), wasm_content);
        assert_eq!(fs::read(&js_path).unwrap(), js_content);
    }

    #[test]
    fn patch_handles_empty_dir() {
        let dir = TempDir::new().unwrap();
        // Must not panic on an empty directory
        patch_html_for_ui_input(dir.path());
    }

    #[test]
    fn extract_cook_errors_finds_error_lines() {
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("ue4-cook.log");
        fs::write(&log, "LogCook: Info: Processing...\nError: Missing asset 'Foo'\nLogCook: Info: Done\nERROR: Build failed\n").unwrap();

        let errors = extract_cook_errors(&log);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Missing asset"));
        assert!(errors[1].contains("Build failed"));
    }

    #[test]
    fn extract_cook_errors_empty_on_clean_log() {
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("ue4-cook.log");
        fs::write(&log, "LogCook: Info: Cook complete\nLogHTML5: Success\n").unwrap();

        let errors = extract_cook_errors(&log);
        assert!(errors.is_empty());
    }

    #[test]
    fn extract_cook_errors_caps_at_20() {
        let dir = TempDir::new().unwrap();
        let log = dir.path().join("ue4-cook.log");
        let many_errors: String = (0..30).map(|i| format!("Error: asset{i} missing\n")).collect();
        fs::write(&log, many_errors).unwrap();

        let errors = extract_cook_errors(&log);
        assert_eq!(errors.len(), 20, "must cap at 20 errors");
    }

    #[test]
    fn extract_cook_errors_missing_file_returns_empty() {
        let errors = extract_cook_errors(std::path::Path::new("/nonexistent/cook.log"));
        assert!(errors.is_empty());
    }

    #[test]
    fn patch_handles_multiple_html_files() {
        let dir = TempDir::new().unwrap();
        for name in &["Brm.html", "index.html"] {
            fs::write(dir.path().join(name), html_with_body("<canvas></canvas>")).unwrap();
        }

        patch_html_for_ui_input(dir.path());

        for name in &["Brm.html", "index.html"] {
            let content = fs::read_to_string(dir.path().join(name)).unwrap();
            assert!(content.contains("rocket-html5: suppress"), "{name} must be patched");
        }
    }
}
