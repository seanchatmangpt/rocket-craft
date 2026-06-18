//! `unify-rocket` — bridge crate connecting rocket-sdk patterns with the unify trait system.
//!
//! This crate mirrors the patterns from `tools/rocket-sdk`, `tools/knhk`, and
//! `tools/rocket-cmd` without directly depending on those workspaces.
//!
//! # Modules
//!
//! - [`context`] — WorkspaceManifest and WorkspaceContext (bridges RocketContext)
//! - [`compliance`] — ProjectLaw / ProjectValidator (bridges knhk::Law / knhk::Validator)
//! - [`receipt`] — BLAKE3 receipt chain for rocket operations
//! - [`classify`] — RocketClassify trait and command structs
//! - [`codegen`] — Makefile/Dockerfile code generation from manifests
//! - [`supabase`] — LeaderboardEntry and LeaderboardStore

// ---------------------------------------------------------------------------
// Re-exports (modules are defined inline below)
// ---------------------------------------------------------------------------

pub use context::{UeProject, WorkspaceContext, WorkspaceManifest};
pub use compliance::{LawViolation, NonEmptyTargetsLaw, ProjectLaw, ProjectValidator, ValidUprojectPathLaw};
pub use receipt::{RocketReceipt, RocketReceiptChain};
pub use classify::{
    RocketAuditCommand, RocketBuildCommand, RocketClassify, RocketDoctorCommand,
    RocketInfoCommand, RocketSetupCommand,
};
pub use codegen::{RocketDockerfileCodegen, RocketMakefileCodegen};
pub use supabase::{LeaderboardEntry, LeaderboardStore};

// ============================================================================
// context
// ============================================================================

pub mod context {
    //! Bridge types for `RocketContext` / `Manifest` from rocket-sdk.
    //!
    //! `WorkspaceManifest` deserialises from the same `project-manifest.json`
    //! format used by `tools/rocket-sdk`, without depending on that crate.

    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

    /// A single Unreal Engine project entry as stored in `project-manifest.json`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct UeProject {
        /// Human-readable project name (e.g. `"SurvivalGame"`).
        pub name: String,
        /// Relative path to the `.uproject` file from the workspace root.
        pub uproject_path: PathBuf,
        /// Build targets defined for this project.
        pub targets: Vec<String>,
    }

    impl UeProject {
        /// Create a new `UeProject` with the given name, path, and targets.
        pub fn new(name: impl Into<String>, uproject_path: impl Into<PathBuf>, targets: Vec<String>) -> Self {
            Self {
                name: name.into(),
                uproject_path: uproject_path.into(),
                targets,
            }
        }

        /// Returns `true` if this project has at least one build target.
        pub fn has_targets(&self) -> bool {
            !self.targets.is_empty()
        }

        /// Returns `true` if the `uproject_path` ends with a `.uproject` extension.
        pub fn has_valid_uproject_extension(&self) -> bool {
            self.uproject_path
                .extension()
                .map(|e| e == "uproject")
                .unwrap_or(false)
        }
    }

    /// The in-memory representation of `project-manifest.json`.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct WorkspaceManifest {
        /// All projects listed in the manifest.
        pub projects: Vec<UeProject>,
    }

    impl WorkspaceManifest {
        /// Parse a `project-manifest.json` from a file path.
        pub fn load(path: &Path) -> Result<Self, serde_json::Error> {
            let content = std::fs::read_to_string(path)
                .map_err(|e| serde_json::Error::io(e))?;
            serde_json::from_str(&content)
        }

        /// Parse a `WorkspaceManifest` directly from a JSON string.
        pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
            serde_json::from_str(s)
        }

        /// Look up a project by name (case-sensitive).
        pub fn project(&self, name: &str) -> Option<&UeProject> {
            self.projects.iter().find(|p| p.name == name)
        }

        /// Return `(project_name, target)` pairs for every project that has targets.
        pub fn all_targets(&self) -> Vec<(&str, &str)> {
            self.projects
                .iter()
                .flat_map(|p| p.targets.iter().map(move |t| (p.name.as_str(), t.as_str())))
                .collect()
        }

        /// Serialize the manifest to a pretty-printed JSON string.
        pub fn to_json(&self) -> String {
            serde_json::to_string_pretty(self).unwrap_or_default()
        }
    }

    /// High-level workspace context, pairing a filesystem root with its manifest.
    ///
    /// Mirrors `RocketContext` from `tools/rocket-sdk` without the dependency.
    pub struct WorkspaceContext {
        /// Absolute path to the workspace root.
        pub root: PathBuf,
        /// Parsed manifest data.
        pub manifest: WorkspaceManifest,
    }

    impl WorkspaceContext {
        /// Load a `WorkspaceContext` by reading `project-manifest.json` inside `root`.
        pub fn load(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
            let root = root.into();
            let manifest_path = root.join("project-manifest.json");
            let manifest = WorkspaceManifest::load(&manifest_path)
                .map_err(|e| anyhow::anyhow!("Failed to load manifest at {:?}: {}", manifest_path, e))?;
            Ok(Self { root, manifest })
        }

        /// Build a `WorkspaceContext` from an existing manifest without hitting disk.
        pub fn from_manifest(root: impl Into<PathBuf>, manifest: WorkspaceManifest) -> Self {
            Self {
                root: root.into(),
                manifest,
            }
        }

        /// Return the list of projects.
        pub fn projects(&self) -> &[UeProject] {
            &self.manifest.projects
        }

        /// Serialize the manifest to pretty-printed JSON.
        pub fn to_json(&self) -> String {
            self.manifest.to_json()
        }

        /// Return the absolute path to a project's `.uproject` file.
        pub fn absolute_uproject_path(&self, project: &UeProject) -> PathBuf {
            self.root.join(&project.uproject_path)
        }
    }
}

// ============================================================================
// compliance
// ============================================================================

pub mod compliance {
    //! Project-level law enforcement, bridging `knhk::Law` and `knhk::Validator`.
    //!
    //! Instead of operating on filesystem paths, these laws evaluate `UeProject`
    //! structs — making them useful for pre-flight validation before a build.

    use crate::context::UeProject;
    pub use unify_core::LawViolation;

    /// A semantic constraint that a [`UeProject`] must satisfy.
    ///
    /// Mirrors `knhk::Law` but operates on `UeProject` instead of filesystem paths.
    pub trait ProjectLaw: Send + Sync {
        /// Returns the unique name of this law.
        fn name(&self) -> &str;
        /// Returns a brief description of what this law enforces.
        fn description(&self) -> &str;
        /// Checks the project. Returns `Ok(())` on pass, `Err(LawViolation)` on failure.
        fn check(&self, project: &UeProject) -> Result<(), LawViolation>;
    }

    /// Registry and executor for multiple [`ProjectLaw`] implementations.
    ///
    /// Mirrors `knhk::Validator` but typed to `UeProject`.
    pub struct ProjectValidator {
        laws: Vec<Box<dyn ProjectLaw>>,
    }

    impl Default for ProjectValidator {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProjectValidator {
        /// Create an empty validator.
        pub fn new() -> Self {
            Self { laws: Vec::new() }
        }

        /// Add a law to the registry (builder-style, consumes and returns self).
        pub fn add(mut self, law: Box<dyn ProjectLaw>) -> Self {
            self.laws.push(law);
            self
        }

        /// Add a law by mutable reference.
        pub fn add_law(&mut self, law: Box<dyn ProjectLaw>) {
            self.laws.push(law);
        }

        /// Validate a single project against all registered laws.
        ///
        /// Returns every violation found; an empty vec means the project is compliant.
        pub fn validate(&self, project: &UeProject) -> Vec<LawViolation> {
            self.laws
                .iter()
                .filter_map(|law| law.check(project).err())
                .collect()
        }

        /// Validate every project in `projects`.
        ///
        /// Returns pairs of `(project_name, violations)` for projects that have
        /// at least one violation.
        pub fn validate_all(&self, projects: &[UeProject]) -> Vec<(String, Vec<LawViolation>)> {
            projects
                .iter()
                .filter_map(|p| {
                    let violations = self.validate(p);
                    if violations.is_empty() {
                        None
                    } else {
                        Some((p.name.clone(), violations))
                    }
                })
                .collect()
        }

        /// Validate every project and return `true` only if all pass all laws.
        pub fn all_pass(&self, projects: &[UeProject]) -> bool {
            projects.iter().all(|p| self.validate(p).is_empty())
        }
    }

    // -----------------------------------------------------------------------
    // Concrete laws
    // -----------------------------------------------------------------------

    /// Law: a project must declare at least one build target.
    pub struct NonEmptyTargetsLaw;

    impl ProjectLaw for NonEmptyTargetsLaw {
        fn name(&self) -> &str {
            "NonEmptyTargetsLaw"
        }

        fn description(&self) -> &str {
            "Every project must declare at least one build target."
        }

        fn check(&self, project: &UeProject) -> Result<(), LawViolation> {
            if project.targets.is_empty() {
                Err(LawViolation::new(
                    self.name(),
                    format!("Project '{}' has no build targets.", project.name),
                ))
            } else {
                Ok(())
            }
        }
    }

    /// Law: a project's `uproject_path` must end with `.uproject`.
    pub struct ValidUprojectPathLaw;

    impl ProjectLaw for ValidUprojectPathLaw {
        fn name(&self) -> &str {
            "ValidUprojectPathLaw"
        }

        fn description(&self) -> &str {
            "The uproject_path must have a '.uproject' extension."
        }

        fn check(&self, project: &UeProject) -> Result<(), LawViolation> {
            if project.has_valid_uproject_extension() {
                Ok(())
            } else {
                Err(LawViolation::new(
                    self.name(),
                    format!(
                        "Project '{}' uproject_path '{:?}' does not have a .uproject extension.",
                        project.name, project.uproject_path
                    ),
                ))
            }
        }
    }

    /// Law: project name must be non-empty.
    pub struct NonEmptyNameLaw;

    impl ProjectLaw for NonEmptyNameLaw {
        fn name(&self) -> &str {
            "NonEmptyNameLaw"
        }

        fn description(&self) -> &str {
            "Every project must have a non-empty name."
        }

        fn check(&self, project: &UeProject) -> Result<(), LawViolation> {
            if project.name.is_empty() {
                Err(LawViolation::new(self.name(), "Project name is empty."))
            } else {
                Ok(())
            }
        }
    }
}

// ============================================================================
// receipt
// ============================================================================

pub mod receipt {
    //! BLAKE3-backed receipt chain for rocket operations.
    //!
    //! Each `RocketReceipt` stores hashes of the operation parameters so that
    //! integrity can be verified at a later time.  `RocketReceiptChain` collects
    //! receipts produced during a session and exposes aggregate statistics.

    use serde::{Deserialize, Serialize};

    fn now_millis() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn blake3_hex(data: &str) -> String {
        blake3::hash(data.as_bytes()).to_hex().to_string()
    }

    /// A single receipt recording one rocket operation.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct RocketReceipt {
        /// Operation kind: `"build"`, `"audit"`, `"setup"`, `"doctor"`, etc.
        pub operation: String,
        /// The project this receipt is associated with.
        pub project_name: String,
        /// Optional build target (e.g. `"SurvivalGameEditor"`).
        pub target: Option<String>,
        /// Optional target platform (e.g. `"Win64"`, `"Linux"`).
        pub platform: Option<String>,
        /// BLAKE3 hash of `operation + project_name + target + platform`.
        pub data_hash: String,
        /// Unix timestamp in milliseconds when the receipt was created.
        pub timestamp: u64,
        /// BLAKE3 hash of all fields (self-sealing hash).
        pub receipt_hash: String,
        /// Whether the operation completed successfully.
        pub success: bool,
    }

    impl RocketReceipt {
        /// Create a new receipt, computing both `data_hash` and `receipt_hash`.
        pub fn new(
            operation: &str,
            project_name: &str,
            target: Option<&str>,
            platform: Option<&str>,
            success: bool,
        ) -> Self {
            let timestamp = now_millis();
            let data_payload = format!(
                "{}:{}:{}:{}",
                operation,
                project_name,
                target.unwrap_or(""),
                platform.unwrap_or(""),
            );
            let data_hash = blake3_hex(&data_payload);

            let receipt_payload = format!(
                "{}:{}:{}:{}:{}:{}:{}",
                operation,
                project_name,
                target.unwrap_or(""),
                platform.unwrap_or(""),
                &data_hash,
                timestamp,
                success,
            );
            let receipt_hash = blake3_hex(&receipt_payload);

            Self {
                operation: operation.to_owned(),
                project_name: project_name.to_owned(),
                target: target.map(str::to_owned),
                platform: platform.map(str::to_owned),
                data_hash,
                timestamp,
                receipt_hash,
                success,
            }
        }

        /// Recompute both hashes and verify they match stored values.
        pub fn verify(&self) -> bool {
            let data_payload = format!(
                "{}:{}:{}:{}",
                self.operation,
                self.project_name,
                self.target.as_deref().unwrap_or(""),
                self.platform.as_deref().unwrap_or(""),
            );
            let expected_data_hash = blake3_hex(&data_payload);
            if expected_data_hash != self.data_hash {
                return false;
            }

            let receipt_payload = format!(
                "{}:{}:{}:{}:{}:{}:{}",
                self.operation,
                self.project_name,
                self.target.as_deref().unwrap_or(""),
                self.platform.as_deref().unwrap_or(""),
                &self.data_hash,
                self.timestamp,
                self.success,
            );
            blake3_hex(&receipt_payload) == self.receipt_hash
        }

        /// Returns the operation string.
        pub fn operation(&self) -> &str {
            &self.operation
        }
    }

    /// An ordered chain of [`RocketReceipt`]s for a session.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct RocketReceiptChain {
        receipts: Vec<RocketReceipt>,
    }

    impl RocketReceiptChain {
        /// Create an empty chain.
        pub fn new() -> Self {
            Self { receipts: Vec::new() }
        }

        /// Append a new receipt to the chain.
        pub fn push(
            &mut self,
            op: &str,
            project: &str,
            target: Option<&str>,
            platform: Option<&str>,
            success: bool,
        ) {
            let receipt = RocketReceipt::new(op, project, target, platform, success);
            self.receipts.push(receipt);
        }

        /// Verify every receipt in the chain.
        pub fn verify_all(&self) -> bool {
            self.receipts.iter().all(|r| r.verify())
        }

        /// Return an immutable slice of all receipts.
        pub fn receipts(&self) -> &[RocketReceipt] {
            &self.receipts
        }

        /// Serialize to pretty-printed JSON.
        pub fn to_json(&self) -> String {
            serde_json::to_string_pretty(self).unwrap_or_default()
        }

        /// Deserialize from JSON.
        pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
            serde_json::from_str(s)
        }

        /// Number of receipts that recorded a successful operation.
        pub fn success_count(&self) -> usize {
            self.receipts.iter().filter(|r| r.success).count()
        }

        /// Number of receipts that recorded a failed operation.
        pub fn failure_count(&self) -> usize {
            self.receipts.iter().filter(|r| !r.success).count()
        }

        /// Total number of receipts.
        pub fn len(&self) -> usize {
            self.receipts.len()
        }

        /// Returns `true` if no receipts have been recorded.
        pub fn is_empty(&self) -> bool {
            self.receipts.is_empty()
        }
    }
}

// ============================================================================
// classify
// ============================================================================

pub mod classify {
    //! Noun-verb classify interface for rocket commands.
    //!
    //! Each command struct implements [`RocketClassify`], which mirrors the
    //! `Classify` trait in `unify-bp` without coupling to `unify-core`.

    use serde::{Deserialize, Serialize};

    /// A noun-verb command classification compatible with the unify-cli protocol.
    pub trait RocketClassify {
        /// Returns the namespace (always `"rocket"` for this crate).
        fn namespace(&self) -> &'static str {
            "rocket"
        }
        /// The noun (resource type): `"project"`, `"env"`, etc.
        fn noun(&self) -> &'static str;
        /// The verb (action): `"build"`, `"audit"`, `"doctor"`, etc.
        fn verb(&self) -> &'static str;
        /// A human-readable description of the command.
        fn description(&self) -> &str;
        /// Serialize the command's arguments to a JSON string.
        fn to_args_json(&self) -> String;
    }

    // -----------------------------------------------------------------------
    // RocketBuildCommand
    // -----------------------------------------------------------------------

    /// Command for building a project target on a specific platform.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RocketBuildCommand {
        /// Project name (must match a name in `project-manifest.json`).
        pub project: String,
        /// Build target (e.g. `"SurvivalGameEditor"`).
        pub target: String,
        /// Target platform (e.g. `"Win64"`, `"Linux"`, `"Android"`).
        pub platform: String,
    }

    impl RocketBuildCommand {
        /// Create a new build command.
        pub fn new(project: impl Into<String>, target: impl Into<String>, platform: impl Into<String>) -> Self {
            Self {
                project: project.into(),
                target: target.into(),
                platform: platform.into(),
            }
        }
    }

    impl RocketClassify for RocketBuildCommand {
        fn noun(&self) -> &'static str {
            let n = "project";
            n
        }
        fn verb(&self) -> &'static str {
            let v = "build";
            v
        }
        fn description(&self) -> &str {
            let desc = "Build a project target for a specific platform using RunUAT";
            desc
        }
        fn to_args_json(&self) -> String {
            let args = serde_json::to_string(self).unwrap_or_default();
            args
        }
    }

    // -----------------------------------------------------------------------
    // RocketAuditCommand
    // -----------------------------------------------------------------------

    /// Command for auditing compliance laws across one or all projects.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RocketAuditCommand {
        /// Audit only this project if `Some`, otherwise audit all projects.
        pub project: Option<String>,
    }

    impl RocketAuditCommand {
        /// Audit all projects.
        pub fn all() -> Self {
            Self { project: None }
        }

        /// Audit a single named project.
        pub fn for_project(name: impl Into<String>) -> Self {
            Self { project: Some(name.into()) }
        }
    }

    impl RocketClassify for RocketAuditCommand {
        fn noun(&self) -> &'static str {
            let n = "project";
            n
        }
        fn verb(&self) -> &'static str {
            let v = "audit";
            v
        }
        fn description(&self) -> &str {
            let desc = "Audit projects for compliance law violations";
            desc
        }
        fn to_args_json(&self) -> String {
            let args = serde_json::to_string(self).unwrap_or_default();
            args
        }
    }

    // -----------------------------------------------------------------------
    // RocketDoctorCommand
    // -----------------------------------------------------------------------

    /// Command for running environment diagnostics.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RocketDoctorCommand;

    impl RocketClassify for RocketDoctorCommand {
        fn noun(&self) -> &'static str {
            let n = "env";
            n
        }
        fn verb(&self) -> &'static str {
            let v = "doctor";
            v
        }
        fn description(&self) -> &str {
            let desc = "Run environment health checks (git, rust, UE4 root, manifest)";
            desc
        }
        fn to_args_json(&self) -> String {
            let args = "{}".to_owned();
            args
        }
    }

    // -----------------------------------------------------------------------
    // RocketSetupCommand
    // -----------------------------------------------------------------------

    /// Command for running interactive environment setup.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RocketSetupCommand;

    impl RocketClassify for RocketSetupCommand {
        fn noun(&self) -> &'static str {
            let n = "env";
            n
        }
        fn verb(&self) -> &'static str {
            let v = "setup";
            v
        }
        fn description(&self) -> &str {
            let desc = "Configure the UE4 root and other environment settings";
            desc
        }
        fn to_args_json(&self) -> String {
            let args = "{}".to_owned();
            args
        }
    }

    // -----------------------------------------------------------------------
    // RocketInfoCommand
    // -----------------------------------------------------------------------

    /// Command for printing workspace / manifest information.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RocketInfoCommand;

    impl RocketClassify for RocketInfoCommand {
        fn noun(&self) -> &'static str {
            let n = "workspace";
            n
        }
        fn verb(&self) -> &'static str {
            let v = "info";
            v
        }
        fn description(&self) -> &str {
            let desc = "Print workspace manifest and project information";
            desc
        }
        fn to_args_json(&self) -> String {
            let args = "{}".to_owned();
            args
        }
    }

    // -----------------------------------------------------------------------
    // Dynamic dispatch helper
    // -----------------------------------------------------------------------

    /// A boxed trait object for any rocket classify command.
    pub type BoxedRocketCommand = Box<dyn RocketClassify + Send + Sync>;

    /// A registry of rocket commands available in the current session.
    pub struct RocketCommandRegistry {
        commands: Vec<BoxedRocketCommand>,
    }

    impl Default for RocketCommandRegistry {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RocketCommandRegistry {
        /// Create an empty registry.
        pub fn new() -> Self {
            Self { commands: Vec::new() }
        }

        /// Register a command.
        pub fn register(&mut self, cmd: BoxedRocketCommand) {
            self.commands.push(cmd);
        }

        /// Find all commands with a given noun.
        pub fn by_noun<'a>(&'a self, noun: &str) -> Vec<&'a (dyn RocketClassify + Send + Sync)> {
            self.commands
                .iter()
                .filter(|c| c.noun() == noun)
                .map(|c| c.as_ref())
                .collect()
        }

        /// Number of registered commands.
        pub fn len(&self) -> usize {
            self.commands.len()
        }

        /// Returns `true` if no commands are registered.
        pub fn is_empty(&self) -> bool {
            self.commands.is_empty()
        }
    }
}

// ============================================================================
// codegen
// ============================================================================

pub mod codegen {
    //! Code generation from `WorkspaceManifest`.
    //!
    //! Generates Makefile.toml tasks and Dockerfiles suitable for CI pipelines
    //! that build Unreal Engine projects via RunUAT.

    use crate::context::WorkspaceManifest;

    /// Generates `Makefile.toml` task definitions from the workspace manifest.
    pub struct RocketMakefileCodegen<'a> {
        pub manifest: &'a WorkspaceManifest,
    }

    impl<'a> RocketMakefileCodegen<'a> {
        /// Create a new codegen instance wrapping the given manifest.
        pub fn new(manifest: &'a WorkspaceManifest) -> Self {
            Self { manifest }
        }

        /// Generate a `Makefile.toml` with one task per (project, target) pair.
        ///
        /// Projects without targets get a placeholder task that explains there are
        /// no buildable targets.
        pub fn generate_makefile(&self) -> String {
            let mut out = String::new();
            out.push_str("# Auto-generated by unify-rocket — do not edit by hand\n\n");

            out.push_str("[tasks.default]\n");
            out.push_str("description = \"List all available rocket tasks\"\n");
            out.push_str("script = \"cargo make --list-all-steps\"\n\n");

            for project in &self.manifest.projects {
                if project.targets.is_empty() {
                    let task_name = format!("build-{}", project.name.to_lowercase());
                    out.push_str(&format!("[tasks.{}]\n", task_name));
                    out.push_str(&format!(
                        "description = \"No buildable targets for {}\"\n",
                        project.name
                    ));
                    out.push_str("script = \"echo 'No targets defined'\"\n\n");
                } else {
                    for target in &project.targets {
                        let task_name = format!(
                            "build-{}-{}",
                            project.name.to_lowercase(),
                            target.as_str().to_lowercase()
                        );
                        out.push_str(&format!("[tasks.{}]\n", task_name));
                        out.push_str(&format!(
                            "description = \"Build {} / {} for Win64\"\n",
                            project.name, target
                        ));
                        out.push_str("script = [\n");
                        out.push_str("  \"RunUAT.sh BuildCookRun \\\\\",\n");
                        out.push_str(&format!(
                            "  \"  -project=${{PROJECT_ROOT}}/{} \\\\\",\n",
                            project.uproject_path.display()
                        ));
                        out.push_str(&format!("  \"  -target={} \\\\\",\n", target));
                        out.push_str("  \"  -platform=Win64\"\n");
                        out.push_str("]\n\n");
                    }
                }
            }

            out
        }

        /// Generate a shell script that runs `rocket audit` for every project.
        pub fn generate_audit_script(&self) -> String {
            let mut out = String::new();
            out.push_str("#!/usr/bin/env bash\n");
            out.push_str("# Auto-generated audit script — runs compliance checks for all projects\n");
            out.push_str("set -euo pipefail\n\n");

            for project in &self.manifest.projects {
                out.push_str(&format!(
                    "echo \"==> Auditing project: {}\"\n",
                    project.name
                ));
                out.push_str(&format!(
                    "rocket audit --project {}\n\n",
                    project.name
                ));
            }

            out.push_str("echo \"All audits complete.\"\n");
            out
        }
    }

    /// Generates a `Dockerfile` suitable for CI builds of Unreal Engine projects.
    pub struct RocketDockerfileCodegen<'a> {
        pub manifest: &'a WorkspaceManifest,
    }

    impl<'a> RocketDockerfileCodegen<'a> {
        /// Create a new codegen instance wrapping the given manifest.
        pub fn new(manifest: &'a WorkspaceManifest) -> Self {
            Self { manifest }
        }

        /// Generate a multi-stage Dockerfile with build arguments for each project.
        pub fn generate(&self) -> String {
            let mut out = String::new();

            out.push_str("# Auto-generated by unify-rocket\n");
            out.push_str("FROM ubuntu:20.04 AS base\n\n");
            out.push_str("# Install Unreal Engine build dependencies\n");
            out.push_str("RUN apt-get update && apt-get install -y \\\n");
            out.push_str("    build-essential \\\n");
            out.push_str("    clang \\\n");
            out.push_str("    cmake \\\n");
            out.push_str("    curl \\\n");
            out.push_str("    git \\\n");
            out.push_str("    mono-complete \\\n");
            out.push_str("    python3 \\\n");
            out.push_str("    && rm -rf /var/lib/apt/lists/*\n\n");
            out.push_str("ARG UE4_ROOT=/opt/UnrealEngine\n");
            out.push_str("ENV UE4_ROOT=${UE4_ROOT}\n\n");
            out.push_str("WORKDIR /workspace\n");
            out.push_str("COPY . .\n\n");

            // One build stage per project that has targets
            let buildable: Vec<_> = self
                .manifest
                .projects
                .iter()
                .filter(|p| !p.targets.is_empty())
                .collect();

            for project in &buildable {
                let stage = project.name.to_lowercase();
                out.push_str(&format!("# --- Project: {} ---\n", project.name));
                out.push_str(&format!("FROM base AS {}\n", stage));
                out.push_str(&format!(
                    "ARG TARGET={}\n",
                    project.targets.first().map(String::as_str).unwrap_or("Editor")
                ));
                out.push_str("RUN ${UE4_ROOT}/Engine/Build/BatchFiles/RunUAT.sh BuildCookRun \\\n");
                out.push_str(&format!(
                    "    -project=/workspace/{} \\\n",
                    project.uproject_path.display()
                ));
                out.push_str("    -target=${TARGET} \\\n");
                out.push_str("    -platform=Linux \\\n");
                out.push_str("    -cook -build -stage -archive -archivedirectory=/output\n\n");
            }

            // Final stage copies outputs
            out.push_str("FROM base AS final\n");
            for project in &buildable {
                let stage = project.name.to_lowercase();
                out.push_str(&format!(
                    "COPY --from={} /output /{}-output\n",
                    stage, stage
                ));
            }
            out.push_str("CMD [\"bash\"]\n");

            out
        }
    }
}

// ============================================================================
// supabase
// ============================================================================

pub mod supabase {
    //! In-memory leaderboard store mirroring `tools/rocket-sdk/src/supabase.rs`.
    //!
    //! This module decouples the data model from HTTP transport so that game-state
    //! types can be used in CLI code, tests, and codegen without pulling in reqwest.

    use serde::{Deserialize, Serialize};

    /// A leaderboard entry for a specific game project.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct LeaderboardEntry {
        /// Opaque player identifier.
        pub player_id: String,
        /// Display name of the player.
        pub player_name: String,
        /// Numeric score (higher is better).
        pub score: i64,
        /// Project (game) name the score belongs to.
        pub project: String,
        /// ISO 8601 submission timestamp.
        pub submitted_at: String,
    }

    impl LeaderboardEntry {
        /// Create a new entry.
        pub fn new(
            player_id: impl Into<String>,
            player_name: impl Into<String>,
            score: i64,
            project: impl Into<String>,
            submitted_at: impl Into<String>,
        ) -> Self {
            Self {
                player_id: player_id.into(),
                player_name: player_name.into(),
                score,
                project: project.into(),
                submitted_at: submitted_at.into(),
            }
        }
    }

    /// An in-memory store for [`LeaderboardEntry`] records.
    ///
    /// Supports filtering by project and returning the top-N entries by score.
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct LeaderboardStore {
        entries: Vec<LeaderboardEntry>,
    }

    impl LeaderboardStore {
        /// Create an empty store.
        pub fn new() -> Self {
            Self { entries: Vec::new() }
        }

        /// Add an entry.
        pub fn push(&mut self, entry: LeaderboardEntry) {
            self.entries.push(entry);
        }

        /// Return the top `n` entries for the given project, sorted by score descending.
        pub fn top_n(&self, n: usize, project: &str) -> Vec<&LeaderboardEntry> {
            let mut filtered: Vec<&LeaderboardEntry> = self
                .entries
                .iter()
                .filter(|e| e.project == project)
                .collect();
            filtered.sort_by(|a, b| b.score.cmp(&a.score));
            filtered.into_iter().take(n).collect()
        }

        /// Return all entries for the given project, sorted by score descending.
        pub fn for_project(&self, project: &str) -> Vec<&LeaderboardEntry> {
            let mut filtered: Vec<&LeaderboardEntry> = self
                .entries
                .iter()
                .filter(|e| e.project == project)
                .collect();
            filtered.sort_by(|a, b| b.score.cmp(&a.score));
            filtered
        }

        /// Total number of entries across all projects.
        pub fn len(&self) -> usize {
            self.entries.len()
        }

        /// Returns `true` if the store is empty.
        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        /// Serialize to pretty-printed JSON.
        pub fn to_json(&self) -> String {
            serde_json::to_string_pretty(self).unwrap_or_default()
        }

        /// Deserialize from JSON.
        pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
            serde_json::from_str(s)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use context::{UeProject, WorkspaceManifest, WorkspaceContext};
    use compliance::{
        LawViolation, NonEmptyTargetsLaw, ProjectLaw, ProjectValidator,
        ValidUprojectPathLaw,
    };
    use receipt::{RocketReceipt, RocketReceiptChain};
    use classify::{
        RocketAuditCommand, RocketBuildCommand, RocketClassify, RocketDoctorCommand,
        RocketInfoCommand, RocketSetupCommand,
    };
    use codegen::{RocketDockerfileCodegen, RocketMakefileCodegen};
    use supabase::{LeaderboardEntry, LeaderboardStore};

    // -----------------------------------------------------------------------
    // Helper fixture matching project-manifest.json
    // -----------------------------------------------------------------------

    fn real_manifest_json() -> &'static str {
        r#"
        {
          "projects": [
            {
              "name": "SurvivalGame",
              "uproject_path": "versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/SurvivalGame.uproject",
              "targets": ["SurvivalGameEditor", "SurvivalGameServer", "SurvivalGame"]
            },
            {
              "name": "RealisticRendering",
              "uproject_path": "versions/Realistic/RealisticRendering/RealisticRendering.uproject",
              "targets": []
            },
            {
              "name": "Brm",
              "uproject_path": "versions/4.24.0/Brm.uproject",
              "targets": ["BrmServer", "BrmEditor", "Brm"]
            },
            {
              "name": "ShooterGame",
              "uproject_path": "versions/4.24-Shooter/ShooterGame/ShooterGame.uproject",
              "targets": ["ShooterGameEditor", "ShooterClient", "ShooterGame", "ShooterServer"]
            }
          ]
        }
        "#
    }

    fn make_manifest() -> WorkspaceManifest {
        WorkspaceManifest::from_json(real_manifest_json()).expect("fixture must parse")
    }

    fn survival_project() -> UeProject {
        UeProject::new(
            "SurvivalGame",
            "versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/SurvivalGame.uproject",
            vec!["SurvivalGameEditor".into(), "SurvivalGameServer".into(), "SurvivalGame".into()],
        )
    }

    fn empty_targets_project() -> UeProject {
        UeProject::new(
            "RealisticRendering",
            "versions/Realistic/RealisticRendering/RealisticRendering.uproject",
            vec![],
        )
    }

    fn bad_extension_project() -> UeProject {
        UeProject::new(
            "BadProject",
            "versions/bad/project.txt",
            vec!["Editor".into()],
        )
    }

    // -----------------------------------------------------------------------
    // WorkspaceManifest tests
    // -----------------------------------------------------------------------

    #[test]
    fn manifest_load_from_json_parses_projects() {
        let m = make_manifest();
        assert_eq!(m.projects.len(), 4);
    }

    #[test]
    fn manifest_project_finds_by_name() {
        let m = make_manifest();
        let p = m.project("SurvivalGame").expect("should find SurvivalGame");
        assert_eq!(p.name, "SurvivalGame");
        assert_eq!(p.targets.len(), 3);
    }

    #[test]
    fn manifest_project_returns_none_for_unknown() {
        let m = make_manifest();
        assert!(m.project("DoesNotExist").is_none());
    }

    #[test]
    fn manifest_all_targets_returns_project_target_pairs() {
        let m = make_manifest();
        let pairs = m.all_targets();
        // SurvivalGame has 3, RealisticRendering has 0, Brm has 3, ShooterGame has 4
        assert_eq!(pairs.len(), 10);
        assert!(pairs.contains(&("SurvivalGame", "SurvivalGameEditor")));
        assert!(pairs.contains(&("ShooterGame", "ShooterServer")));
    }

    #[test]
    fn manifest_to_json_round_trips() {
        let m = make_manifest();
        let json = m.to_json();
        let m2 = WorkspaceManifest::from_json(&json).expect("round-trip must parse");
        assert_eq!(m, m2);
    }

    #[test]
    fn workspace_context_from_manifest_exposes_projects() {
        let m = make_manifest();
        let ctx = WorkspaceContext::from_manifest("/workspace", m.clone());
        assert_eq!(ctx.projects().len(), m.projects.len());
    }

    #[test]
    fn workspace_context_to_json_is_valid_json() {
        let m = make_manifest();
        let ctx = WorkspaceContext::from_manifest("/workspace", m);
        let json = ctx.to_json();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn workspace_context_absolute_uproject_path() {
        let m = make_manifest();
        let ctx = WorkspaceContext::from_manifest("/workspace", m);
        let p = ctx.projects().iter().find(|p| p.name == "SurvivalGame").unwrap();
        let abs = ctx.absolute_uproject_path(p);
        assert!(abs.starts_with("/workspace"));
        assert!(abs.to_string_lossy().ends_with(".uproject"));
    }

    // -----------------------------------------------------------------------
    // UeProject tests
    // -----------------------------------------------------------------------

    #[test]
    fn ue_project_has_targets_returns_true_when_non_empty() {
        assert!(survival_project().has_targets());
    }

    #[test]
    fn ue_project_has_targets_returns_false_for_empty() {
        assert!(!empty_targets_project().has_targets());
    }

    #[test]
    fn ue_project_valid_uproject_extension() {
        assert!(survival_project().has_valid_uproject_extension());
        assert!(!bad_extension_project().has_valid_uproject_extension());
    }

    // -----------------------------------------------------------------------
    // WorkspaceManifest file load test
    // -----------------------------------------------------------------------

    #[test]
    fn manifest_load_from_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("project-manifest.json");
        std::fs::write(&path, real_manifest_json()).expect("write fixture");
        let m = WorkspaceManifest::load(&path).expect("load");
        assert_eq!(m.projects.len(), 4);
    }

    // -----------------------------------------------------------------------
    // compliance tests
    // -----------------------------------------------------------------------

    #[test]
    fn non_empty_targets_law_passes_with_targets() {
        let law = NonEmptyTargetsLaw;
        assert!(law.check(&survival_project()).is_ok());
    }

    #[test]
    fn non_empty_targets_law_fails_with_empty_targets() {
        let law = NonEmptyTargetsLaw;
        let err = law.check(&empty_targets_project()).unwrap_err();
        assert_eq!(err.law_name, "NonEmptyTargetsLaw");
        assert!(err.message.contains("RealisticRendering"));
    }

    #[test]
    fn valid_uproject_path_law_passes_uproject_extension() {
        let law = ValidUprojectPathLaw;
        assert!(law.check(&survival_project()).is_ok());
    }

    #[test]
    fn valid_uproject_path_law_fails_bad_extension() {
        let law = ValidUprojectPathLaw;
        let err = law.check(&bad_extension_project()).unwrap_err();
        assert_eq!(err.law_name, "ValidUprojectPathLaw");
    }

    #[test]
    fn project_validator_validate_collects_violations() {
        let validator = ProjectValidator::new()
            .add(Box::new(NonEmptyTargetsLaw))
            .add(Box::new(ValidUprojectPathLaw));

        let violations = validator.validate(&empty_targets_project());
        assert_eq!(violations.len(), 1); // only NonEmptyTargetsLaw fires
    }

    #[test]
    fn project_validator_validate_all_skips_compliant_projects() {
        let validator = ProjectValidator::new()
            .add(Box::new(NonEmptyTargetsLaw));

        let manifest = make_manifest();
        let results = validator.validate_all(&manifest.projects);
        // Only RealisticRendering (and FullSpectrum if present) has no targets
        // In our 4-project fixture: RealisticRendering has empty targets
        assert!(results.iter().any(|(name, _)| name == "RealisticRendering"));
        // Projects with targets must not appear
        assert!(!results.iter().any(|(name, _)| name == "SurvivalGame"));
    }

    #[test]
    fn project_validator_all_pass_returns_false_on_any_violation() {
        let validator = ProjectValidator::new()
            .add(Box::new(NonEmptyTargetsLaw));
        let manifest = make_manifest();
        assert!(!validator.all_pass(&manifest.projects));
    }

    #[test]
    fn project_validator_all_pass_returns_true_when_no_violations() {
        let validator = ProjectValidator::new()
            .add(Box::new(ValidUprojectPathLaw));
        // All projects in our fixture have .uproject paths
        let manifest = make_manifest();
        assert!(validator.all_pass(&manifest.projects));
    }

    #[test]
    fn law_violation_display_format() {
        let v = LawViolation::new("MyLaw", "something broke");
        let s = v.to_string();
        assert!(s.contains("MyLaw"));
        assert!(s.contains("something broke"));
    }

    // -----------------------------------------------------------------------
    // receipt tests
    // -----------------------------------------------------------------------

    #[test]
    fn rocket_receipt_new_has_non_empty_hashes() {
        let r = RocketReceipt::new("build", "SurvivalGame", Some("SurvivalGameEditor"), Some("Win64"), true);
        assert!(!r.data_hash.is_empty());
        assert!(!r.receipt_hash.is_empty());
        assert_ne!(r.data_hash, r.receipt_hash);
    }

    #[test]
    fn rocket_receipt_verify_returns_true_for_fresh_receipt() {
        let r = RocketReceipt::new("audit", "Brm", None, None, false);
        assert!(r.verify());
    }

    #[test]
    fn rocket_receipt_verify_returns_false_after_tampering() {
        let mut r = RocketReceipt::new("build", "ShooterGame", Some("ShooterGameEditor"), Some("Linux"), true);
        r.receipt_hash = "0000000000000000000000000000000000000000000000000000000000000000".into();
        assert!(!r.verify());
    }

    #[test]
    fn rocket_receipt_chain_push_and_verify_all() {
        let mut chain = RocketReceiptChain::new();
        chain.push("build", "SurvivalGame", Some("SurvivalGameEditor"), Some("Win64"), true);
        chain.push("audit", "SurvivalGame", None, None, true);
        assert_eq!(chain.len(), 2);
        assert!(chain.verify_all());
    }

    #[test]
    fn rocket_receipt_chain_success_failure_counts() {
        let mut chain = RocketReceiptChain::new();
        chain.push("build", "Brm", Some("BrmEditor"), Some("Win64"), true);
        chain.push("build", "Brm", Some("BrmServer"), Some("Linux"), false);
        chain.push("audit", "Brm", None, None, true);
        assert_eq!(chain.success_count(), 2);
        assert_eq!(chain.failure_count(), 1);
    }

    #[test]
    fn rocket_receipt_chain_is_empty_on_new() {
        let chain = RocketReceiptChain::new();
        assert!(chain.is_empty());
    }

    #[test]
    fn rocket_receipt_chain_to_json_round_trips() {
        let mut chain = RocketReceiptChain::new();
        chain.push("doctor", "ShooterGame", None, None, true);
        let json = chain.to_json();
        let chain2 = RocketReceiptChain::from_json(&json).expect("round-trip");
        assert_eq!(chain2.len(), 1);
        assert!(chain2.verify_all());
    }

    // -----------------------------------------------------------------------
    // classify tests
    // -----------------------------------------------------------------------

    #[test]
    fn rocket_build_command_noun_verb() {
        let cmd = RocketBuildCommand::new("SurvivalGame", "SurvivalGameEditor", "Win64");
        assert_eq!(cmd.namespace(), "rocket");
        assert_eq!(cmd.noun(), "project");
        assert_eq!(cmd.verb(), "build");
    }

    #[test]
    fn rocket_audit_command_noun_verb() {
        let cmd = RocketAuditCommand::all();
        assert_eq!(cmd.noun(), "project");
        assert_eq!(cmd.verb(), "audit");
    }

    #[test]
    fn rocket_doctor_command_noun_verb() {
        let cmd = RocketDoctorCommand;
        assert_eq!(cmd.noun(), "env");
        assert_eq!(cmd.verb(), "doctor");
    }

    #[test]
    fn rocket_setup_command_noun_verb() {
        let cmd = RocketSetupCommand;
        assert_eq!(cmd.noun(), "env");
        assert_eq!(cmd.verb(), "setup");
    }

    #[test]
    fn rocket_info_command_noun_verb() {
        let cmd = RocketInfoCommand;
        assert_eq!(cmd.noun(), "workspace");
        assert_eq!(cmd.verb(), "info");
    }

    #[test]
    fn rocket_build_command_to_args_json_contains_fields() {
        let cmd = RocketBuildCommand::new("Brm", "BrmEditor", "Linux");
        let json = cmd.to_args_json();
        assert!(json.contains("Brm"));
        assert!(json.contains("BrmEditor"));
        assert!(json.contains("Linux"));
    }

    #[test]
    fn rocket_audit_command_for_project_serializes_name() {
        let cmd = RocketAuditCommand::for_project("ShooterGame");
        let json = cmd.to_args_json();
        assert!(json.contains("ShooterGame"));
    }

    // -----------------------------------------------------------------------
    // codegen tests
    // -----------------------------------------------------------------------

    #[test]
    fn makefile_codegen_contains_all_project_names() {
        let m = make_manifest();
        let gen = RocketMakefileCodegen::new(&m);
        let out = gen.generate_makefile();
        assert!(out.contains("survivalgame"));
        assert!(out.contains("brm"));
        assert!(out.contains("shootergame"));
    }

    #[test]
    fn makefile_codegen_contains_targets() {
        let m = make_manifest();
        let gen = RocketMakefileCodegen::new(&m);
        let out = gen.generate_makefile();
        assert!(out.contains("survivalgameeditor"));
        assert!(out.contains("shooterclient"));
    }

    #[test]
    fn makefile_codegen_placeholder_for_no_targets() {
        let m = make_manifest();
        let gen = RocketMakefileCodegen::new(&m);
        let out = gen.generate_makefile();
        // RealisticRendering has no targets → placeholder task
        assert!(out.contains("realisticrendering"));
        assert!(out.contains("No targets defined"));
    }

    #[test]
    fn audit_script_codegen_contains_all_projects() {
        let m = make_manifest();
        let gen = RocketMakefileCodegen::new(&m);
        let out = gen.generate_audit_script();
        assert!(out.contains("SurvivalGame"));
        assert!(out.contains("ShooterGame"));
        assert!(out.contains("Brm"));
    }

    #[test]
    fn dockerfile_codegen_contains_project_stages() {
        let m = make_manifest();
        let gen = RocketDockerfileCodegen::new(&m);
        let out = gen.generate();
        // Buildable projects get stages
        assert!(out.contains("survivalgame"));
        assert!(out.contains("shootergame"));
        assert!(out.contains("brm"));
        // RealisticRendering has no targets → no stage
        assert!(!out.contains("realisticrendering"));
    }

    #[test]
    fn dockerfile_codegen_includes_ue4_build_commands() {
        let m = make_manifest();
        let gen = RocketDockerfileCodegen::new(&m);
        let out = gen.generate();
        assert!(out.contains("RunUAT.sh"));
        assert!(out.contains("BuildCookRun"));
    }

    // -----------------------------------------------------------------------
    // supabase tests
    // -----------------------------------------------------------------------

    fn make_entries() -> Vec<LeaderboardEntry> {
        vec![
            LeaderboardEntry::new("p1", "Alice", 500, "SurvivalGame", "2024-01-01T00:00:00Z"),
            LeaderboardEntry::new("p2", "Bob",   300, "SurvivalGame", "2024-01-02T00:00:00Z"),
            LeaderboardEntry::new("p3", "Carol", 700, "SurvivalGame", "2024-01-03T00:00:00Z"),
            LeaderboardEntry::new("p4", "Dave",  900, "ShooterGame",  "2024-01-04T00:00:00Z"),
        ]
    }

    #[test]
    fn leaderboard_store_push_and_len() {
        let mut store = LeaderboardStore::new();
        for e in make_entries() {
            store.push(e);
        }
        assert_eq!(store.len(), 4);
    }

    #[test]
    fn leaderboard_store_top_n_sorted_desc() {
        let mut store = LeaderboardStore::new();
        for e in make_entries() {
            store.push(e);
        }
        let top = store.top_n(2, "SurvivalGame");
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].player_name, "Carol"); // 700
        assert_eq!(top[1].player_name, "Alice"); // 500
    }

    #[test]
    fn leaderboard_store_top_n_filters_by_project() {
        let mut store = LeaderboardStore::new();
        for e in make_entries() {
            store.push(e);
        }
        let top = store.top_n(10, "ShooterGame");
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].player_name, "Dave");
    }

    #[test]
    fn leaderboard_store_top_n_respects_n_cap() {
        let mut store = LeaderboardStore::new();
        for e in make_entries() {
            store.push(e);
        }
        let top = store.top_n(1, "SurvivalGame");
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].score, 700);
    }

    #[test]
    fn leaderboard_store_to_json_from_json_round_trip() {
        let mut store = LeaderboardStore::new();
        for e in make_entries() {
            store.push(e);
        }
        let json = store.to_json();
        let store2 = LeaderboardStore::from_json(&json).expect("round-trip");
        assert_eq!(store2.len(), 4);
    }

    #[test]
    fn leaderboard_store_is_empty_on_new() {
        let store = LeaderboardStore::new();
        assert!(store.is_empty());
    }
}
