use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnosticReport {
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<CheckResult>,
}

pub struct RocketDoctor {
    project_root: PathBuf,
}

impl RocketDoctor {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn run_diagnostics(&self) -> DiagnosticReport {
        let checks = vec![
            self.check_git(),
            self.check_git_status(),
            self.check_rust(),
            self.check_python(),
            self.check_manifest(),
            self.check_versions_dir(),
            self.check_ue4_root(),
            self.check_ggen(),
        ];

        DiagnosticReport {
            timestamp: Utc::now(),
            checks,
        }
    }

    fn check_git(&self) -> CheckResult {
        match Command::new("git").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Git".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Git".to_string(),
                status: CheckStatus::Fail,
                message: "Git not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_git_status(&self) -> CheckResult {
        match git2::Repository::open(&self.project_root) {
            Ok(repo) => {
                let mut message = String::new();
                let mut status = CheckStatus::Pass;

                // Branch name
                let head = match repo.head() {
                    Ok(head) => head.shorthand().unwrap_or("unknown").to_string(),
                    Err(_) => "HEAD detached or empty".to_string(),
                };
                message.push_str(&format!("Branch: {}", head));

                // Uncommitted changes
                let mut status_options = git2::StatusOptions::new();
                status_options.include_untracked(true);
                match repo.statuses(Some(&mut status_options)) {
                    Ok(statuses) => {
                        if !statuses.is_empty() {
                            status = CheckStatus::Warn;
                            message.push_str(&format!(", {} uncommitted changes", statuses.len()));
                        } else {
                            message.push_str(", no uncommitted changes");
                        }
                    }
                    Err(e) => {
                        return CheckResult {
                            name: "Git Status".to_string(),
                            status: CheckStatus::Warn,
                            message: format!("Branch: {}, could not check statuses: {}", head, e),
                            details: None,
                        };
                    }
                }

                CheckResult {
                    name: "Git Status".to_string(),
                    status,
                    message,
                    details: None,
                }
            }
            Err(e) => CheckResult {
                name: "Git Status".to_string(),
                status: CheckStatus::Fail,
                message: "Not a git repository".to_string(),
                details: Some(e.to_string()),
            },
        }
    }

    fn check_rust(&self) -> CheckResult {
        match Command::new("rustc").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Rust".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Rust".to_string(),
                status: CheckStatus::Fail,
                message: "Rust (rustc) not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_python(&self) -> CheckResult {
        let cmd = if Command::new("python3").arg("--version").output().is_ok() {
            "python3"
        } else {
            "python"
        };

        match Command::new(cmd).arg("--version").output() {
            Ok(output) => CheckResult {
                name: "Python".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "Python".to_string(),
                status: CheckStatus::Fail,
                message: "Python not found in PATH".to_string(),
                details: None,
            },
        }
    }

    fn check_ggen(&self) -> CheckResult {
        match Command::new("ggen").arg("--version").output() {
            Ok(output) => CheckResult {
                name: "ggen".to_string(),
                status: CheckStatus::Pass,
                message: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                details: None,
            },
            Err(_) => CheckResult {
                name: "ggen".to_string(),
                status: CheckStatus::Warn,
                message: "ggen not found in PATH".to_string(),
                details: Some("ggen is required for Ostar generative workflows.".to_string()),
            },
        }
    }

    fn check_manifest(&self) -> CheckResult {
        let path = self.project_root.join("project-manifest.json");
        if path.exists() {
            CheckResult {
                name: "Project Manifest".to_string(),
                status: CheckStatus::Pass,
                message: "project-manifest.json found".to_string(),
                details: Some(format!("Path: {}", path.display())),
            }
        } else {
            CheckResult {
                name: "Project Manifest".to_string(),
                status: CheckStatus::Fail,
                message: "project-manifest.json MISSING".to_string(),
                details: Some("Run 'rocket sync' to generate it.".to_string()),
            }
        }
    }

    fn check_versions_dir(&self) -> CheckResult {
        let path = self.project_root.join("versions");
        if path.exists() && path.is_dir() {
            CheckResult {
                name: "Versions Directory".to_string(),
                status: CheckStatus::Pass,
                message: "versions/ directory exists".to_string(),
                details: None,
            }
        } else {
            CheckResult {
                name: "Versions Directory".to_string(),
                status: CheckStatus::Fail,
                message: "versions/ directory MISSING or not a directory".to_string(),
                details: Some("This directory should contain the Unreal Engine projects.".to_string()),
            }
        }
    }
    
    fn check_ue4_root(&self) -> CheckResult {
        let rocket_json = self.project_root.join(".rocket.json");
        if rocket_json.exists() {
             if let Ok(content) = std::fs::read_to_string(&rocket_json) {
                 if content.contains("ue4_root") {
                     return CheckResult {
                        name: "UE4 Root".to_string(),
                        status: CheckStatus::Pass,
                        message: "UE4 root configured in .rocket.json".to_string(),
                        details: None,
                    };
                 }
             }
        }
        
        if std::env::var("UE4_ROOT").is_ok() {
            CheckResult {
                name: "UE4 Root".to_string(),
                status: CheckStatus::Pass,
                message: "UE4_ROOT environment variable is set".to_string(),
                details: None,
            }
        } else {
            CheckResult {
                name: "UE4 Root".to_string(),
                status: CheckStatus::Warn,
                message: "UE4 root not configured".to_string(),
                details: Some("Run 'rocket setup' to configure Unreal Engine path.".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rocket_doctor_new() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        assert_eq!(doctor.project_root, dir.path().to_path_buf());
    }

    #[test]
    fn test_check_manifest_missing() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest();
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("MISSING"));
    }

    #[test]
    fn test_check_manifest_exists() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("project-manifest.json");
        fs::write(&manifest_path, "{}").unwrap();
        
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_manifest();
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("found"));
    }

    #[test]
    fn test_check_git_status_no_repo() {
        let dir = tempdir().unwrap();
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_git_status();
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("Not a git repository"));
    }

    #[test]
    fn test_check_git_status_with_repo() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        
        let doctor = RocketDoctor::new(dir.path().to_path_buf());
        let result = doctor.check_git_status();
        
        // Initial repo might have no HEAD yet
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("Branch:"));
        assert!(result.message.contains("no uncommitted changes"));

        // Add a file
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        let result = doctor.check_git_status();
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("1 uncommitted changes"));
    }
}
