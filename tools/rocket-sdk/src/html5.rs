use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

const GIT_WRAPPER_DIR: &str = "/tmp/ubt-git-wrapper";
const PYTHON: &str = "/usr/local/bin/python3.11";

pub struct Html5Cook {
    pub engine_root: PathBuf,
    pub project: PathBuf,
    pub archive_dir: PathBuf,
    pub client_config: String,
}

impl Html5Cook {
    pub fn new(
        engine_root: impl Into<PathBuf>,
        project: impl Into<PathBuf>,
        archive_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            engine_root: engine_root.into(),
            project: project.into(),
            archive_dir: archive_dir.into(),
            client_config: "Development".into(),
        }
    }

    pub fn run(&self) -> Result<()> {
        ensure_git_wrapper()?;

        let run_uat = self.engine_root.join("Engine/Build/BatchFiles/RunUAT.sh");
        if !run_uat.exists() {
            bail!("RunUAT.sh not found at {}", run_uat.display());
        }

        // UHT computes CURRENT_FILE_ID relative to the parent of the project directory.
        // If that parent dir starts with a digit (e.g. versions/4.27.0/), all generated
        // macros start with a digit — invalid C identifier. Create a symlink with a
        // letter-starting name so UHT sees a valid base path.
        let project = ensure_letter_start_symlink(&self.project)?;

        let path_with_wrapper = prepend_to_path(GIT_WRAPPER_DIR);

        let status = Command::new("arch")
            .args(["-x86_64", "/bin/bash"])
            .arg(&run_uat)
            .args([
                "BuildCookRun",
                &format!("-project={}", project.display()),
                "-noP4",
                "-platform=HTML5",
                &format!("-clientconfig={}", self.client_config),
                "-cook",
                "-build",
                "-stage",
                "-pak",
                "-package",
                "-archive",
                "-IgnoreCookErrors",
                &format!("-archivedirectory={}", self.archive_dir.display()),
            ])
            .env("PYTHON", PYTHON)
            .env("PATH", &path_with_wrapper)
            .env("MONO_THREADS_SUSPEND", "preemptive")
            .env("MONO_GC_PARAMS", "nursery-size=64m")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to spawn RunUAT.sh")?;

        check_exit(status, "RunUAT.sh BuildCookRun")
    }
}

pub struct Html5Setup {
    pub engine_root: PathBuf,
}

impl Html5Setup {
    pub fn new(engine_root: impl Into<PathBuf>) -> Self {
        Self {
            engine_root: engine_root.into(),
        }
    }

    pub fn run(&self) -> Result<()> {
        let setup_sh = self
            .engine_root
            .join("Engine/Platforms/HTML5/HTML5Setup.sh");
        if !setup_sh.exists() {
            bail!("HTML5Setup.sh not found at {}", setup_sh.display());
        }

        ensure_metal_toolchain()?;

        // chmod +x all HTML5 shell scripts (emsdk tarball sometimes loses execute bits)
        let _ = Command::new("find")
            .args([
                self.engine_root
                    .join("Engine/Platforms/HTML5")
                    .to_str()
                    .unwrap(),
                "-name",
                "*.sh",
                "-exec",
                "chmod",
                "+x",
                "{}",
                ";",
            ])
            .status();

        let path_with_wrapper = prepend_to_path(GIT_WRAPPER_DIR);

        let status = Command::new("/opt/homebrew/bin/bash")
            .arg(&setup_sh)
            .env("PYTHON", PYTHON)
            .env("PATH", &path_with_wrapper)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to spawn HTML5Setup.sh")?;

        check_exit(status, "HTML5Setup.sh")
    }
}

/// On macOS 15+, the Metal compiler ships as a separately downloadable Xcode component.
/// cmake's system compiler probe triggers a popup if it's absent; we gate on it here so
/// the error surfaces as a typed diagnostic before HTML5Setup.sh launches.
#[cfg(target_os = "macos")]
fn ensure_metal_toolchain() -> Result<()> {
    let probe = Command::new("xcrun")
        .args(["--find", "metal"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match probe {
        Ok(s) if s.success() => return Ok(()),
        _ => {}
    }

    eprintln!("Metal compiler not found — downloading Metal toolchain (requires internet)...");
    let status = Command::new("xcodebuild")
        .args(["-downloadComponent", "MetalToolchain"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn xcodebuild -downloadComponent MetalToolchain")?;

    if !status.success() {
        bail!(
            "Metal toolchain download failed (exit {:?}).\n\
             Run manually: xcodebuild -downloadComponent MetalToolchain",
            status.code()
        );
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn ensure_metal_toolchain() -> Result<()> {
    Ok(())
}

/// UHT derives CURRENT_FILE_ID from the PARENT of the .uproject directory.
/// If that parent starts with a digit (e.g. `versions/4.27.0/`), the generated
/// macro prefix is `4_27_0_Source_...` — invalid C identifier. This function
/// creates a sibling symlink with a letter-starting name if needed.
/// .NET Path.GetFullPath() does NOT resolve symlinks, so UHT sees the link name.
fn ensure_letter_start_symlink(project: &Path) -> Result<PathBuf> {
    let project_dir = project
        .parent()
        .ok_or_else(|| anyhow::anyhow!("project has no parent dir"))?;
    let parent = project_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("project dir has no parent"))?;
    let dir_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("project dir name is not valid UTF-8"))?;

    let starts_with_digit = dir_name
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);
    if !starts_with_digit {
        return Ok(project.to_path_buf());
    }

    // Build a letter-starting name: strip leading digits, prefix with project stem
    let project_stem = project
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Project");
    let suffix: String = dir_name.chars().filter(|c| c.is_alphanumeric()).collect();
    let link_name = format!("{project_stem}{suffix}");
    let link_path = parent.join(&link_name);

    if !link_path.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(project_dir, &link_path).with_context(|| {
            format!(
                "create symlink {} → {}",
                link_path.display(),
                project_dir.display()
            )
        })?;
        #[cfg(not(unix))]
        bail!("symlink creation is not supported on this platform — rename the project directory to start with a letter");
    }

    let uproject_name = project
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("project has no file name"))?;
    Ok(link_path.join(uproject_name))
}

fn ensure_git_wrapper() -> Result<()> {
    let wrapper = Path::new(GIT_WRAPPER_DIR).join("git");
    if !wrapper.exists() {
        std::fs::create_dir_all(GIT_WRAPPER_DIR).context("create git wrapper dir")?;
        std::fs::write(&wrapper, "#!/bin/sh\nexit 0\n").context("write git wrapper")?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .context("chmod git wrapper")?;
    }
    Ok(())
}

fn prepend_to_path(dir: &str) -> String {
    match std::env::var("PATH") {
        Ok(p) => format!("{dir}:{p}"),
        Err(_) => dir.to_string(),
    }
}

fn check_exit(status: ExitStatus, label: &str) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("{label} failed with exit code {:?}", status.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_uproject(base: &Path, dir_name: &str, project_name: &str) -> PathBuf {
        let proj_dir = base.join(dir_name);
        std::fs::create_dir_all(&proj_dir).unwrap();
        let uproject = proj_dir.join(format!("{project_name}.uproject"));
        std::fs::write(&uproject, "{}").unwrap();
        uproject
    }

    #[test]
    fn letter_start_dir_unchanged() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "Brm427", "Brm");
        let result = ensure_letter_start_symlink(&uproject).unwrap();
        // Already starts with a letter — returned path should equal input
        assert_eq!(result, uproject);
    }

    #[test]
    fn digit_start_dir_gets_symlink() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "4.27.0", "Brm");
        let result = ensure_letter_start_symlink(&uproject).unwrap();
        // Result must start with a letter
        let link_name = result
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            link_name.starts_with(|c: char| c.is_ascii_alphabetic()),
            "symlink name '{link_name}' should start with a letter"
        );
        // The .uproject file must be reachable through the symlink
        assert!(
            result.exists(),
            "uproject must be accessible via symlink at {}",
            result.display()
        );
    }

    #[test]
    fn symlink_idempotent() {
        let dir = TempDir::new().unwrap();
        let uproject = make_uproject(dir.path(), "4.27.0", "Brm");
        let r1 = ensure_letter_start_symlink(&uproject).unwrap();
        let r2 = ensure_letter_start_symlink(&uproject).unwrap();
        assert_eq!(r1, r2, "calling twice must return the same path");
    }

    #[test]
    fn git_wrapper_is_created() {
        let wrapper = Path::new(GIT_WRAPPER_DIR).join("git");
        ensure_git_wrapper().unwrap();
        assert!(
            wrapper.exists(),
            "git wrapper must exist at {}",
            wrapper.display()
        );
        let content = std::fs::read_to_string(&wrapper).unwrap();
        assert!(content.contains("exit 0"));
    }

    #[test]
    fn prepend_to_path_contains_dir() {
        let result = prepend_to_path("/my/custom/dir");
        assert!(
            result.starts_with("/my/custom/dir"),
            "prepended dir must be first in PATH"
        );
        assert!(result.contains(':'), "must contain a PATH separator");
    }
}
