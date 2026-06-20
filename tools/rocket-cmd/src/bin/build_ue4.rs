//! Stage 4 — Build UE4Editor (Mac / arm64 via Rosetta x86_64)
//!
//! Replaces build-ue4editor.sh with structured error extraction, stall detection,
//! pre-flight syntax checks, and a JSON build receipt.
//!
//! Usage:
//!   build_ue4 [--engine-root PATH] [--log PATH] [--stall-secs N] [--preflight]

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "build_ue4", about = "Build UE4Editor for Mac (Rosetta x86_64)")]
struct Args {
    #[arg(long, default_value = "/Users/sac/ue-4.27-html5-es3")]
    engine_root: PathBuf,

    #[arg(long, default_value_t = home_log())]
    log: String,

    /// Seconds of silence from the build before declaring a stall
    #[arg(long, default_value_t = 300)]
    stall_secs: u64,

    /// Run clang -fsyntax-only preflight on HTML5 platform sources before invoking UBT
    #[arg(long, default_value_t = false)]
    preflight: bool,

    /// Skip ShaderCompileWorker and UnrealPak (faster, editor-only)
    #[arg(long)]
    editor_only: bool,

    /// Extra flags passed directly to UnrealBuildTool (e.g. -DisableAdaptiveUnityBuild)
    #[arg(last = true)]
    ubt_args: Vec<String>,
}

fn home_log() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/ue4-build.log", home)
}

// ── Receipt ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
struct BuildReceipt {
    timestamp: String,
    engine_root: String,
    targets: Vec<TargetResult>,
    preflight: Option<PreflightResult>,
    editor_binary: Option<String>,
    editor_size_mb: Option<u64>,
    blake3_hash: Option<String>,
    success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TargetResult {
    target: String,
    success: bool,
    duration_secs: f64,
    errors: Vec<ExtractedError>,
    stalled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExtractedError {
    file: String,
    line: Option<u32>,
    message: String,
    raw: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PreflightResult {
    files_checked: usize,
    errors: Vec<ExtractedError>,
    passed: bool,
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args = Args::parse();

    let log_path = PathBuf::from(&args.log);
    let mut log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Cannot open log: {}", log_path.display()))?;

    let engine = &args.engine_root;
    let build_sh = engine.join("Engine/Build/BatchFiles/Mac/Build.sh");

    macro_rules! log {
        ($($arg:tt)*) => {{
            let msg = format!($($arg)*);
            let ts = Utc::now().format("%Y-%m-%d %H:%M:%S");
            let line = format!("[{ts}] {msg}");
            println!("{}", line);
            writeln!(log_file, "{}", line).ok();
        }};
    }

    log!("========================================");
    log!("STAGE 4 — build_ue4 (Rust)");
    log!("engine: {}", engine.display());
    log!("stall timeout: {}s", args.stall_secs);
    log!("preflight: {}", args.preflight);
    log!("========================================");

    // Gate: engine must exist
    if !build_sh.exists() {
        bail!(
            "BLOCKED: Build.sh not found at {}\nRun clone/setup first.",
            build_sh.display()
        );
    }
    log!("Gate passed: {}", build_sh.display());

    let mut receipt = BuildReceipt {
        timestamp: Utc::now().to_rfc3339(),
        engine_root: engine.display().to_string(),
        targets: vec![],
        preflight: None,
        editor_binary: None,
        editor_size_mb: None,
        blake3_hash: None,
        success: false,
    };

    // ── Preflight ─────────────────────────────────────────────────────────────
    if args.preflight {
        log!("--- Preflight: clang syntax check on HTML5 platform sources ---");
        let pf = run_preflight(engine, &mut log_file)?;
        log!(
            "Preflight: {} files checked, {} errors, {}",
            pf.files_checked,
            pf.errors.len(),
            if pf.passed {
                "PASSED".green()
            } else {
                "FAILED".red()
            }
        );
        if !pf.passed {
            for e in &pf.errors {
                log!(
                    "  PREFLIGHT ERROR: {}:{} — {}",
                    e.file,
                    e.line.unwrap_or(0),
                    e.message
                );
            }
            receipt.preflight = Some(pf);
            write_receipt(engine, &receipt)?;
            bail!("BLOCKED: Preflight syntax check failed — fix errors above before invoking UBT");
        }
        receipt.preflight = Some(pf);
    }

    // ── Targets ───────────────────────────────────────────────────────────────
    let targets: &[&str] = if args.editor_only {
        &["UE4Editor"]
    } else {
        &["UE4Editor", "ShaderCompileWorker", "UnrealPak"]
    };

    for target in targets {
        log!("--- Building: {} Mac Development ---", target);
        let result = build_target(
            target,
            &build_sh,
            args.stall_secs,
            &args.ubt_args,
            &mut log_file,
        )?;

        let ok = result.success;
        let stalled = result.stalled;
        receipt.targets.push(result);

        if stalled {
            write_receipt(engine, &receipt)?;
            bail!(
                "BLOCKED: {} stalled (no output for {}s) — killed. Receipt written.",
                target,
                args.stall_secs
            );
        }
        if !ok {
            let errors = &receipt.targets.last().unwrap().errors;
            log!("{} compile errors extracted:", errors.len());
            for e in errors.iter().take(20) {
                log!("  {}:{} — {}", e.file, e.line.unwrap_or(0), e.message);
            }
            write_receipt(engine, &receipt)?;
            bail!(
                "BLOCKED: {} Mac Development FAILED — {} errors. Receipt: {}",
                target,
                errors.len(),
                receipt_path(engine).display()
            );
        }
        log!(
            "Finished: {} Mac Development ({:.1}s)",
            target,
            receipt.targets.last().unwrap().duration_secs
        );
    }

    // ── Verify editor binary ──────────────────────────────────────────────────
    let editor_bin = engine.join("Engine/Binaries/Mac/UE4Editor");
    if !editor_bin.exists() {
        bail!(
            "BLOCKED: UE4Editor binary not found at {} after build",
            editor_bin.display()
        );
    }

    // On macOS, the loader binary is a dynamically linked stub.
    // The main engine code lives in UE4Editor-Engine.dylib, which we measure for size.
    let dylib_path = engine.join("Engine/Binaries/Mac/UE4Editor-Engine.dylib");
    let size_bytes = if dylib_path.exists() {
        fs::metadata(&dylib_path)?.len()
    } else {
        fs::metadata(&editor_bin)?.len()
    };
    let size_mb = size_bytes / (1024 * 1024);
    if size_mb < 100 {
        bail!(
            "BLOCKED: UE4Editor binary is only {} MB (expected >100 MB) — likely a stub",
            size_mb
        );
    }

    // BLAKE3 hash of the editor binary for the receipt
    let blake3_hex = hash_file(&editor_bin)?;

    log!("========================================");
    log!("Editor binary: {} ({} MB)", editor_bin.display(), size_mb);
    log!("BLAKE3: {}", blake3_hex);
    log!("STAGE 4 COMPLETE — ready for rocket build");
    log!("========================================");

    receipt.editor_binary = Some(editor_bin.display().to_string());
    receipt.editor_size_mb = Some(size_mb);
    receipt.blake3_hash = Some(blake3_hex);
    receipt.success = true;

    write_receipt(engine, &receipt)?;

    println!();
    println!("{}", "STAGE 4 COMPLETE".green().bold());
    println!("  Editor: {} ({} MB)", editor_bin.display(), size_mb);
    println!("  Receipt: {}", receipt_path(engine).display());

    Ok(())
}

// ── Preflight ─────────────────────────────────────────────────────────────────

fn run_preflight(engine: &Path, log_file: &mut fs::File) -> Result<PreflightResult> {
    let html5_private =
        engine.join("Engine/Platforms/HTML5/Source/Developer/HTML5TargetPlatform/Private");

    let mut cpp_files: Vec<PathBuf> = vec![];
    if html5_private.exists() {
        for entry in walkdir::WalkDir::new(&html5_private)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path().to_path_buf();
            if matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("cpp") | Some("h")
            ) {
                cpp_files.push(p);
            }
        }
    }

    let mut all_errors: Vec<ExtractedError> = vec![];

    // clang -fsyntax-only each .cpp (not .h — headers are checked via their including .cpp)
    let cpp_only: Vec<&PathBuf> = cpp_files
        .iter()
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("cpp"))
        .collect();

    let include_flags = format!(
        "-I{}/Engine/Source -I{} -I{}/Engine/Source/Runtime/Core/Public \
         -std=c++17 -x c++ -Wno-everything",
        engine.display(),
        html5_private.display(),
        engine.display()
    );

    for cpp in &cpp_only {
        let output = Command::new("clang")
            .args(["-fsyntax-only"])
            .args(include_flags.split_whitespace())
            .arg(cpp)
            .output();

        match output {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                for e in parse_clang_errors(&stderr, cpp) {
                    writeln!(
                        log_file,
                        "[PREFLIGHT] {}: {} — {}",
                        e.file,
                        e.line.unwrap_or(0),
                        e.message
                    )
                    .ok();
                    all_errors.push(e);
                }
            }
            Err(e) => {
                // clang not in PATH — skip preflight gracefully
                writeln!(log_file, "[PREFLIGHT] clang not available: {e} — skipping").ok();
                return Ok(PreflightResult {
                    files_checked: 0,
                    errors: vec![],
                    passed: true,
                });
            }
            _ => {}
        }
    }

    Ok(PreflightResult {
        files_checked: cpp_only.len(),
        errors: all_errors.clone(),
        passed: all_errors.is_empty(),
    })
}

// ── Build target ──────────────────────────────────────────────────────────────

fn build_target(
    target: &str,
    build_sh: &Path,
    stall_secs: u64,
    ubt_args: &[String],
    log_file: &mut fs::File,
) -> Result<TargetResult> {
    let start = Instant::now();

    let mut child = Command::new("arch")
        .args(["-x86_64", "/bin/bash"])
        .arg(build_sh)
        .arg(target)
        .arg("Mac")
        .arg("Development")
        .args(ubt_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn build for {target}"))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let errors: Arc<Mutex<Vec<ExtractedError>>> = Arc::new(Mutex::new(vec![]));
    let last_output: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));

    // Thread: drain stdout
    let errors_stdout = Arc::clone(&errors);
    let last_stdout = Arc::clone(&last_output);
    let mut log_clone = log_file.try_clone()?;
    let thread_stdout = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            *last_stdout.lock().unwrap() = Instant::now();
            writeln!(log_clone, "{}", line).ok();
            println!("{}", line);
            for e in parse_ubt_errors(&line) {
                errors_stdout.lock().unwrap().push(e);
            }
        }
    });

    // Thread: drain stderr
    let errors_stderr = Arc::clone(&errors);
    let last_stderr = Arc::clone(&last_output);
    let mut log_clone2 = log_file.try_clone()?;
    let thread_stderr = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            *last_stderr.lock().unwrap() = Instant::now();
            writeln!(log_clone2, "[stderr] {}", line).ok();
            for e in parse_ubt_errors(&line) {
                errors_stderr.lock().unwrap().push(e);
            }
        }
    });

    // Stall watchdog — polls every 10s
    let stall_timeout = Duration::from_secs(stall_secs);
    let last_watch = Arc::clone(&last_output);
    let mut stalled = false;

    loop {
        thread::sleep(Duration::from_secs(10));

        // Check if child already exited
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(e) => {
                writeln!(log_file, "[watchdog] try_wait error: {e}").ok();
                break;
            }
        }

        let idle = last_watch.lock().unwrap().elapsed();
        if idle > stall_timeout {
            writeln!(
                log_file,
                "[watchdog] STALL detected: {}s of silence — killing",
                idle.as_secs()
            )
            .ok();
            stalled = true;
            let _ = child.kill();
            break;
        }
    }

    thread_stdout.join().ok();
    thread_stderr.join().ok();

    let status = child.wait().ok();
    let success = !stalled && status.map(|s| s.success()).unwrap_or(false);
    let extracted = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();

    Ok(TargetResult {
        target: target.to_string(),
        success,
        duration_secs: start.elapsed().as_secs_f64(),
        errors: extracted,
        stalled,
    })
}

// ── Error parsers ─────────────────────────────────────────────────────────────

/// Parse clang `-fsyntax-only` stderr lines into structured errors.
fn parse_clang_errors(stderr: &str, source: &Path) -> Vec<ExtractedError> {
    let mut out = vec![];
    for raw in stderr.lines() {
        // Pattern: /path/to/file.cpp:10:9: error: message
        if !raw.contains(": error:") && !raw.contains(": fatal error:") {
            continue;
        }
        let parts: Vec<&str> = raw.splitn(5, ':').collect();
        if parts.len() >= 4 {
            let file = parts[0].to_string();
            let line = parts[1].parse::<u32>().ok();
            let message = parts.get(4).unwrap_or(&"").trim().to_string();
            out.push(ExtractedError {
                file,
                line,
                message,
                raw: raw.to_string(),
            });
        } else {
            out.push(ExtractedError {
                file: source.display().to_string(),
                line: None,
                message: raw.to_string(),
                raw: raw.to_string(),
            });
        }
    }
    out
}

/// Parse UBT / ninja output lines into structured errors.
fn parse_ubt_errors(line: &str) -> Vec<ExtractedError> {
    // Ninja / clang: /path/file.cpp:10:5: error: message
    if line.contains(": error:") || line.contains(": fatal error:") {
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        if parts.len() >= 4 {
            return vec![ExtractedError {
                file: parts[0].to_string(),
                line: parts[1].parse::<u32>().ok(),
                message: parts.get(4).unwrap_or(&"").trim().to_string(),
                raw: line.to_string(),
            }];
        }
    }
    // UBT C# style: "Error: ..."
    if line.trim_start().starts_with("Error: ") || line.contains("]: error ") {
        return vec![ExtractedError {
            file: String::new(),
            line: None,
            message: line.to_string(),
            raw: line.to_string(),
        }];
    }
    vec![]
}

// ── Receipt / hash helpers ────────────────────────────────────────────────────

fn receipt_path(engine: &Path) -> PathBuf {
    engine.join("build-receipt.json")
}

fn write_receipt(engine: &Path, receipt: &BuildReceipt) -> Result<()> {
    let path = receipt_path(engine);
    fs::write(&path, serde_json::to_string_pretty(receipt)?)?;
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_ubt_errors ──────────────────────────────────────────────────────

    #[test]
    fn ubt_clang_error_line_parsed() {
        let line =
            "/home/user/Engine/Source/foo.cpp:42:5: error: use of undeclared identifier 'Bar'";
        let errs = parse_ubt_errors(line);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].file, "/home/user/Engine/Source/foo.cpp");
        assert_eq!(errs[0].line, Some(42));
        assert!(errs[0].message.contains("undeclared identifier"));
    }

    #[test]
    fn ubt_fatal_error_parsed() {
        let line = "/src/file.h:1:10: fatal error: 'missing.h' file not found";
        let errs = parse_ubt_errors(line);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].message.contains("file not found"));
    }

    #[test]
    fn ubt_csharp_error_line_parsed() {
        let line = "Error: Module 'MyMod' not found";
        let errs = parse_ubt_errors(line);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].message.contains("not found"));
        assert_eq!(errs[0].file, String::new());
        assert_eq!(errs[0].line, None);
    }

    #[test]
    fn ubt_bracket_error_style_parsed() {
        let line = "  [1/100]: error UBT_12345: target failed";
        let errs = parse_ubt_errors(line);
        assert_eq!(errs.len(), 1);
    }

    #[test]
    fn ubt_warning_line_ignored() {
        let line = "/src/file.cpp:5:3: warning: unused variable 'x'";
        let errs = parse_ubt_errors(line);
        assert!(errs.is_empty(), "warnings must not be extracted as errors");
    }

    #[test]
    fn ubt_clean_build_line_ignored() {
        let line = "[42/500] Compiling CppFile.cpp";
        let errs = parse_ubt_errors(line);
        assert!(errs.is_empty());
    }

    // ── parse_clang_errors ───────────────────────────────────────────────────

    #[test]
    fn clang_error_extracted() {
        let stderr =
            "/path/to/HTML5TargetPlatform.cpp:10:9: error: unknown type name 'FPlatformMisc'";
        let path = std::path::Path::new("/path/to/HTML5TargetPlatform.cpp");
        let errs = parse_clang_errors(stderr, path);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].file, "/path/to/HTML5TargetPlatform.cpp");
        assert_eq!(errs[0].line, Some(10));
        assert!(errs[0].message.contains("unknown type name"));
    }

    #[test]
    fn clang_warning_ignored() {
        let stderr = "/path/file.cpp:5:1: warning: unused parameter 'ctx'";
        let path = std::path::Path::new("/path/file.cpp");
        let errs = parse_clang_errors(stderr, path);
        assert!(errs.is_empty(), "clang warnings must not be extracted");
    }

    #[test]
    fn clang_multiple_errors_extracted() {
        let stderr = "\
/src/a.cpp:1:1: error: first error
/src/b.cpp:2:2: error: second error";
        let path = std::path::Path::new("/src/a.cpp");
        let errs = parse_clang_errors(stderr, path);
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].file, "/src/a.cpp");
        assert_eq!(errs[1].file, "/src/b.cpp");
    }

    #[test]
    fn clang_fatal_error_included() {
        let stderr = "/path/file.h:1:1: fatal error: 'Core.h' file not found";
        let path = std::path::Path::new("/path/file.h");
        let errs = parse_clang_errors(stderr, path);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].raw.contains("fatal error"));
    }
}
