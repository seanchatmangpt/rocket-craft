//! SOC2-grade access control primitives for the Rocket Craft monorepo.
//!
//! Implements a Role-Based Access Control (RBAC) model with conditional grants,
//! environment isolation checks, and a policy engine suitable for SOC2 Type II evidence
//! generation. The permission matrix can be exported as a Markdown table via
//! [`PolicyEngine::export_matrix`] and used as a living audit artifact.
//!
//! # Architecture
//!
//! ```text
//! PolicyEngine::default()   ← built-in role-permission matrix
//!   + load_policy(path)     ← optional JSON overrides (additive)
//!   → check(role, resource, action) → Decision
//!   → explain(...)          → audit trail String
//!   → export_matrix()       → Markdown SOC2 evidence artifact
//! ```
//!
//! All access decisions are represented as [`AccessDecision`] structs that implement
//! `Serialize`, enabling direct integration with the `audit_affidavit` log chain.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Resource taxonomy ──────────────────────────────────────────────────────────

/// A resource that can be protected by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Resource {
    /// A named UE4 game project (e.g. `ShooterGame`, `SurvivalGame`).
    UE4Project(String),
    /// An Android signing keystore file.
    Keystore(String),
    /// The monorepo `project-manifest.json` registry.
    Manifest,
    /// The tamper-evident audit log chain produced by `audit_affidavit`.
    AuditLog,
    /// Supabase project admin operations (service-role key usage, schema changes).
    SupabaseAdmin,
    /// CI/CD pipeline trigger (`.github/workflows/ci.yml`, `rocket build`).
    CiPipeline,
    /// Any secret or credential vault entry.
    SecretsVault,
    /// Escape hatch for domain-specific resources.
    Custom(String),
}

impl Resource {
    /// Returns a stable lowercase identifier used in the permission matrix.
    pub fn key(&self) -> String {
        match self {
            Resource::UE4Project(n) => format!("ue4project:{}", n.to_lowercase()),
            Resource::Keystore(n) => format!("keystore:{}", n.to_lowercase()),
            Resource::Manifest => "manifest".to_string(),
            Resource::AuditLog => "auditlog".to_string(),
            Resource::SupabaseAdmin => "supabase_admin".to_string(),
            Resource::CiPipeline => "ci_pipeline".to_string(),
            Resource::SecretsVault => "secrets_vault".to_string(),
            Resource::Custom(n) => format!("custom:{}", n.to_lowercase()),
        }
    }

    /// Human-readable display name for policy documents and matrix headers.
    pub fn display_name(&self) -> String {
        match self {
            Resource::UE4Project(n) => format!("UE4 Project ({n})"),
            Resource::Keystore(n) => format!("Keystore ({n})"),
            Resource::Manifest => "Project Manifest".to_string(),
            Resource::AuditLog => "Audit Log".to_string(),
            Resource::SupabaseAdmin => "Supabase Admin".to_string(),
            Resource::CiPipeline => "CI Pipeline".to_string(),
            Resource::SecretsVault => "Secrets Vault".to_string(),
            Resource::Custom(n) => format!("Custom ({n})"),
        }
    }
}

// ── Action taxonomy ────────────────────────────────────────────────────────────

/// An operation that can be performed on a [`Resource`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Retrieve / view a resource without modification.
    Read,
    /// Create or update a resource.
    Write,
    /// Permanently remove a resource.
    Delete,
    /// Invoke or run a resource (e.g. trigger a CI build, execute a WASM law).
    Execute,
    /// Review or export audit trail entries for a resource.
    Audit,
    /// Full administrative control, including policy changes.
    Admin,
}

impl Action {
    pub fn all() -> &'static [Action] {
        &[
            Action::Read,
            Action::Write,
            Action::Delete,
            Action::Execute,
            Action::Audit,
            Action::Admin,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Delete => "delete",
            Action::Execute => "execute",
            Action::Audit => "audit",
            Action::Admin => "admin",
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Role definitions ───────────────────────────────────────────────────────────

/// An actor role within the Rocket Craft organisation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Contributor with read/write access to UE4 projects.
    Developer,
    /// Senior contributor; additionally manages the manifest and keystores.
    LeadDeveloper,
    /// Infrastructure operator; controls CI pipelines and secrets.
    DevOps,
    /// Compliance reader; can inspect audit logs but not modify resources.
    SecurityAuditor,
    /// Unauthenticated or restricted viewer; read-only access to non-secrets.
    ReadOnly,
    /// Full administrative access to all resources.
    Admin,
    /// Automated service identity (e.g. CI bot, deploy agent).
    ServiceAccount(String),
}

impl Role {
    /// Stable lowercase identifier for this role.
    pub fn key(&self) -> String {
        match self {
            Role::Developer => "developer".to_string(),
            Role::LeadDeveloper => "lead_developer".to_string(),
            Role::DevOps => "devops".to_string(),
            Role::SecurityAuditor => "security_auditor".to_string(),
            Role::ReadOnly => "readonly".to_string(),
            Role::Admin => "admin".to_string(),
            Role::ServiceAccount(name) => format!("service_account:{}", name.to_lowercase()),
        }
    }

    /// Human-readable display name for policy documents.
    pub fn display_name(&self) -> String {
        match self {
            Role::Developer => "Developer".to_string(),
            Role::LeadDeveloper => "Lead Developer".to_string(),
            Role::DevOps => "DevOps Engineer".to_string(),
            Role::SecurityAuditor => "Security Auditor".to_string(),
            Role::ReadOnly => "Read-Only Viewer".to_string(),
            Role::Admin => "Administrator".to_string(),
            Role::ServiceAccount(name) => format!("Service Account ({name})"),
        }
    }

    /// All well-known non-parameterised roles (used when generating the full matrix).
    pub fn canonical_list() -> Vec<Role> {
        vec![
            Role::ReadOnly,
            Role::Developer,
            Role::LeadDeveloper,
            Role::DevOps,
            Role::SecurityAuditor,
            Role::Admin,
        ]
    }
}

// ── Conditions ─────────────────────────────────────────────────────────────────

/// Additional conditions that must be satisfied before an access grant is realised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The actor must have completed multi-factor authentication.
    RequireMfa,
    /// The request must originate from a VPN-assigned IP range.
    RequireVpn,
    /// The request must arrive within a UTC hour window (inclusive on both ends).
    TimeWindow { start_hour: u8, end_hour: u8 },
    /// The source IP must be in the allowlist.
    IpAllowlist(Vec<String>),
}

impl Condition {
    pub fn description(&self) -> String {
        match self {
            Condition::RequireMfa => "MFA authentication required".to_string(),
            Condition::RequireVpn => "VPN connection required".to_string(),
            Condition::TimeWindow { start_hour, end_hour } => {
                format!("Access restricted to UTC hours {start_hour:02}:00–{end_hour:02}:59")
            }
            Condition::IpAllowlist(ips) => {
                format!("Source IP must be in allowlist: [{}]", ips.join(", "))
            }
        }
    }
}

// ── Permission ─────────────────────────────────────────────────────────────────

/// A single permission entry binding a role to a resource+action pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub role: Role,
    pub resource: Resource,
    pub action: Action,
    /// Conditions that must be satisfied for this permission to become effective.
    pub conditions: Vec<Condition>,
}

impl Permission {
    pub fn unconditional(role: Role, resource: Resource, action: Action) -> Self {
        Self { role, resource, action, conditions: vec![] }
    }

    pub fn conditional(
        role: Role,
        resource: Resource,
        action: Action,
        conditions: Vec<Condition>,
    ) -> Self {
        Self { role, resource, action, conditions }
    }
}

// ── Explicit denial ────────────────────────────────────────────────────────────

/// An explicit deny entry that overrides any allow matching the same tuple.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitDeny {
    pub role: Role,
    pub resource: Resource,
    pub action: Action,
    pub reason: String,
}

// ── Decision ───────────────────────────────────────────────────────────────────

/// The outcome of a policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Access is unconditionally granted.
    Allow,
    /// Access is denied; `reason` is recorded in the audit trail.
    Deny { reason: String },
    /// Access is granted only when all listed conditions are satisfied.
    ConditionalAllow { conditions: Vec<Condition> },
}

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow | Decision::ConditionalAllow { .. })
    }
}

// ── Policy override format (JSON file) ────────────────────────────────────────

/// A single policy override read from an external JSON configuration file.
/// Additive: built-in defaults remain; file entries can add grants or explicit denials.
///
/// Example JSON:
/// ```json
/// [
///   { "role": "developer", "resource": "ci_pipeline", "action": "execute", "allow": true },
///   { "role": "readonly", "resource": "secrets_vault", "action": "read", "allow": false }
/// ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyOverride {
    pub role: String,
    pub resource: String,
    pub action: String,
    pub allow: bool,
}

// ── Access request / decision log types ───────────────────────────────────────

/// A request to access a resource, suitable for recording in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Globally unique request ID (UUID or similar).
    pub id: String,
    /// UTC timestamp at which the request was received.
    pub timestamp: DateTime<Utc>,
    /// Human-readable identity of the actor (email, username, service name).
    pub actor: String,
    /// Asserted role of the actor at the time of the request.
    pub role: Role,
    /// Target resource.
    pub resource: Resource,
    /// Requested action.
    pub action: Action,
}

/// The result of evaluating a policy for an [`AccessRequest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    /// The original request.
    pub request: AccessRequest,
    /// Policy evaluation outcome.
    pub decision: Decision,
    /// UTC timestamp at which the policy engine produced this decision.
    pub evaluated_at: DateTime<Utc>,
}

// ── Helper: parse simple resource / action strings from overrides ──────────────

fn parse_resource_key(s: &str) -> Resource {
    let lower = s.to_lowercase();
    if let Some(name) = lower.strip_prefix("ue4project:") {
        return Resource::UE4Project(name.to_string());
    }
    if let Some(name) = lower.strip_prefix("keystore:") {
        return Resource::Keystore(name.to_string());
    }
    if let Some(name) = lower.strip_prefix("custom:") {
        return Resource::Custom(name.to_string());
    }
    match lower.as_str() {
        "manifest" => Resource::Manifest,
        "auditlog" | "audit_log" => Resource::AuditLog,
        "supabase_admin" | "supabaseadmin" => Resource::SupabaseAdmin,
        "ci_pipeline" | "cipipeline" => Resource::CiPipeline,
        "secrets_vault" | "secretsvault" => Resource::SecretsVault,
        other => Resource::Custom(other.to_string()),
    }
}

fn parse_role_key(s: &str) -> Role {
    let lower = s.to_lowercase();
    if let Some(name) = lower.strip_prefix("service_account:") {
        return Role::ServiceAccount(name.to_string());
    }
    match lower.as_str() {
        "developer" => Role::Developer,
        "lead_developer" | "leaddeveloper" => Role::LeadDeveloper,
        "devops" => Role::DevOps,
        "security_auditor" | "securityauditor" => Role::SecurityAuditor,
        "readonly" | "read_only" => Role::ReadOnly,
        "admin" => Role::Admin,
        other => Role::ServiceAccount(other.to_string()),
    }
}

fn parse_action_key(s: &str) -> Option<Action> {
    match s.to_lowercase().as_str() {
        "read" => Some(Action::Read),
        "write" => Some(Action::Write),
        "delete" => Some(Action::Delete),
        "execute" => Some(Action::Execute),
        "audit" => Some(Action::Audit),
        "admin" => Some(Action::Admin),
        _ => None,
    }
}

// ── PolicyEngine ───────────────────────────────────────────────────────────────

/// The RBAC policy engine.
///
/// # Usage
///
/// ```rust
/// use rocket_sdk::access_control::{PolicyEngine, Role, Resource, Action, Decision};
///
/// let engine = PolicyEngine::new();
/// let decision = engine.check(&Role::Developer, &Resource::UE4Project("ShooterGame".into()), &Action::Write);
/// assert!(decision.is_allowed());
/// ```
#[derive(Debug, Default)]
pub struct PolicyEngine {
    /// Allowed permissions (built-in + overrides).
    allows: Vec<Permission>,
    /// Explicit denials (override any matching allow).
    denies: Vec<ExplicitDeny>,
}

impl PolicyEngine {
    // ── Constructor ────────────────────────────────────────────────────────────

    /// Returns a new engine pre-loaded with the built-in role-permission matrix.
    ///
    /// Matrix summary:
    ///
    /// | Role             | UE4Project     | Manifest | Keystore | AuditLog | CiPipeline | SecretsVault | SupabaseAdmin |
    /// |------------------|----------------|----------|----------|----------|------------|--------------|---------------|
    /// | ReadOnly         | Read           | Read     | —        | —        | —          | —            | —             |
    /// | Developer        | Read, Write    | Read     | —        | —        | —          | —            | —             |
    /// | LeadDeveloper    | Read, Write    | Read, Write | Read, Write | —   | —          | —            | —             |
    /// | DevOps           | Read           | Read     | Read     | Read     | Read,Execute| Read (cond) | —             |
    /// | SecurityAuditor  | Read           | Read     | —        | Read, Audit| —        | —            | —             |
    /// | Admin            | ALL            | ALL      | ALL      | ALL      | ALL        | ALL          | ALL           |
    pub fn new() -> Self {
        let mut engine = PolicyEngine { allows: vec![], denies: vec![] };
        engine.load_builtin_matrix();
        engine
    }

    fn load_builtin_matrix(&mut self) {
        // ── ReadOnly: view UE4 projects and manifest only ─────────────────────
        let ro_resources: &[(Resource, &[Action])] = &[
            (Resource::UE4Project("*".into()), &[Action::Read]),
            (Resource::Manifest, &[Action::Read]),
        ];
        for (res, actions) in ro_resources {
            for act in *actions {
                self.allows.push(Permission::unconditional(Role::ReadOnly, res.clone(), act.clone()));
            }
        }

        // ── Developer: read/write UE4 projects, read manifest ─────────────────
        let dev_resources: &[(Resource, &[Action])] = &[
            (Resource::UE4Project("*".into()), &[Action::Read, Action::Write]),
            (Resource::Manifest, &[Action::Read]),
        ];
        for (res, actions) in dev_resources {
            for act in *actions {
                self.allows.push(Permission::unconditional(Role::Developer, res.clone(), act.clone()));
            }
        }

        // ── LeadDeveloper: dev + manifest management + keystore read ──────────
        let lead_resources: &[(Resource, &[Action])] = &[
            (Resource::UE4Project("*".into()), &[Action::Read, Action::Write, Action::Delete]),
            (Resource::Manifest, &[Action::Read, Action::Write]),
            (Resource::Keystore("*".into()), &[Action::Read, Action::Write]),
            (Resource::AuditLog, &[Action::Read]),
        ];
        for (res, actions) in lead_resources {
            for act in *actions {
                self.allows.push(Permission::unconditional(Role::LeadDeveloper, res.clone(), act.clone()));
            }
        }

        // ── DevOps: CI pipeline control, limited secrets access (MFA-gated) ───
        let devops_plain: &[(Resource, &[Action])] = &[
            (Resource::UE4Project("*".into()), &[Action::Read]),
            (Resource::Manifest, &[Action::Read]),
            (Resource::Keystore("*".into()), &[Action::Read]),
            (Resource::AuditLog, &[Action::Read]),
            (Resource::CiPipeline, &[Action::Read, Action::Execute]),
        ];
        for (res, actions) in devops_plain {
            for act in *actions {
                self.allows.push(Permission::unconditional(Role::DevOps, res.clone(), act.clone()));
            }
        }
        // Secrets vault: MFA + VPN required
        self.allows.push(Permission::conditional(
            Role::DevOps,
            Resource::SecretsVault,
            Action::Read,
            vec![Condition::RequireMfa, Condition::RequireVpn],
        ));
        self.allows.push(Permission::conditional(
            Role::DevOps,
            Resource::SecretsVault,
            Action::Write,
            vec![Condition::RequireMfa, Condition::RequireVpn],
        ));

        // ── SecurityAuditor: read-only auditing, no writes ────────────────────
        let auditor_resources: &[(Resource, &[Action])] = &[
            (Resource::UE4Project("*".into()), &[Action::Read]),
            (Resource::Manifest, &[Action::Read]),
            (Resource::AuditLog, &[Action::Read, Action::Audit]),
            (Resource::CiPipeline, &[Action::Read]),
        ];
        for (res, actions) in auditor_resources {
            for act in *actions {
                self.allows.push(Permission::unconditional(
                    Role::SecurityAuditor,
                    res.clone(),
                    act.clone(),
                ));
            }
        }

        // ── Admin: full access to everything ─────────────────────────────────
        let all_resources = [
            Resource::UE4Project("*".into()),
            Resource::Keystore("*".into()),
            Resource::Manifest,
            Resource::AuditLog,
            Resource::SupabaseAdmin,
            Resource::CiPipeline,
            Resource::SecretsVault,
        ];
        for res in &all_resources {
            for act in Action::all() {
                self.allows.push(Permission::unconditional(Role::Admin, res.clone(), act.clone()));
            }
        }

        // ── ServiceAccount: CI bot — execute pipeline, read manifest ──────────
        let sa_wildcard = Role::ServiceAccount("*".into());
        let sa_resources: &[(Resource, &[Action])] = &[
            (Resource::Manifest, &[Action::Read]),
            (Resource::CiPipeline, &[Action::Read, Action::Execute]),
            (Resource::AuditLog, &[Action::Read]),
        ];
        for (res, actions) in sa_resources {
            for act in *actions {
                self.allows.push(Permission::unconditional(
                    sa_wildcard.clone(),
                    res.clone(),
                    act.clone(),
                ));
            }
        }

        // ── Explicit denials: no role (except Admin) may delete AuditLog ─────
        for role in &[
            Role::ReadOnly,
            Role::Developer,
            Role::LeadDeveloper,
            Role::DevOps,
            Role::SecurityAuditor,
        ] {
            self.denies.push(ExplicitDeny {
                role: role.clone(),
                resource: Resource::AuditLog,
                action: Action::Delete,
                reason: "Audit log deletion is prohibited to maintain SOC2 tamper-evident chain; only Admin may purge after formal retention review".to_string(),
            });
        }
    }

    // ── Query API ──────────────────────────────────────────────────────────────

    /// Evaluate the policy for a given actor role, resource, and action.
    ///
    /// Algorithm (deny-overrides):
    /// 1. If an explicit deny matches → `Deny`.
    /// 2. If an unconditional allow matches → `Allow`.
    /// 3. If only conditional allows match → `ConditionalAllow` (union of conditions).
    /// 4. Otherwise → `Deny { reason: "no matching grant" }`.
    pub fn check(&self, actor_role: &Role, resource: &Resource, action: &Action) -> Decision {
        // Step 1: explicit deny wins.
        if let Some(deny) = self.find_deny(actor_role, resource, action) {
            return Decision::Deny { reason: deny.reason.clone() };
        }

        // Step 2 & 3: find matching allows.
        let matching: Vec<&Permission> = self
            .allows
            .iter()
            .filter(|p| self.role_matches(&p.role, actor_role))
            .filter(|p| self.resource_matches(&p.resource, resource))
            .filter(|p| p.action == *action)
            .collect();

        if matching.is_empty() {
            return Decision::Deny {
                reason: format!(
                    "No grant found for role '{}' on resource '{}' with action '{}'",
                    actor_role.key(),
                    resource.key(),
                    action
                ),
            };
        }

        // If any unconditional allow exists → Allow.
        if matching.iter().any(|p| p.conditions.is_empty()) {
            return Decision::Allow;
        }

        // All matching entries are conditional — aggregate conditions.
        let mut all_conditions: Vec<Condition> = vec![];
        for perm in &matching {
            for cond in &perm.conditions {
                if !all_conditions.contains(cond) {
                    all_conditions.push(cond.clone());
                }
            }
        }
        Decision::ConditionalAllow { conditions: all_conditions }
    }

    /// Returns a human-readable explanation of the policy decision.
    /// Suitable for inclusion in audit trail entries.
    pub fn explain(&self, actor_role: &Role, resource: &Resource, action: &Action) -> String {
        let decision = self.check(actor_role, resource, action);
        let base = format!(
            "Policy check — role: '{}', resource: '{}', action: '{}' → ",
            actor_role.display_name(),
            resource.display_name(),
            action
        );
        match &decision {
            Decision::Allow => {
                format!("{base}ALLOW (unconditional grant in built-in role-permission matrix)")
            }
            Decision::Deny { reason } => {
                format!("{base}DENY — {reason}")
            }
            Decision::ConditionalAllow { conditions } => {
                let cond_list: Vec<String> = conditions.iter().map(|c| c.description()).collect();
                format!(
                    "{base}CONDITIONAL ALLOW — the following conditions must be satisfied: {}",
                    cond_list.join("; ")
                )
            }
        }
    }

    // ── Policy override loader ─────────────────────────────────────────────────

    /// Load additional policy overrides from a JSON file.
    ///
    /// The file must contain a JSON array of [`PolicyOverride`] objects.
    /// Entries with `allow: true` add unconditional grants; entries with `allow: false`
    /// add explicit denials. Built-in defaults are preserved.
    pub fn load_policy(path: &Path) -> Result<PolicyEngine> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read policy file: {}", path.display()))?;
        let overrides: Vec<PolicyOverride> = serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in policy file: {}", path.display()))?;

        let mut engine = PolicyEngine::new();
        for entry in &overrides {
            let role = parse_role_key(&entry.role);
            let resource = parse_resource_key(&entry.resource);
            let action = parse_action_key(&entry.action).with_context(|| {
                format!("Unknown action '{}' in policy override", entry.action)
            })?;

            if entry.allow {
                engine.allows.push(Permission::unconditional(role, resource, action));
            } else {
                engine.denies.push(ExplicitDeny {
                    role,
                    resource,
                    action,
                    reason: "Explicit deny from policy override file".to_string(),
                });
            }
        }

        Ok(engine)
    }

    // ── Matrix export ──────────────────────────────────────────────────────────

    /// Render the full role × resource × action permission matrix as a Markdown table.
    ///
    /// Used to generate SOC2 evidence artifacts. The matrix covers the canonical resource
    /// set (omitting wildcard variants) and all actions.
    pub fn export_matrix(&self) -> String {
        let canonical_resources = vec![
            Resource::UE4Project("ShooterGame".into()),
            Resource::Manifest,
            Resource::Keystore("release".into()),
            Resource::AuditLog,
            Resource::CiPipeline,
            Resource::SecretsVault,
            Resource::SupabaseAdmin,
        ];
        let actions = Action::all();
        let roles = Role::canonical_list();

        // Build header row: Role | Resource:Action columns
        let mut col_headers: Vec<String> = vec![];
        for res in &canonical_resources {
            for act in actions {
                col_headers.push(format!("{} / {}", short_resource(res), act));
            }
        }

        let mut lines: Vec<String> = vec![];
        // Header
        lines.push(format!("| Role | {} |", col_headers.join(" | ")));
        // Separator
        let sep: Vec<&str> = std::iter::once("---")
            .chain(std::iter::repeat("---").take(col_headers.len()))
            .collect();
        lines.push(format!("|{}|", sep.join("|")));

        // Data rows
        for role in &roles {
            let mut cells: Vec<String> = vec![];
            for res in &canonical_resources {
                for act in actions {
                    let cell = match self.check(role, res, act) {
                        Decision::Allow => "✓".to_string(),
                        Decision::Deny { .. } => "✗".to_string(),
                        Decision::ConditionalAllow { .. } => "⚠ cond".to_string(),
                    };
                    cells.push(cell);
                }
            }
            lines.push(format!("| {} | {} |", role.display_name(), cells.join(" | ")));
        }

        lines.join("\n")
    }

    // ── Evaluate a full AccessRequest ─────────────────────────────────────────

    /// Evaluate an [`AccessRequest`] and return a timestamped [`AccessDecision`].
    pub fn evaluate(&self, request: AccessRequest) -> AccessDecision {
        let decision = self.check(&request.role, &request.resource, &request.action);
        AccessDecision { request, decision, evaluated_at: Utc::now() }
    }

    // ── Internal helpers ───────────────────────────────────────────────────────

    fn find_deny<'a>(
        &'a self,
        actor_role: &Role,
        resource: &Resource,
        action: &Action,
    ) -> Option<&'a ExplicitDeny> {
        self.denies.iter().find(|d| {
            self.role_matches(&d.role, actor_role)
                && self.resource_matches(&d.resource, resource)
                && d.action == *action
        })
    }

    /// Returns true if `pattern` (from a permission entry) matches `actual` (from a request).
    /// Wildcard entries (e.g. `UE4Project("*")`) match any resource of that variant.
    fn resource_matches(&self, pattern: &Resource, actual: &Resource) -> bool {
        match (pattern, actual) {
            (Resource::UE4Project(p), Resource::UE4Project(_)) => p == "*" || p == &actual.key().trim_start_matches("ue4project:").to_string(),
            (Resource::Keystore(p), Resource::Keystore(_)) => p == "*" || p == &actual.key().trim_start_matches("keystore:").to_string(),
            _ => pattern == actual,
        }
    }

    /// Returns true if `pattern` role matches `actual` role.
    /// `ServiceAccount("*")` matches any ServiceAccount.
    fn role_matches(&self, pattern: &Role, actual: &Role) -> bool {
        match (pattern, actual) {
            (Role::ServiceAccount(p), Role::ServiceAccount(_)) => p == "*" || p == match actual {
                Role::ServiceAccount(n) => n,
                _ => unreachable!(),
            },
            _ => pattern == actual,
        }
    }
}

fn short_resource(r: &Resource) -> &str {
    match r {
        Resource::UE4Project(_) => "UE4Proj",
        Resource::Manifest => "Manifest",
        Resource::Keystore(_) => "Keystore",
        Resource::AuditLog => "AuditLog",
        Resource::CiPipeline => "CI",
        Resource::SecretsVault => "Secrets",
        Resource::SupabaseAdmin => "SupaAdmin",
        Resource::Custom(_) => "Custom",
    }
}

// ── EnvironmentIsolation ──────────────────────────────────────────────────────

/// Environment isolation checker. Validates that the runtime context is appropriate
/// for the requested action, providing early-warning guardrails before `PolicyEngine`
/// is consulted.
pub struct EnvironmentIsolation;

impl EnvironmentIsolation {
    /// Heuristic: returns `true` when running in a production context.
    ///
    /// Production is detected via:
    /// - `ROCKET_ENV=production` environment variable, OR
    /// - `CI=true` AND the `GIT_BRANCH` / `GITHUB_REF` env vars indicate `main` or `master`.
    pub fn is_production() -> bool {
        if let Ok(env) = std::env::var("ROCKET_ENV") {
            if env.to_lowercase() == "production" {
                return true;
            }
        }
        let ci = std::env::var("CI").unwrap_or_default().to_lowercase() == "true";
        if ci {
            let branch = std::env::var("GIT_BRANCH")
                .or_else(|_| std::env::var("GITHUB_REF"))
                .unwrap_or_default();
            if branch.contains("main") || branch.contains("master") {
                return true;
            }
        }
        false
    }

    /// Check environment isolation rules for a given `env` label and target resource.
    ///
    /// Returns a list of warning strings. An empty vector means no isolation concerns.
    ///
    /// Rules:
    /// - Writing or deleting keystores outside `staging`/`production` envs warns about
    ///   cross-environment contamination.
    /// - Any write/delete to `SecretsVault` without `ROCKET_MFA_VERIFIED=true` warns.
    /// - Production writes to `SupabaseAdmin` always warn (human approval needed).
    pub fn check_isolation(env: &str, resource: &Resource) -> Vec<String> {
        let mut warnings: Vec<String> = vec![];
        let env_lower = env.to_lowercase();
        let is_prod = env_lower == "production";
        let is_dev = env_lower == "development" || env_lower == "dev" || env_lower == "local";

        match resource {
            Resource::Keystore(_) if is_dev => {
                warnings.push(
                    "Writing a Keystore in a development environment may overwrite signing \
                     credentials used in production builds. Ensure this is a separate \
                     development keystore (CC6.1)."
                    .to_string(),
                );
            }
            Resource::SecretsVault => {
                let mfa_verified =
                    std::env::var("ROCKET_MFA_VERIFIED").unwrap_or_default().to_lowercase()
                        == "true";
                if !mfa_verified {
                    warnings.push(
                        "SecretsVault access requires MFA verification. \
                         Set ROCKET_MFA_VERIFIED=true after authenticating (CC6.3)."
                        .to_string(),
                    );
                }
                if is_prod {
                    warnings.push(
                        "Production SecretsVault write requires dual-approval. \
                         Open a change request before proceeding (CC8.1)."
                        .to_string(),
                    );
                }
            }
            Resource::SupabaseAdmin if is_prod => {
                warnings.push(
                    "SupabaseAdmin operations in production require out-of-band human approval \
                     and must be logged in the audit trail (CC6.2, CC7.2)."
                    .to_string(),
                );
            }
            Resource::AuditLog if is_prod => {
                warnings.push(
                    "Direct modification of the AuditLog in production is prohibited. \
                     Use `rocket audit` to append entries through the approved chain (CC7.1)."
                    .to_string(),
                );
            }
            _ => {}
        }

        warnings
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn engine() -> PolicyEngine {
        PolicyEngine::new()
    }

    // ── 1. Developer permissions ───────────────────────────────────────────────

    #[test]
    fn developer_can_read_ue4_project() {
        let e = engine();
        let d = e.check(&Role::Developer, &Resource::UE4Project("ShooterGame".into()), &Action::Read);
        assert_eq!(d, Decision::Allow, "Developer must be able to read UE4 projects");
    }

    #[test]
    fn developer_can_write_ue4_project() {
        let e = engine();
        let d = e.check(&Role::Developer, &Resource::UE4Project("SurvivalGame".into()), &Action::Write);
        assert_eq!(d, Decision::Allow, "Developer must be able to write UE4 projects");
    }

    #[test]
    fn developer_cannot_delete_ue4_project() {
        let e = engine();
        let d = e.check(&Role::Developer, &Resource::UE4Project("Brm".into()), &Action::Delete);
        assert!(matches!(d, Decision::Deny { .. }), "Developer must not delete UE4 projects");
    }

    #[test]
    fn developer_cannot_access_secrets_vault() {
        let e = engine();
        let d = e.check(&Role::Developer, &Resource::SecretsVault, &Action::Read);
        assert!(matches!(d, Decision::Deny { .. }), "Developer must not access SecretsVault");
    }

    // ── 2. Admin has full access ───────────────────────────────────────────────

    #[test]
    fn admin_has_all_permissions() {
        let e = engine();
        let all_resources = vec![
            Resource::UE4Project("ShooterGame".into()),
            Resource::Keystore("release".into()),
            Resource::Manifest,
            Resource::AuditLog,
            Resource::SupabaseAdmin,
            Resource::CiPipeline,
            Resource::SecretsVault,
        ];
        for res in &all_resources {
            for act in Action::all() {
                let d = e.check(&Role::Admin, res, act);
                assert_eq!(
                    d,
                    Decision::Allow,
                    "Admin must have {act} on {:?}",
                    res
                );
            }
        }
    }

    // ── 3. ReadOnly restrictions ───────────────────────────────────────────────

    #[test]
    fn readonly_can_read_ue4_project() {
        let e = engine();
        let d = e.check(&Role::ReadOnly, &Resource::UE4Project("InfinityBlade4".into()), &Action::Read);
        assert_eq!(d, Decision::Allow);
    }

    #[test]
    fn readonly_cannot_write_anything() {
        let e = engine();
        let resources = vec![
            Resource::UE4Project("ShooterGame".into()),
            Resource::Manifest,
            Resource::AuditLog,
            Resource::CiPipeline,
            Resource::SecretsVault,
        ];
        for res in &resources {
            let d = e.check(&Role::ReadOnly, res, &Action::Write);
            assert!(
                matches!(d, Decision::Deny { .. }),
                "ReadOnly must not write {:?}",
                res
            );
        }
    }

    #[test]
    fn readonly_cannot_delete_anything() {
        let e = engine();
        let resources = vec![
            Resource::UE4Project("ShooterGame".into()),
            Resource::Manifest,
            Resource::Keystore("debug".into()),
        ];
        for res in &resources {
            let d = e.check(&Role::ReadOnly, res, &Action::Delete);
            assert!(matches!(d, Decision::Deny { .. }), "ReadOnly must not delete {:?}", res);
        }
    }

    // ── 4. Deny reason is non-empty ────────────────────────────────────────────

    #[test]
    fn deny_reason_is_non_empty() {
        let e = engine();
        let d = e.check(&Role::ReadOnly, &Resource::SecretsVault, &Action::Read);
        match d {
            Decision::Deny { reason } => assert!(!reason.is_empty(), "deny reason must not be empty"),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    // ── 5. Conditional allow carries conditions ────────────────────────────────

    #[test]
    fn devops_secrets_vault_read_is_conditional() {
        let e = engine();
        let d = e.check(&Role::DevOps, &Resource::SecretsVault, &Action::Read);
        match d {
            Decision::ConditionalAllow { conditions } => {
                assert!(!conditions.is_empty(), "conditional allow must carry conditions");
                assert!(
                    conditions.contains(&Condition::RequireMfa),
                    "SecretsVault DevOps read must require MFA"
                );
                assert!(
                    conditions.contains(&Condition::RequireVpn),
                    "SecretsVault DevOps read must require VPN"
                );
            }
            other => panic!("expected ConditionalAllow, got {:?}", other),
        }
    }

    // ── 6. DevOps can execute CI pipeline ─────────────────────────────────────

    #[test]
    fn devops_can_execute_ci_pipeline() {
        let e = engine();
        let d = e.check(&Role::DevOps, &Resource::CiPipeline, &Action::Execute);
        assert_eq!(d, Decision::Allow);
    }

    // ── 7. SecurityAuditor can read audit log ─────────────────────────────────

    #[test]
    fn security_auditor_can_read_audit_log() {
        let e = engine();
        let d = e.check(&Role::SecurityAuditor, &Resource::AuditLog, &Action::Read);
        assert_eq!(d, Decision::Allow);
    }

    #[test]
    fn security_auditor_can_audit_audit_log() {
        let e = engine();
        let d = e.check(&Role::SecurityAuditor, &Resource::AuditLog, &Action::Audit);
        assert_eq!(d, Decision::Allow);
    }

    #[test]
    fn security_auditor_cannot_write_anything() {
        let e = engine();
        let d = e.check(&Role::SecurityAuditor, &Resource::Manifest, &Action::Write);
        assert!(matches!(d, Decision::Deny { .. }));
    }

    // ── 8. Explicit deny on AuditLog deletion ─────────────────────────────────

    #[test]
    fn no_non_admin_role_can_delete_audit_log() {
        let e = engine();
        let non_admins = vec![
            Role::ReadOnly,
            Role::Developer,
            Role::LeadDeveloper,
            Role::DevOps,
            Role::SecurityAuditor,
        ];
        for role in &non_admins {
            let d = e.check(role, &Resource::AuditLog, &Action::Delete);
            assert!(
                matches!(d, Decision::Deny { .. }),
                "{:?} must not delete AuditLog",
                role
            );
        }
    }

    // ── 9. JSON policy override — additive grant ───────────────────────────────

    #[test]
    fn json_override_adds_grant() {
        // Developer cannot execute CI by default.
        let base = engine();
        let before = base.check(&Role::Developer, &Resource::CiPipeline, &Action::Execute);
        assert!(matches!(before, Decision::Deny { .. }));

        // Write a policy override that grants it.
        let overrides = serde_json::json!([
            { "role": "developer", "resource": "ci_pipeline", "action": "execute", "allow": true }
        ]);
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), overrides.to_string()).unwrap();
        let engine_with_override = PolicyEngine::load_policy(tmp.path()).unwrap();
        let after = engine_with_override.check(&Role::Developer, &Resource::CiPipeline, &Action::Execute);
        assert_eq!(after, Decision::Allow, "override must grant Developer CI execute");
    }

    #[test]
    fn json_override_adds_explicit_deny() {
        // LeadDeveloper can write Manifest by default.
        let base = engine();
        let before = base.check(&Role::LeadDeveloper, &Resource::Manifest, &Action::Write);
        assert_eq!(before, Decision::Allow);

        let overrides = serde_json::json!([
            { "role": "lead_developer", "resource": "manifest", "action": "write", "allow": false }
        ]);
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), overrides.to_string()).unwrap();
        let engine_with_deny = PolicyEngine::load_policy(tmp.path()).unwrap();
        let after = engine_with_deny.check(&Role::LeadDeveloper, &Resource::Manifest, &Action::Write);
        assert!(matches!(after, Decision::Deny { .. }), "explicit deny must override built-in allow");
    }

    #[test]
    fn json_override_preserves_existing_allows() {
        // After loading an override, existing grants should still work.
        let overrides = serde_json::json!([
            { "role": "developer", "resource": "ci_pipeline", "action": "execute", "allow": true }
        ]);
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), overrides.to_string()).unwrap();
        let e = PolicyEngine::load_policy(tmp.path()).unwrap();
        // Developer still reads UE4 projects.
        let d = e.check(&Role::Developer, &Resource::UE4Project("ShooterGame".into()), &Action::Read);
        assert_eq!(d, Decision::Allow, "existing grants must survive override load");
    }

    // ── 10. Matrix export contains all roles ──────────────────────────────────

    #[test]
    fn matrix_export_contains_all_canonical_roles() {
        let e = engine();
        let matrix = e.export_matrix();
        for role in Role::canonical_list() {
            assert!(
                matrix.contains(&role.display_name()),
                "matrix must contain role '{}'",
                role.display_name()
            );
        }
    }

    #[test]
    fn matrix_export_is_valid_markdown_table() {
        let e = engine();
        let matrix = e.export_matrix();
        let lines: Vec<&str> = matrix.lines().collect();
        // At least header + separator + one data row.
        assert!(lines.len() >= 3, "matrix must have at least 3 lines");
        assert!(lines[0].starts_with('|'), "header must start with '|'");
        assert!(lines[1].contains("---"), "separator must contain '---'");
    }

    // ── 11. Explain returns non-empty string with role/resource/action ─────────

    #[test]
    fn explain_includes_role_resource_action() {
        let e = engine();
        let explanation = e.explain(
            &Role::Developer,
            &Resource::UE4Project("ShooterGame".into()),
            &Action::Write,
        );
        assert!(explanation.contains("Developer"), "explanation must mention role");
        assert!(explanation.contains("UE4"), "explanation must mention resource");
        assert!(explanation.contains("write"), "explanation must mention action");
    }

    #[test]
    fn explain_deny_includes_reason() {
        let e = engine();
        let explanation =
            e.explain(&Role::ReadOnly, &Resource::SecretsVault, &Action::Write);
        assert!(explanation.contains("DENY"), "explanation must say DENY");
    }

    // ── 12. AccessRequest / AccessDecision serialisation ─────────────────────

    #[test]
    fn access_decision_serialises_to_json() {
        let e = engine();
        let req = AccessRequest {
            id: "req-001".to_string(),
            timestamp: Utc::now(),
            actor: "alice@example.com".to_string(),
            role: Role::Developer,
            resource: Resource::UE4Project("ShooterGame".into()),
            action: Action::Write,
        };
        let decision = e.evaluate(req);
        let json = serde_json::to_string(&decision).expect("must serialise");
        assert!(json.contains("req-001"));
        assert!(json.contains("alice@example.com"));
    }

    // ── 13. EnvironmentIsolation warnings ─────────────────────────────────────

    #[test]
    fn isolation_warns_on_keystore_write_in_dev() {
        let warnings =
            EnvironmentIsolation::check_isolation("development", &Resource::Keystore("debug".into()));
        assert!(!warnings.is_empty(), "must warn about keystore in dev env");
    }

    #[test]
    fn isolation_warns_on_supabase_admin_in_production() {
        let warnings =
            EnvironmentIsolation::check_isolation("production", &Resource::SupabaseAdmin);
        assert!(!warnings.is_empty(), "must warn about SupabaseAdmin in production");
    }

    #[test]
    fn isolation_no_warnings_for_ue4_project_read_in_dev() {
        let warnings = EnvironmentIsolation::check_isolation(
            "development",
            &Resource::UE4Project("ShooterGame".into()),
        );
        assert!(warnings.is_empty(), "reading UE4 project in dev must not warn");
    }

    // ── 14. ServiceAccount wildcard matching ──────────────────────────────────

    #[test]
    fn service_account_can_execute_ci_pipeline() {
        let e = engine();
        let d = e.check(
            &Role::ServiceAccount("ci-bot".into()),
            &Resource::CiPipeline,
            &Action::Execute,
        );
        assert_eq!(d, Decision::Allow, "any ServiceAccount must be able to trigger CI");
    }

    // ── 15. LeadDeveloper can manage keystores ─────────────────────────────────

    #[test]
    fn lead_developer_can_read_write_keystore() {
        let e = engine();
        for act in &[Action::Read, Action::Write] {
            let d = e.check(&Role::LeadDeveloper, &Resource::Keystore("release".into()), act);
            assert_eq!(d, Decision::Allow, "LeadDeveloper must {:?} Keystore", act);
        }
    }

    // ── 16. Decision::is_allowed helper ───────────────────────────────────────

    #[test]
    fn decision_is_allowed_returns_correct_values() {
        assert!(Decision::Allow.is_allowed());
        assert!(Decision::ConditionalAllow { conditions: vec![] }.is_allowed());
        assert!(!Decision::Deny { reason: "test".into() }.is_allowed());
    }

    // ── 17. Invalid policy file returns error ─────────────────────────────────

    #[test]
    fn load_policy_returns_error_for_invalid_json() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "this is not json").unwrap();
        let result = PolicyEngine::load_policy(tmp.path());
        assert!(result.is_err(), "must error on invalid JSON");
    }

    // ── 18. Resource key parsing round-trips ──────────────────────────────────

    #[test]
    fn resource_key_parsing_round_trips() {
        let pairs = vec![
            ("ue4project:shootergame", Resource::UE4Project("shootergame".into())),
            ("keystore:release", Resource::Keystore("release".into())),
            ("manifest", Resource::Manifest),
            ("auditlog", Resource::AuditLog),
            ("ci_pipeline", Resource::CiPipeline),
            ("secrets_vault", Resource::SecretsVault),
            ("supabase_admin", Resource::SupabaseAdmin),
        ];
        for (key, expected) in pairs {
            assert_eq!(parse_resource_key(key), expected, "key '{}' must parse correctly", key);
        }
    }

    // ── 19. Role key parsing ──────────────────────────────────────────────────

    #[test]
    fn role_key_parsing_works() {
        assert_eq!(parse_role_key("developer"), Role::Developer);
        assert_eq!(parse_role_key("admin"), Role::Admin);
        assert_eq!(parse_role_key("devops"), Role::DevOps);
        assert_eq!(parse_role_key("security_auditor"), Role::SecurityAuditor);
        assert_eq!(parse_role_key("readonly"), Role::ReadOnly);
    }

    // ── 20. Condition descriptions are non-empty ──────────────────────────────

    #[test]
    fn condition_descriptions_are_non_empty() {
        let conditions = vec![
            Condition::RequireMfa,
            Condition::RequireVpn,
            Condition::TimeWindow { start_hour: 9, end_hour: 17 },
            Condition::IpAllowlist(vec!["10.0.0.1".into()]),
        ];
        for cond in &conditions {
            assert!(!cond.description().is_empty(), "{:?} description must be non-empty", cond);
        }
    }

    // ── 21. Wildcard UE4Project matches any named project ─────────────────────

    #[test]
    fn wildcard_ue4project_matches_any_named_project() {
        let e = engine();
        let projects = ["ShooterGame", "SurvivalGame", "Brm", "InfinityBlade4", "FullSpectrum"];
        for proj in &projects {
            let d = e.check(&Role::Developer, &Resource::UE4Project(proj.to_string()), &Action::Read);
            assert_eq!(d, Decision::Allow, "Developer must read any UE4Project: {}", proj);
        }
    }
}
