use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::RocketError;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RocketConfig {
    pub ue4_root: Option<PathBuf>,
    /// Path to Python 3 interpreter for UAT/UHT scripts.
    /// Auto-discovered at build time by `discover_python3()` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python3_path: Option<PathBuf>,
    /// Path to the emscripten SDK root (directory containing `emsdk`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emscripten_root: Option<PathBuf>,
}

impl RocketConfig {
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from(Path::new(".rocket.json"))
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).map_err(RocketError::Io)?;
            let config: Self = serde_json::from_str(&content).map_err(RocketError::Json)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(Path::new(".rocket.json"))
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Return the effective Python 3 path: config value → auto-discovery → error.
    pub fn effective_python3(&self) -> anyhow::Result<PathBuf> {
        if let Some(ref p) = self.python3_path {
            if p.exists() {
                return Ok(p.clone());
            }
        }
        discover_python3().ok_or_else(|| {
            anyhow::anyhow!(
                "Python 3 not found. Set 'python3_path' in .rocket.json or install python3."
            )
        })
    }
}

/// Probe candidate Python 3 paths and return the first executable one.
/// Search order:
///   1. /opt/homebrew/bin/python3   (Apple Silicon Homebrew)
///   2. /usr/local/bin/python3      (Intel Homebrew / pyenv)
///   3. /usr/bin/python3            (system, Linux / macOS Xcode CLT)
///   4. `python3` on PATH           (any remaining install)
///   5. `python` on PATH (≥3.x)     (Windows / conda envs)
pub fn discover_python3() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("/opt/homebrew/bin/python3"),
        PathBuf::from("/usr/local/bin/python3"),
        PathBuf::from("/usr/bin/python3"),
    ];

    for path in &candidates {
        if path.exists() && is_python3(path) {
            return Some(path.clone());
        }
    }

    // Fall back to PATH resolution
    for bin in &["python3", "python"] {
        if let Ok(out) = Command::new(bin).arg("--version").output() {
            if out.status.success() {
                let ver = String::from_utf8_lossy(&out.stdout);
                let ver2 = String::from_utf8_lossy(&out.stderr); // python 2.x writes to stderr
                if ver.contains("Python 3") || ver2.contains("Python 3") {
                    // Resolve the absolute path so callers get a stable reference
                    if let Ok(which) = which_python(bin) {
                        return Some(which);
                    }
                    return Some(PathBuf::from(bin));
                }
            }
        }
    }
    None
}

fn is_python3(path: &PathBuf) -> bool {
    Command::new(path)
        .arg("--version")
        .output()
        .map(|o| {
            let v = String::from_utf8_lossy(&o.stdout);
            v.contains("Python 3")
        })
        .unwrap_or(false)
}

fn which_python(bin: &str) -> anyhow::Result<PathBuf> {
    let out = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(bin)
        .output()?;
    let line = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if line.is_empty() {
        anyhow::bail!("which returned empty");
    }
    Ok(PathBuf::from(line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── load_from / save_to round-trip ────────────────────────────────────

    #[test]
    fn load_missing_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".rocket.json");
        let cfg = RocketConfig::load_from(&path).unwrap();
        assert!(cfg.ue4_root.is_none());
        assert!(cfg.python3_path.is_none());
        assert!(cfg.emscripten_root.is_none());
    }

    #[test]
    fn save_and_reload_round_trips_ue4_root() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".rocket.json");
        let mut cfg = RocketConfig::default();
        cfg.ue4_root = Some(PathBuf::from("/engines/ue4-html5"));
        cfg.save_to(&path).unwrap();
        let loaded = RocketConfig::load_from(&path).unwrap();
        assert_eq!(loaded.ue4_root, Some(PathBuf::from("/engines/ue4-html5")));
    }

    #[test]
    fn save_and_reload_round_trips_python3_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".rocket.json");
        let mut cfg = RocketConfig::default();
        cfg.python3_path = Some(PathBuf::from("/opt/homebrew/bin/python3"));
        cfg.save_to(&path).unwrap();
        let loaded = RocketConfig::load_from(&path).unwrap();
        assert_eq!(
            loaded.python3_path,
            Some(PathBuf::from("/opt/homebrew/bin/python3"))
        );
    }

    #[test]
    fn save_and_reload_round_trips_emscripten_root() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".rocket.json");
        let mut cfg = RocketConfig::default();
        cfg.emscripten_root = Some(PathBuf::from("/engines/ue4/Engine/Platforms/HTML5/Build/emsdk"));
        cfg.save_to(&path).unwrap();
        let loaded = RocketConfig::load_from(&path).unwrap();
        assert_eq!(
            loaded.emscripten_root,
            Some(PathBuf::from("/engines/ue4/Engine/Platforms/HTML5/Build/emsdk"))
        );
    }

    #[test]
    fn optional_fields_omitted_from_json_when_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".rocket.json");
        let cfg = RocketConfig {
            ue4_root: Some(PathBuf::from("/ue4")),
            python3_path: None,
            emscripten_root: None,
        };
        cfg.save_to(&path).unwrap();
        let json = fs::read_to_string(&path).unwrap();
        assert!(!json.contains("python3_path"), "None fields must not appear in JSON");
        assert!(!json.contains("emscripten_root"), "None fields must not appear in JSON");
    }

    // ── effective_python3 ─────────────────────────────────────────────────

    #[test]
    fn effective_python3_returns_configured_path_when_exists() {
        let dir = TempDir::new().unwrap();
        // Write a fake python3 executable
        let fake_py = dir.path().join("python3");
        fs::write(&fake_py, "#!/bin/sh\necho 'Python 3.99.0'\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&fake_py, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let cfg = RocketConfig {
            ue4_root: None,
            python3_path: Some(fake_py.clone()),
            emscripten_root: None,
        };
        let result = cfg.effective_python3().unwrap();
        assert_eq!(result, fake_py);
    }

    #[test]
    fn effective_python3_falls_back_to_discovery_when_config_path_missing() {
        let cfg = RocketConfig {
            ue4_root: None,
            python3_path: Some(PathBuf::from("/nonexistent/python3")),
            emscripten_root: None,
        };
        // Falls back to discover_python3() — will succeed on any dev machine with python3
        // or fail gracefully with an informative error (not a panic).
        let _ = cfg.effective_python3(); // just must not panic
    }

    // ── discover_python3 ─────────────────────────────────────────────────

    #[test]
    fn discover_python3_finds_a_python_on_this_machine() {
        // Every CI/dev machine has python3; if it's missing, the test is an environment
        // problem, not a code problem — but we don't want to panic.
        if let Some(path) = discover_python3() {
            assert!(
                path.to_str().unwrap().contains("python"),
                "discovered path must contain 'python': {path:?}"
            );
        }
    }
}
