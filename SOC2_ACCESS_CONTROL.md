# SOC2 Access Control — Integration Guide

| Attribute        | Value                          |
|------------------|--------------------------------|
| **Document ID**  | INT-AC-001                     |
| **Version**      | 1.0                            |
| **Status**       | DRAFT – Pending Approval       |
| **Effective Date** | 2026-06-20                   |

This document is the integration reference for the `access_control` module introduced in `tools/rocket-sdk/src/access_control.rs`. It covers how to wire the module into `lib.rs`, expected `main.rs` patterns for the two new `rocket policy` subcommands, SOC2 controls addressed, and pointers to the generated policy documents.

---

## 1. Wiring into `lib.rs`

Add the following declaration to `tools/rocket-sdk/src/lib.rs`:

```rust
pub mod access_control;
```

No additional Cargo dependencies are required — the module uses only crates already declared in `tools/rocket-sdk/Cargo.toml`:

| Crate        | Already in Cargo.toml? | Usage in `access_control` |
|--------------|------------------------|---------------------------|
| `serde`      | Yes (with `derive`)    | `Serialize`, `Deserialize` on all public types |
| `serde_json` | Yes                    | JSON policy file parsing, `AccessDecision` serialisation |
| `chrono`     | Yes (with `serde`)     | `DateTime<Utc>` in `AccessRequest` and `AccessDecision` |
| `anyhow`     | Yes                    | `Result<PolicyEngine>` from `load_policy()` |
| `thiserror`  | Yes                    | *(available; not used directly — `anyhow` used instead)* |

---

## 2. `rocket policy check` — CLI Snippet for `main.rs`

Add a `Policy` subcommand to the existing `clap` derive tree in `tools/rocket-cmd/src/main.rs`:

```rust
use rocket_sdk::access_control::{Action, PolicyEngine, Resource, Role};

/// Policy subcommand added to the top-level Cli enum:
///
/// ```
/// #[derive(Subcommand)]
/// enum Commands {
///     // ... existing commands ...
///     Policy(PolicyArgs),
/// }
/// ```

#[derive(Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub command: PolicyCommand,
}

#[derive(Subcommand)]
pub enum PolicyCommand {
    /// Check whether a role may perform an action on a resource.
    ///
    /// Example:
    ///   rocket policy check --role developer --resource ue4project:ShooterGame --action write
    Check {
        #[arg(long)]
        role: String,
        #[arg(long)]
        resource: String,
        #[arg(long)]
        action: String,
    },
    /// Export the full role × resource × action permission matrix as Markdown.
    ///
    /// Example:
    ///   rocket policy matrix
    Matrix,
}

/// Handler wired from Commands::Policy in the match block:
pub fn handle_policy(args: &PolicyArgs) -> anyhow::Result<()> {
    let engine = PolicyEngine::new();

    match &args.command {
        PolicyCommand::Check { role, resource, action } => {
            let r = rocket_sdk::access_control::parse_role_key_pub(role);
            let res = rocket_sdk::access_control::parse_resource_key_pub(resource);
            let act = rocket_sdk::access_control::parse_action_key_pub(action)
                .ok_or_else(|| anyhow::anyhow!("Unknown action: {}", action))?;

            let explanation = engine.explain(&r, &res, &act);
            println!("{explanation}");
        }
        PolicyCommand::Matrix => {
            println!("{}", engine.export_matrix());
        }
    }
    Ok(())
}
```

> **Note on parse helpers**: `parse_role_key`, `parse_resource_key`, and `parse_action_key`
> are currently private helpers in `access_control.rs`. To expose them for CLI use, add
> `pub` visibility or re-export them as thin wrappers:
>
> ```rust
> pub fn parse_role_key_pub(s: &str) -> Role { parse_role_key(s) }
> pub fn parse_resource_key_pub(s: &str) -> Resource { parse_resource_key(s) }
> pub fn parse_action_key_pub(s: &str) -> Option<Action> { parse_action_key(s) }
> ```

### Example CLI invocations

```bash
# Check whether a Developer may write a UE4 project:
rocket policy check --role developer --resource ue4project:ShooterGame --action write
# → Policy check — role: 'Developer', resource: 'UE4 Project (ShooterGame)', action: 'write' → ALLOW (unconditional grant in built-in role-permission matrix)

# Check whether ReadOnly can write the manifest:
rocket policy check --role readonly --resource manifest --action write
# → Policy check — role: 'Read-Only Viewer', resource: 'Project Manifest', action: 'write' → DENY — No grant found for role 'readonly' on resource 'manifest' with action 'write'

# Check DevOps access to SecretsVault:
rocket policy check --role devops --resource secrets_vault --action read
# → Policy check — role: 'DevOps Engineer', resource: 'Secrets Vault', action: 'read' → CONDITIONAL ALLOW — the following conditions must be satisfied: MFA authentication required; VPN connection required

# Export the full permission matrix for SOC2 evidence:
rocket policy matrix
```

---

## 3. SOC2 Controls Addressed

| SOC2 Control | Description                                     | Technical Implementation                                      |
|--------------|-------------------------------------------------|---------------------------------------------------------------|
| **CC6.1**    | Logical access security; least privilege        | `PolicyEngine::default()` built-in role matrix; `Role` enum enforces least privilege |
| **CC6.2**    | User access management; termination/transfer    | `AccessRequest` / `AccessDecision` audit trail; `load_policy()` for revocation overrides |
| **CC6.3**    | Role-based access and MFA enforcement           | `Condition::RequireMfa`, `Condition::RequireVpn` in `ConditionalAllow` decisions |
| **CC7.1**    | Monitoring; tamper-evident log chain            | `AccessDecision` → `audit_affidavit` BLAKE3 chain integration |
| **CC7.2**    | Audit log retention and review                  | `ExplicitDeny` on `AuditLog::Delete`; retention documented in CC7 policy |
| **CC7.3**    | Security event detection                        | `EnvironmentIsolation::check_isolation()` warnings; anomaly detection rules in CC7 policy |
| **CC7.4**    | Incident escalation matrix                      | Defined in `soc2-policies/INCIDENT-RESPONSE-POLICY.md`        |
| **CC7.5**    | Incident post-mortem and external notification  | Post-mortem template in Incident Response Policy; 72-hour GDPR alignment |
| **CC8.1**    | Change management; PR gates; CI enforcement     | `PolicyEngine` governs `CiPipeline` and `Manifest` write access; CC8 policy defines PR requirements |
| **CC9.1**    | Risk identification; SBOM; CVE review           | `cargo make audit` in `tools/`; SBOM generation process in CC9 policy |
| **CC9.2**    | Third-party vendor assessment                   | Supabase, GitHub Actions, Android SDK assessment in CC9 policy |

---

## 4. Generated Policy Documents

The following policy files in `soc2-policies/` are the authoritative SOC2 evidence documents:

| File | SOC2 Controls | Summary |
|------|--------------|---------|
| `soc2-policies/CC6-ACCESS-CONTROL-POLICY.md` | CC6.1, CC6.2, CC6.3 | Role definitions, permission matrix, provisioning, MFA, access reviews |
| `soc2-policies/CC7-MONITORING-POLICY.md` | CC7.1–CC7.5 | Audit log retention, chain integrity, alert thresholds, log review cadence |
| `soc2-policies/CC8-CHANGE-MANAGEMENT-POLICY.md` | CC8.1 | PR requirements, CI gate, semantic law compliance, deploy approval, rollback |
| `soc2-policies/CC9-RISK-MANAGEMENT-POLICY.md` | CC9.1, CC9.2 | SBOM, CVE review cadence, third-party vendor assessment, business continuity |
| `soc2-policies/INCIDENT-RESPONSE-POLICY.md` | CC7.3, CC7.4, CC7.5 | Severity taxonomy, detection sources, escalation matrix, 72-hour notification, post-mortem template |

---

## 5. Regenerating the Permission Matrix

The permission matrix in `CC6-ACCESS-CONTROL-POLICY.md` (section 3) must be regenerated whenever role or resource definitions change:

```bash
# From the monorepo root (after wiring pub mod access_control; into lib.rs):
rocket policy matrix > /tmp/matrix.md

# Review and paste the output into CC6-ACCESS-CONTROL-POLICY.md section 3.
```

The matrix is also available programmatically:

```rust
use rocket_sdk::access_control::PolicyEngine;
let matrix_markdown: String = PolicyEngine::new().export_matrix();
```

---

## 6. Policy Override File Format

To add site-specific grants or denials without modifying the built-in matrix:

```json
[
  { "role": "developer", "resource": "ci_pipeline", "action": "execute", "allow": true },
  { "role": "readonly",  "resource": "secrets_vault", "action": "read",  "allow": false }
]
```

Load via:

```rust
use rocket_sdk::access_control::PolicyEngine;
let engine = PolicyEngine::load_policy(std::path::Path::new("policy-overrides.json"))?;
```

Or via the planned CLI extension:

```bash
rocket policy check --policy policy-overrides.json --role developer --resource ci_pipeline --action execute
```

---

*This integration guide is subject to update whenever `access_control.rs` public API changes. Changes to this file require review by the LeadDeveloper and Security Auditor roles.*
