//! First-run onboarding / setup wizard for the `rocket` CLI.
//!
//! Goal: **zero-to-productive in one command**. The wizard autodetects the local
//! toolchain (Rust, Node, Python, Blender, Docker, Java, Unreal Engine 4.24),
//! walks the developer through a handful of smart, skippable choices, and writes a
//! merged `.rocket.json` without clobbering keys it does not own. It is fully
//! idempotent (re-running only fills gaps), dry-run-able (`plan()` is separate from
//! `apply()`), and unit-testable end-to-end via the [`Prompter`] trait (no real
//! stdin, network, or UE4 install required).
//!
//! Wiring into `lib.rs` / `main.rs` is documented in the repo-root
//! `DX_ONBOARDING_WIRING.md`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// A single detected (or missing) tool on the developer's machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolStatus {
    /// Human-facing name, e.g. "Node.js".
    pub name: String,
    /// `true` if the tool was found.
    pub found: bool,
    /// Reported version string (trimmed first line of `--version`), if any.
    pub version: Option<String>,
    /// Resolved path / location hint, if known.
    pub location: Option<PathBuf>,
    /// Whether this tool is required for the core workflow (vs. optional).
    pub required: bool,
    /// Exact command to install it, surfaced in the next-steps checklist.
    pub install_hint: Option<String>,
}

impl ToolStatus {
    fn missing(name: &str, required: bool, install_hint: Option<&str>) -> Self {
        ToolStatus {
            name: name.to_string(),
            found: false,
            version: None,
            location: None,
            required,
            install_hint: install_hint.map(str::to_string),
        }
    }
}

/// Structured result of probing the local environment.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedEnv {
    pub rustc: ToolStatus,
    pub cargo: ToolStatus,
    pub node: ToolStatus,
    pub python: ToolStatus,
    pub blender: ToolStatus,
    pub docker: ToolStatus,
    pub java: ToolStatus,
    /// UE4 root candidates discovered (env var + common install locations).
    pub ue4_candidates: Vec<PathBuf>,
}

impl DetectedEnv {
    /// All tools as a flat list, in a stable display order.
    pub fn tools(&self) -> Vec<&ToolStatus> {
        vec![
            &self.rustc,
            &self.cargo,
            &self.node,
            &self.python,
            &self.blender,
            &self.docker,
            &self.java,
        ]
    }

    /// Required tools that were not found — these block the core workflow.
    pub fn missing_required(&self) -> Vec<&ToolStatus> {
        self.tools()
            .into_iter()
            .filter(|t| t.required && !t.found)
            .collect()
    }

    /// Optional tools that were not found — nice-to-haves.
    pub fn missing_optional(&self) -> Vec<&ToolStatus> {
        self.tools()
            .into_iter()
            .filter(|t| !t.required && !t.found)
            .collect()
    }

    /// Best-guess UE4 root from autodetection (first candidate, if any).
    pub fn ue4_default(&self) -> Option<PathBuf> {
        self.ue4_candidates.first().cloned()
    }
}

/// Abstracts the "run a tool and read its version" operation so detection can be
/// tested deterministically. Implementors return `None` when the tool is absent.
pub trait CommandProbe {
    /// Run `cmd --version` (or the given args) and return the trimmed first line
    /// of stdout/stderr, or `None` if the command could not be executed.
    fn probe(&self, cmd: &str, args: &[&str]) -> Option<String>;
}

/// Real probe that shells out to the system.
pub struct SystemProbe;

impl CommandProbe for SystemProbe {
    fn probe(&self, cmd: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(cmd).args(args).output().ok()?;
        if !output.status.success() && output.stdout.is_empty() && output.stderr.is_empty() {
            return None;
        }
        let mut text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.trim().is_empty() {
            // Some tools (e.g. java) print --version to stderr.
            text = String::from_utf8_lossy(&output.stderr).to_string();
        }
        let first = text.lines().next().unwrap_or("").trim().to_string();
        if first.is_empty() {
            None
        } else {
            Some(first)
        }
    }
}

/// Marker file (relative to a UE4 root) that confirms a real engine install.
fn ue4_marker(root: &Path) -> PathBuf {
    let uat_name = if cfg!(windows) {
        "RunUAT.bat"
    } else {
        "RunUAT.sh"
    };
    root.join("Engine")
        .join("Build")
        .join("BatchFiles")
        .join(uat_name)
}

/// Validate that `root` looks like a real UE4 install (has a RunUAT script).
pub fn is_ue4_root(root: &Path) -> bool {
    ue4_marker(root).exists()
}

/// Common UE4 install locations per platform (relative paths are joined to cwd).
fn ue4_common_locations() -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            PathBuf::from("ue4-4.24.3-html5"),
            PathBuf::from(r"C:\Program Files\Epic Games\UE_4.24"),
            PathBuf::from(r"D:\ue-engines\4.24-html\myengine"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("ue4-4.24.3-html5"),
            PathBuf::from("/Users/Shared/Epic Games/UE_4.24"),
        ]
    } else {
        vec![
            PathBuf::from("ue4-4.24.3-html5"),
            PathBuf::from("/opt/UnrealEngine"),
            PathBuf::from("/opt/UE_4.24"),
        ]
    }
}

/// Detect the environment using the given probe and a function that validates a
/// candidate UE4 root. Splitting these out keeps [`detect_env`] testable.
pub fn detect_env_with<P, V>(probe: &P, ue4_valid: &V) -> DetectedEnv
where
    P: CommandProbe,
    V: Fn(&Path) -> bool,
{
    let tool = |name: &str,
                cmd: &str,
                args: &[&str],
                required: bool,
                hint: Option<&str>|
     -> ToolStatus {
        match probe.probe(cmd, args) {
            Some(version) => ToolStatus {
                name: name.to_string(),
                found: true,
                version: Some(version),
                location: which(cmd),
                required,
                install_hint: hint.map(str::to_string),
            },
            None => ToolStatus::missing(name, required, hint),
        }
    };

    // Python: try python3 then python.
    let python = {
        let mut p = tool(
            "Python 3",
            "python3",
            &["--version"],
            true,
            Some("https://www.python.org/downloads/ (3.x)"),
        );
        if !p.found {
            p = tool(
                "Python 3",
                "python",
                &["--version"],
                true,
                Some("https://www.python.org/downloads/ (3.x)"),
            );
        }
        p
    };

    // Blender honours BLENDER_PATH before falling back to PATH.
    let blender = detect_blender(probe);

    DetectedEnv {
        rustc: tool(
            "Rust (rustc)",
            "rustc",
            &["--version"],
            true,
            Some("https://rustup.rs"),
        ),
        cargo: tool(
            "Cargo",
            "cargo",
            &["--version"],
            true,
            Some("https://rustup.rs"),
        ),
        node: tool(
            "Node.js",
            "node",
            &["--version"],
            true,
            Some("https://nodejs.org (20.x LTS) or `nvm install 20`"),
        ),
        python,
        blender,
        docker: tool(
            "Docker",
            "docker",
            &["--version"],
            false,
            Some("https://docs.docker.com/get-docker/ (for local Supabase)"),
        ),
        java: tool(
            "Java (JDK)",
            "java",
            &["-version"],
            false,
            Some("Install a JDK (Android builds): e.g. `apt install openjdk-17-jdk`"),
        ),
        ue4_candidates: detect_ue4_candidates(ue4_valid),
    }
}

fn detect_blender<P: CommandProbe>(probe: &P) -> ToolStatus {
    let hint = Some("https://www.blender.org/download/ and set BLENDER_PATH");
    if let Ok(path) = env::var("BLENDER_PATH") {
        let pb = PathBuf::from(&path);
        if let Some(version) = probe.probe(&path, &["--version"]) {
            return ToolStatus {
                name: "Blender".to_string(),
                found: true,
                version: Some(version),
                location: Some(pb),
                required: false,
                install_hint: hint.map(str::to_string),
            };
        }
    }
    match probe.probe("blender", &["--version"]) {
        Some(version) => ToolStatus {
            name: "Blender".to_string(),
            found: true,
            version: Some(version),
            location: which("blender"),
            required: false,
            install_hint: hint.map(str::to_string),
        },
        None => ToolStatus::missing("Blender", false, hint),
    }
}

/// Discover UE4 root candidates from `UE4_ROOT` and common install locations.
fn detect_ue4_candidates<V: Fn(&Path) -> bool>(ue4_valid: &V) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if ue4_valid(&p) && !out.contains(&p) {
            out.push(p);
        }
    };

    if let Ok(root) = env::var("UE4_ROOT") {
        push(PathBuf::from(root));
    }

    let cwd = env::current_dir().ok();
    for loc in ue4_common_locations() {
        push(loc.clone());
        if let Some(ref cwd) = cwd {
            push(cwd.join(&loc));
        }
    }
    out
}

/// Convenience: detect using the real system probe and real filesystem checks.
pub fn detect_env() -> DetectedEnv {
    detect_env_with(&SystemProbe, &is_ue4_root)
}

/// Best-effort `which`: locate an executable on `PATH`. Returns `None` if not
/// found (never errors — purely informational).
fn which(cmd: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![String::new()]
    };
    for dir in env::split_paths(&path) {
        for ext in &exts {
            let candidate = dir.join(format!("{cmd}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Prompting abstraction (testable without real stdin)
// ---------------------------------------------------------------------------

/// Abstraction over interactive prompts so the wizard's logic is unit-testable
/// without real stdin. [`StdinPrompter`] wraps `inquire`; [`MockPrompter`]
/// replays scripted answers in tests.
pub trait Prompter {
    /// Free-text question with a default shown to the user.
    fn text(&mut self, message: &str, default: Option<&str>) -> anyhow::Result<String>;
    /// Yes/no question with a default.
    fn confirm(&mut self, message: &str, default: bool) -> anyhow::Result<bool>;
    /// Single choice from `options`; `default_index` pre-selects an entry.
    fn select(
        &mut self,
        message: &str,
        options: &[String],
        default_index: usize,
    ) -> anyhow::Result<String>;
    /// Multi-select; `defaults` is a parallel mask of pre-checked options.
    fn multi_select(
        &mut self,
        message: &str,
        options: &[String],
        defaults: &[bool],
    ) -> anyhow::Result<Vec<String>>;
}

/// Real prompter backed by `inquire`.
pub struct StdinPrompter;

impl Prompter for StdinPrompter {
    fn text(&mut self, message: &str, default: Option<&str>) -> anyhow::Result<String> {
        let mut t = inquire::Text::new(message);
        if let Some(d) = default {
            t = t.with_default(d);
        }
        Ok(t.prompt()?)
    }

    fn confirm(&mut self, message: &str, default: bool) -> anyhow::Result<bool> {
        Ok(inquire::Confirm::new(message)
            .with_default(default)
            .prompt()?)
    }

    fn select(
        &mut self,
        message: &str,
        options: &[String],
        default_index: usize,
    ) -> anyhow::Result<String> {
        let idx = default_index.min(options.len().saturating_sub(1));
        Ok(inquire::Select::new(message, options.to_vec())
            .with_starting_cursor(idx)
            .prompt()?)
    }

    fn multi_select(
        &mut self,
        message: &str,
        options: &[String],
        defaults: &[bool],
    ) -> anyhow::Result<Vec<String>> {
        let default_idx: Vec<usize> = defaults
            .iter()
            .enumerate()
            .filter_map(|(i, &on)| if on { Some(i) } else { None })
            .collect();
        Ok(inquire::MultiSelect::new(message, options.to_vec())
            .with_default(&default_idx)
            .prompt()?)
    }
}

/// Scripted prompter for tests. Each prompt method pops the next scripted answer
/// of the appropriate kind; if none remain, it returns the offered default.
#[derive(Default)]
pub struct MockPrompter {
    pub texts: std::collections::VecDeque<String>,
    pub confirms: std::collections::VecDeque<bool>,
    pub selects: std::collections::VecDeque<String>,
    pub multi: std::collections::VecDeque<Vec<String>>,
}

impl MockPrompter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_text(mut self, s: impl Into<String>) -> Self {
        self.texts.push_back(s.into());
        self
    }
    pub fn with_confirm(mut self, b: bool) -> Self {
        self.confirms.push_back(b);
        self
    }
    pub fn with_select(mut self, s: impl Into<String>) -> Self {
        self.selects.push_back(s.into());
        self
    }
    pub fn with_multi(mut self, v: Vec<String>) -> Self {
        self.multi.push_back(v);
        self
    }
}

impl Prompter for MockPrompter {
    fn text(&mut self, _message: &str, default: Option<&str>) -> anyhow::Result<String> {
        Ok(self
            .texts
            .pop_front()
            .unwrap_or_else(|| default.unwrap_or("").to_string()))
    }
    fn confirm(&mut self, _message: &str, default: bool) -> anyhow::Result<bool> {
        Ok(self.confirms.pop_front().unwrap_or(default))
    }
    fn select(
        &mut self,
        _message: &str,
        options: &[String],
        default_index: usize,
    ) -> anyhow::Result<String> {
        Ok(self.selects.pop_front().unwrap_or_else(|| {
            options
                .get(default_index)
                .cloned()
                .unwrap_or_default()
        }))
    }
    fn multi_select(
        &mut self,
        _message: &str,
        options: &[String],
        defaults: &[bool],
    ) -> anyhow::Result<Vec<String>> {
        Ok(self.multi.pop_front().unwrap_or_else(|| {
            options
                .iter()
                .zip(defaults.iter())
                .filter_map(|(o, &on)| if on { Some(o.clone()) } else { None })
                .collect()
        }))
    }
}

// ---------------------------------------------------------------------------
// Config model (merge-safe view over .rocket.json)
// ---------------------------------------------------------------------------

/// Where Supabase lives for this developer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupabaseMode {
    /// Local Docker instance at 127.0.0.1:54321.
    Local,
    /// Hosted / remote Supabase project.
    Remote,
}

impl SupabaseMode {
    fn label(self) -> &'static str {
        match self {
            SupabaseMode::Local => "local",
            SupabaseMode::Remote => "remote",
        }
    }
}

/// The wizard-owned slice of `.rocket.json`. Unknown keys in the file are
/// preserved on write via [`merge_into_value`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ue4_root: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blender_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_html5: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_android: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_mode: Option<SupabaseMode>,
    /// HTML5 networking port (defaults to 8889 per project convention).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html5_port: Option<u16>,
    /// Game projects the developer cares about (subset of [`KNOWN_PROJECTS`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<String>,
}

/// The default HTML5 networking port for the monorepo.
pub const DEFAULT_HTML5_PORT: u16 = 8889;

/// Known UE4 game projects, from `project-manifest.json` / root CLAUDE.md.
pub const KNOWN_PROJECTS: &[&str] = &[
    "ShooterGame",
    "SurvivalGame",
    "Brm",
    "RealisticRendering",
    "FullSpectrum",
    "InfinityBlade4",
];

impl WizardConfig {
    /// `true` if every wizard-owned field has been populated. Used to decide
    /// whether a re-run has anything left to do.
    pub fn is_complete(&self) -> bool {
        self.ue4_root.is_some()
            && self.build_html5.is_some()
            && self.build_android.is_some()
            && self.supabase_mode.is_some()
            && self.html5_port.is_some()
            && !self.projects.is_empty()
    }
}

/// Read the raw `.rocket.json` value (preserving all keys) plus the parsed
/// wizard view. Returns `(raw_value, wizard_config)`; missing file yields
/// defaults. The raw value is always a JSON object (empty object if absent).
pub fn read_config(path: &Path) -> anyhow::Result<(serde_json::Value, WizardConfig)> {
    if !path.exists() {
        return Ok((serde_json::json!({}), WizardConfig::default()));
    }
    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok((serde_json::json!({}), WizardConfig::default()));
    }
    let raw: serde_json::Value = serde_json::from_str(&content)?;
    let cfg: WizardConfig = serde_json::from_value(raw.clone()).unwrap_or_default();
    Ok((raw, cfg))
}

/// Merge the wizard config into an existing raw JSON object, preserving any keys
/// the wizard does not own. Returns the merged value ready to serialize.
pub fn merge_into_value(existing: &serde_json::Value, cfg: &WizardConfig) -> serde_json::Value {
    let mut map: serde_json::Map<String, serde_json::Value> = existing
        .as_object()
        .cloned()
        .unwrap_or_default();

    let cfg_value = serde_json::to_value(cfg).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(cfg_map) = cfg_value.as_object() {
        for (k, v) in cfg_map {
            map.insert(k.clone(), v.clone());
        }
    }
    serde_json::Value::Object(map)
}

// ---------------------------------------------------------------------------
// Plan / Apply
// ---------------------------------------------------------------------------

/// A single field the wizard intends to change, for dry-run display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedChange {
    pub key: String,
    pub from: Option<String>,
    pub to: String,
}

/// The outcome of [`Wizard::plan`]: the resolved config plus a diff against what
/// was already on disk. Nothing is written until [`Wizard::apply`].
#[derive(Debug, Clone)]
pub struct Plan {
    /// The merged raw JSON value that would be written.
    pub merged: serde_json::Value,
    /// The resolved wizard config.
    pub config: WizardConfig,
    /// Field-level changes vs. the on-disk config.
    pub changes: Vec<PlannedChange>,
    /// Path the config will be written to.
    pub config_path: PathBuf,
}

impl Plan {
    /// `true` if applying this plan would not change anything on disk.
    pub fn is_noop(&self) -> bool {
        self.changes.is_empty()
    }
}

fn opt_str<T: std::fmt::Display>(v: &Option<T>) -> Option<String> {
    v.as_ref().map(|x| x.to_string())
}

fn diff(old: &WizardConfig, new: &WizardConfig) -> Vec<PlannedChange> {
    let mut changes = Vec::new();
    let mut push = |key: &str, from: Option<String>, to: Option<String>| {
        if let Some(to) = to {
            if from.as_deref() != Some(to.as_str()) {
                changes.push(PlannedChange {
                    key: key.to_string(),
                    from,
                    to,
                });
            }
        }
    };
    push(
        "ue4_root",
        old.ue4_root.as_ref().map(|p| p.display().to_string()),
        new.ue4_root.as_ref().map(|p| p.display().to_string()),
    );
    push(
        "blender_path",
        old.blender_path.as_ref().map(|p| p.display().to_string()),
        new.blender_path.as_ref().map(|p| p.display().to_string()),
    );
    push(
        "build_html5",
        opt_str(&old.build_html5),
        opt_str(&new.build_html5),
    );
    push(
        "build_android",
        opt_str(&old.build_android),
        opt_str(&new.build_android),
    );
    push(
        "supabase_mode",
        old.supabase_mode.map(|m| m.label().to_string()),
        new.supabase_mode.map(|m| m.label().to_string()),
    );
    push(
        "html5_port",
        opt_str(&old.html5_port),
        opt_str(&new.html5_port),
    );
    // Projects: compare as sorted, joined lists.
    let proj_to = if new.projects.is_empty() {
        None
    } else {
        let mut v = new.projects.clone();
        v.sort();
        Some(v.join(","))
    };
    let proj_from = if old.projects.is_empty() {
        None
    } else {
        let mut v = old.projects.clone();
        v.sort();
        Some(v.join(","))
    };
    push("projects", proj_from, proj_to);
    changes
}

// ---------------------------------------------------------------------------
// Wizard
// ---------------------------------------------------------------------------

/// The onboarding wizard. Generic over a [`Prompter`] so it can be driven by
/// real stdin (`StdinPrompter`) or scripted answers (`MockPrompter`).
pub struct Wizard<P: Prompter> {
    prompter: P,
    env: DetectedEnv,
    /// When `true`, every prompt is skipped and the smart default is taken.
    non_interactive: bool,
    config_path: PathBuf,
}

impl<P: Prompter> Wizard<P> {
    /// Create a wizard with explicit detected environment and config path.
    pub fn new(prompter: P, env: DetectedEnv, config_path: PathBuf) -> Self {
        Self {
            prompter,
            env,
            non_interactive: false,
            config_path,
        }
    }

    /// Take all defaults without prompting.
    pub fn non_interactive(mut self, yes: bool) -> Self {
        self.non_interactive = yes;
        self
    }

    /// Compute the resolved config and diff **without writing anything**.
    ///
    /// Existing `.rocket.json` values become the starting defaults so re-running
    /// the wizard only fills gaps (idempotent + resumable).
    pub fn plan(&mut self) -> anyhow::Result<Plan> {
        let (raw, existing) = read_config(&self.config_path)?;
        let resolved = self.resolve(&existing)?;
        let merged = merge_into_value(&raw, &resolved);
        let changes = diff(&existing, &resolved);
        Ok(Plan {
            merged,
            config: resolved,
            changes,
            config_path: self.config_path.clone(),
        })
    }

    /// Resolve every field, using existing config first, then autodetection, then
    /// prompting (unless `non_interactive`).
    fn resolve(&mut self, existing: &WizardConfig) -> anyhow::Result<WizardConfig> {
        let ue4_root = self.resolve_ue4(existing)?;
        let blender_path = self.resolve_blender(existing)?;
        let build_html5 = self.resolve_bool(
            existing.build_html5,
            true,
            "Will you build the HTML5 (web) target?",
        )?;
        let build_android = self.resolve_bool(
            existing.build_android,
            false,
            "Will you build the Android target?",
        )?;
        let supabase_mode = self.resolve_supabase(existing)?;
        let html5_port = Some(existing.html5_port.unwrap_or(DEFAULT_HTML5_PORT));
        let projects = self.resolve_projects(existing)?;

        Ok(WizardConfig {
            ue4_root,
            blender_path,
            build_html5: Some(build_html5),
            build_android: Some(build_android),
            supabase_mode: Some(supabase_mode),
            html5_port,
            projects,
        })
    }

    fn resolve_ue4(&mut self, existing: &WizardConfig) -> anyhow::Result<Option<PathBuf>> {
        // Prefer an already-configured value.
        if let Some(ref root) = existing.ue4_root {
            return Ok(Some(root.clone()));
        }
        let default = self.env.ue4_default();

        if self.non_interactive {
            return Ok(default);
        }

        // Build a selection list: detected candidates + manual entry + skip.
        let mut options: Vec<String> = self
            .env
            .ue4_candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        const MANUAL: &str = "Enter path manually...";
        const SKIP: &str = "Skip (configure later)";
        options.push(MANUAL.to_string());
        options.push(SKIP.to_string());

        let choice = self
            .prompter
            .select("Select your Unreal Engine 4.24 root:", &options, 0)?;
        if choice == SKIP {
            Ok(None)
        } else if choice == MANUAL {
            let entered = self
                .prompter
                .text("Path to Unreal Engine 4.24 root:", default.as_ref().map(|p| p.display().to_string()).as_deref())?;
            if entered.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(PathBuf::from(entered.trim())))
            }
        } else {
            Ok(Some(PathBuf::from(choice)))
        }
    }

    fn resolve_blender(&mut self, existing: &WizardConfig) -> anyhow::Result<Option<PathBuf>> {
        if let Some(ref p) = existing.blender_path {
            return Ok(Some(p.clone()));
        }
        let default = self.env.blender.location.clone();
        if self.non_interactive {
            return Ok(default);
        }
        let default_str = default.as_ref().map(|p| p.display().to_string());
        let entered = self.prompter.text(
            "Path to Blender (for the asset pipeline, blank to skip):",
            default_str.as_deref(),
        )?;
        if entered.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(PathBuf::from(entered.trim())))
        }
    }

    fn resolve_bool(
        &mut self,
        existing: Option<bool>,
        default: bool,
        message: &str,
    ) -> anyhow::Result<bool> {
        if let Some(v) = existing {
            return Ok(v);
        }
        if self.non_interactive {
            return Ok(default);
        }
        self.prompter.confirm(message, default)
    }

    fn resolve_supabase(&mut self, existing: &WizardConfig) -> anyhow::Result<SupabaseMode> {
        if let Some(m) = existing.supabase_mode {
            return Ok(m);
        }
        // Default: local if Docker is present, else remote.
        let default = if self.env.docker.found {
            SupabaseMode::Local
        } else {
            SupabaseMode::Remote
        };
        if self.non_interactive {
            return Ok(default);
        }
        let options = vec!["local".to_string(), "remote".to_string()];
        let default_index = if default == SupabaseMode::Local { 0 } else { 1 };
        let choice = self.prompter.select(
            "Supabase: local Docker instance or remote project?",
            &options,
            default_index,
        )?;
        Ok(if choice == "remote" {
            SupabaseMode::Remote
        } else {
            SupabaseMode::Local
        })
    }

    fn resolve_projects(&mut self, existing: &WizardConfig) -> anyhow::Result<Vec<String>> {
        if !existing.projects.is_empty() {
            return Ok(existing.projects.clone());
        }
        let options: Vec<String> = KNOWN_PROJECTS.iter().map(|s| s.to_string()).collect();
        // Default: select all projects.
        let defaults = vec![true; options.len()];
        if self.non_interactive {
            return Ok(options);
        }
        let chosen = self.prompter.multi_select(
            "Which game projects do you care about?",
            &options,
            &defaults,
        )?;
        // Never persist an empty list (would look "incomplete"); fall back to all.
        if chosen.is_empty() {
            Ok(options)
        } else {
            Ok(chosen)
        }
    }

    /// Write the plan to disk (creating or merging `.rocket.json`). Returns the
    /// resolved config. Idempotent: re-applying an identical plan rewrites the
    /// same bytes.
    pub fn apply(&mut self, plan: &Plan) -> anyhow::Result<WizardConfig> {
        let content = serde_json::to_string_pretty(&plan.merged)?;
        if let Some(parent) = plan.config_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(&plan.config_path, content + "\n")?;
        Ok(plan.config.clone())
    }

    /// Convenience: plan then apply in one call.
    pub fn run(&mut self) -> anyhow::Result<(Plan, WizardConfig)> {
        let plan = self.plan()?;
        let cfg = self.apply(&plan)?;
        Ok((plan, cfg))
    }

    /// Borrow the detected environment (for next-steps rendering).
    pub fn env(&self) -> &DetectedEnv {
        &self.env
    }
}

// ---------------------------------------------------------------------------
// Next-steps guidance
// ---------------------------------------------------------------------------

/// A personalized post-setup checklist item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NextStep {
    /// Short label.
    pub title: String,
    /// Exact command to run (if applicable).
    pub command: Option<String>,
    /// Whether this is blocking the core workflow (missing required tool, etc.).
    pub blocking: bool,
}

impl NextStep {
    fn action(title: &str, command: &str) -> Self {
        NextStep {
            title: title.to_string(),
            command: Some(command.to_string()),
            blocking: false,
        }
    }
}

/// Compute a personalized next-steps checklist from the detected environment and
/// the resolved config. Missing required tools come first (blocking), then
/// project-specific actions, then optional installs.
pub fn next_steps(env: &DetectedEnv, cfg: &WizardConfig) -> Vec<NextStep> {
    let mut steps = Vec::new();

    // 1. Blocking: missing required tools, with exact install commands.
    for tool in env.missing_required() {
        steps.push(NextStep {
            title: format!("Install required tool: {}", tool.name),
            command: tool.install_hint.clone(),
            blocking: true,
        });
    }

    // 2. UE4 root not set yet.
    if cfg.ue4_root.is_none() {
        steps.push(NextStep {
            title: "Configure Unreal Engine 4.24 root".to_string(),
            command: Some("./rocket setup --interactive".to_string()),
            blocking: true,
        });
    }

    // 3. Always-useful verification.
    steps.push(NextStep::action(
        "Verify your environment",
        "./rocket doctor",
    ));

    // 4. Project-specific build suggestion (first selected project).
    if let Some(project) = cfg.projects.first() {
        steps.push(NextStep::action(
            &format!("Build your first project ({project})"),
            &format!("./rocket build -p {project}"),
        ));
    }

    // 5. HTML5 / Android reminders.
    if cfg.build_html5 == Some(true) {
        let port = cfg.html5_port.unwrap_or(DEFAULT_HTML5_PORT);
        steps.push(NextStep {
            title: format!("HTML5 networking uses port {port} (WebSocketNetworking plugin)"),
            command: None,
            blocking: false,
        });
    }
    if cfg.build_android == Some(true) {
        steps.push(NextStep::action(
            "Generate an Android keystore",
            "./rocket crypto generate",
        ));
    }

    // 6. Supabase reminder.
    match cfg.supabase_mode {
        Some(SupabaseMode::Local) => {
            steps.push(NextStep::action(
                "Start local Supabase (needs Docker)",
                "supabase start",
            ));
        }
        Some(SupabaseMode::Remote) => {
            steps.push(NextStep {
                title: "Set your remote Supabase URL + anon key in the PWA env".to_string(),
                command: None,
                blocking: false,
            });
        }
        None => {}
    }

    // 7. Optional tools (non-blocking nice-to-haves).
    for tool in env.missing_optional() {
        steps.push(NextStep {
            title: format!("Optional: install {}", tool.name),
            command: tool.install_hint.clone(),
            blocking: false,
        });
    }

    steps
}

/// Render the next-steps checklist as a human-readable string.
pub fn render_next_steps(steps: &[NextStep]) -> String {
    if steps.is_empty() {
        return "You're all set — nothing else to do.".to_string();
    }
    let mut out = String::from("Next steps:\n");
    for (i, step) in steps.iter().enumerate() {
        let marker = if step.blocking { "!" } else { "-" };
        out.push_str(&format!("  {marker} {}. {}\n", i + 1, step.title));
        if let Some(cmd) = &step.command {
            out.push_str(&format!("      $ {cmd}\n"));
        }
    }
    out
}

/// Render the detected environment as a human-readable summary table.
pub fn render_env(env: &DetectedEnv) -> String {
    let mut out = String::from("Detected environment:\n");
    for tool in env.tools() {
        let status = if tool.found { "ok" } else { "MISSING" };
        let req = if tool.required { "required" } else { "optional" };
        let ver = tool.version.as_deref().unwrap_or("-");
        out.push_str(&format!(
            "  [{status:>7}] {} ({req}): {ver}\n",
            tool.name
        ));
    }
    if env.ue4_candidates.is_empty() {
        out.push_str("  UE4 4.24: no install detected\n");
    } else {
        out.push_str(&format!(
            "  UE4 4.24: {} candidate(s) found\n",
            env.ue4_candidates.len()
        ));
    }
    out
}

/// A small machine-readable summary, handy for `--json` style output later.
pub fn summary(env: &DetectedEnv, cfg: &WizardConfig) -> BTreeMap<String, serde_json::Value> {
    let mut m = BTreeMap::new();
    m.insert(
        "missing_required".to_string(),
        serde_json::json!(env.missing_required().len()),
    );
    m.insert(
        "config_complete".to_string(),
        serde_json::json!(cfg.is_complete()),
    );
    m.insert(
        "ue4_configured".to_string(),
        serde_json::json!(cfg.ue4_root.is_some()),
    );
    m
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A probe that returns scripted versions keyed by command name.
    struct FakeProbe {
        present: std::collections::HashMap<String, String>,
    }
    impl FakeProbe {
        fn new(present: &[(&str, &str)]) -> Self {
            FakeProbe {
                present: present
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }
    }
    impl CommandProbe for FakeProbe {
        fn probe(&self, cmd: &str, _args: &[&str]) -> Option<String> {
            self.present.get(cmd).cloned()
        }
    }

    fn detected(present: &[(&str, &str)], ue4: &[&str]) -> DetectedEnv {
        let probe = FakeProbe::new(present);
        let ue4_owned: Vec<PathBuf> = ue4.iter().map(PathBuf::from).collect();
        let valid = move |p: &Path| ue4_owned.iter().any(|c| c == p);
        // detect_env_with reads env vars for UE4/Blender; clear-ish by using a
        // validator that only accepts our explicit list and no env in test.
        let mut env = detect_env_with(&probe, &valid);
        // Override ue4_candidates deterministically (env var independence).
        env.ue4_candidates = ue4.iter().map(PathBuf::from).collect();
        env
    }

    #[test]
    fn detect_marks_present_and_missing_tools() {
        let env = detected(
            &[
                ("rustc", "rustc 1.85.0"),
                ("cargo", "cargo 1.85.0"),
                ("python3", "Python 3.12.0"),
            ],
            &[],
        );
        assert!(env.rustc.found);
        assert_eq!(env.rustc.version.as_deref(), Some("rustc 1.85.0"));
        assert!(env.cargo.found);
        assert!(env.python.found);
        assert!(!env.node.found);
        assert!(!env.docker.found);
        // node is required; docker is optional.
        let req: Vec<_> = env.missing_required().iter().map(|t| t.name.clone()).collect();
        assert!(req.contains(&"Node.js".to_string()));
        let opt: Vec<_> = env.missing_optional().iter().map(|t| t.name.clone()).collect();
        assert!(opt.contains(&"Docker".to_string()));
    }

    #[test]
    fn python_falls_back_to_python_command() {
        let env = detected(&[("python", "Python 3.11.0")], &[]);
        assert!(env.python.found);
        assert_eq!(env.python.version.as_deref(), Some("Python 3.11.0"));
    }

    #[test]
    fn non_interactive_takes_all_defaults_and_writes_config() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join(".rocket.json");
        let env = detected(
            &[("docker", "Docker version 25")],
            &["/opt/UnrealEngine"],
        );
        let mut wiz = Wizard::new(MockPrompter::new(), env, cfg_path.clone()).non_interactive(true);

        let plan = wiz.plan().unwrap();
        // Nothing on disk yet => everything is a change.
        assert!(!plan.is_noop());
        assert_eq!(plan.config.ue4_root, Some(PathBuf::from("/opt/UnrealEngine")));
        assert_eq!(plan.config.build_html5, Some(true));
        assert_eq!(plan.config.build_android, Some(false));
        assert_eq!(plan.config.supabase_mode, Some(SupabaseMode::Local)); // docker present
        assert_eq!(plan.config.html5_port, Some(DEFAULT_HTML5_PORT));
        assert_eq!(plan.config.projects.len(), KNOWN_PROJECTS.len());

        // Not written until apply.
        assert!(!cfg_path.exists());
        wiz.apply(&plan).unwrap();
        assert!(cfg_path.exists());

        let (_, on_disk) = read_config(&cfg_path).unwrap();
        assert!(on_disk.is_complete());
    }

    #[test]
    fn supabase_defaults_to_remote_without_docker() {
        let dir = tempdir().unwrap();
        let env = detected(&[], &[]);
        let mut wiz =
            Wizard::new(MockPrompter::new(), env, dir.path().join(".rocket.json")).non_interactive(true);
        let plan = wiz.plan().unwrap();
        assert_eq!(plan.config.supabase_mode, Some(SupabaseMode::Remote));
    }

    #[test]
    fn interactive_uses_scripted_answers() {
        let dir = tempdir().unwrap();
        let env = detected(&[], &["/opt/UnrealEngine"]);
        let prompter = MockPrompter::new()
            .with_select("/opt/UnrealEngine") // ue4 root selection
            .with_text("/usr/bin/blender") // blender path
            .with_confirm(false) // build html5?
            .with_confirm(true) // build android?
            .with_select("remote") // supabase
            .with_multi(vec!["ShooterGame".to_string()]); // projects
        let mut wiz = Wizard::new(prompter, env, dir.path().join(".rocket.json"));
        let (_, cfg) = wiz.run().unwrap();
        assert_eq!(cfg.ue4_root, Some(PathBuf::from("/opt/UnrealEngine")));
        assert_eq!(cfg.blender_path, Some(PathBuf::from("/usr/bin/blender")));
        assert_eq!(cfg.build_html5, Some(false));
        assert_eq!(cfg.build_android, Some(true));
        assert_eq!(cfg.supabase_mode, Some(SupabaseMode::Remote));
        assert_eq!(cfg.projects, vec!["ShooterGame".to_string()]);
    }

    #[test]
    fn skip_ue4_leaves_it_unset() {
        let dir = tempdir().unwrap();
        let env = detected(&[], &["/opt/UnrealEngine"]);
        let prompter = MockPrompter::new().with_select("Skip (configure later)");
        let mut wiz =
            Wizard::new(prompter, env.clone(), dir.path().join(".rocket.json")).non_interactive(false);
        // Empty confirm/multi queues fall back to their defaults.
        let plan = wiz.plan().unwrap();
        assert_eq!(plan.config.ue4_root, None);
        // The unset UE4 root must surface as a blocking next step.
        let steps = next_steps(&env, &plan.config);
        assert!(steps.iter().any(|s| s.blocking && s.title.contains("Unreal Engine")));
    }

    #[test]
    fn merge_preserves_unrelated_keys() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join(".rocket.json");
        // Pre-existing file with a key the wizard does not own.
        std::fs::write(
            &cfg_path,
            r#"{ "ue4_root": "/existing/ue4", "custom_team_setting": 42 }"#,
        )
        .unwrap();

        let env = detected(&[("docker", "Docker 25")], &["/opt/UnrealEngine"]);
        let mut wiz =
            Wizard::new(MockPrompter::new(), env, cfg_path.clone()).non_interactive(true);
        let (plan, _) = wiz.run().unwrap();

        // Existing ue4_root should be respected (resume), not overwritten by detect.
        assert_eq!(plan.config.ue4_root, Some(PathBuf::from("/existing/ue4")));

        // The unknown key must survive the merge.
        let written = std::fs::read_to_string(&cfg_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(v["custom_team_setting"], serde_json::json!(42));
        assert_eq!(v["ue4_root"], serde_json::json!("/existing/ue4"));
    }

    #[test]
    fn rerun_is_idempotent_and_noop_when_complete() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join(".rocket.json");
        let env = detected(&[("docker", "Docker 25")], &["/opt/UnrealEngine"]);

        // First run populates everything.
        let mut wiz1 =
            Wizard::new(MockPrompter::new(), env.clone(), cfg_path.clone()).non_interactive(true);
        wiz1.run().unwrap();
        let first = std::fs::read_to_string(&cfg_path).unwrap();

        // Second run should detect a complete config and plan no changes.
        let mut wiz2 =
            Wizard::new(MockPrompter::new(), env, cfg_path.clone()).non_interactive(true);
        let plan2 = wiz2.plan().unwrap();
        assert!(plan2.is_noop(), "expected no changes on re-run, got {:?}", plan2.changes);

        wiz2.apply(&plan2).unwrap();
        let second = std::fs::read_to_string(&cfg_path).unwrap();
        assert_eq!(first, second, "re-apply must be byte-identical");
    }

    #[test]
    fn resumable_fills_only_gaps() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join(".rocket.json");
        // Partial config: ue4 set, rest missing.
        std::fs::write(&cfg_path, r#"{ "ue4_root": "/preset/ue4" }"#).unwrap();

        let env = detected(&[], &["/opt/UnrealEngine"]);
        let mut wiz =
            Wizard::new(MockPrompter::new(), env, cfg_path.clone()).non_interactive(true);
        let plan = wiz.plan().unwrap();

        // ue4_root is NOT in the change set (already present).
        assert!(!plan.changes.iter().any(|c| c.key == "ue4_root"));
        // but the gaps are.
        assert!(plan.changes.iter().any(|c| c.key == "build_html5"));
        assert!(plan.changes.iter().any(|c| c.key == "supabase_mode"));
        assert_eq!(plan.config.ue4_root, Some(PathBuf::from("/preset/ue4")));
    }

    #[test]
    fn next_steps_flags_missing_required_and_unset_ue4() {
        // No tools, no ue4.
        let env = detected(&[], &[]);
        let mut cfg = WizardConfig::default();
        cfg.projects = vec!["ShooterGame".to_string()];
        let steps = next_steps(&env, &cfg);

        // There should be blocking steps for missing rust/node/python and ue4.
        let blocking: Vec<_> = steps.iter().filter(|s| s.blocking).collect();
        assert!(!blocking.is_empty());
        assert!(steps.iter().any(|s| s.title.contains("Unreal Engine")));
        // doctor and a build suggestion are always present.
        assert!(steps.iter().any(|s| s.command.as_deref() == Some("./rocket doctor")));
        assert!(steps
            .iter()
            .any(|s| s.command.as_deref() == Some("./rocket build -p ShooterGame")));
    }

    #[test]
    fn next_steps_android_and_local_supabase() {
        let env = detected(
            &[
                ("rustc", "rustc 1.85"),
                ("cargo", "cargo 1.85"),
                ("node", "v20.11.0"),
                ("python3", "Python 3.12"),
                ("docker", "Docker 25"),
            ],
            &["/opt/UnrealEngine"],
        );
        let cfg = WizardConfig {
            ue4_root: Some(PathBuf::from("/opt/UnrealEngine")),
            blender_path: None,
            build_html5: Some(true),
            build_android: Some(true),
            supabase_mode: Some(SupabaseMode::Local),
            html5_port: Some(DEFAULT_HTML5_PORT),
            projects: vec!["SurvivalGame".to_string()],
        };
        let steps = next_steps(&env, &cfg);
        assert!(steps
            .iter()
            .any(|s| s.command.as_deref() == Some("./rocket crypto generate")));
        assert!(steps
            .iter()
            .any(|s| s.command.as_deref() == Some("supabase start")));
        assert!(steps.iter().any(|s| s.title.contains("8889")));
        // No blocking steps: all required tools present and ue4 set.
        assert!(!steps.iter().any(|s| s.blocking));
    }

    #[test]
    fn render_helpers_do_not_panic() {
        let env = detected(&[("rustc", "rustc 1.85")], &["/opt/UE"]);
        let cfg = WizardConfig::default();
        let s = render_env(&env);
        assert!(s.contains("Rust"));
        let steps = next_steps(&env, &cfg);
        let r = render_next_steps(&steps);
        assert!(r.contains("Next steps"));
        let sum = summary(&env, &cfg);
        assert!(sum.contains_key("config_complete"));
    }

    #[test]
    fn empty_or_missing_config_reads_as_default() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("nope.json");
        let (raw, cfg) = read_config(&p).unwrap();
        assert!(raw.is_object());
        assert_eq!(cfg, WizardConfig::default());

        let p2 = dir.path().join("empty.json");
        std::fs::write(&p2, "   ").unwrap();
        let (_, cfg2) = read_config(&p2).unwrap();
        assert_eq!(cfg2, WizardConfig::default());
    }
}
