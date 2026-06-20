# CC6 — Logical and Physical Access Control Policy

| Attribute        | Value                          |
|------------------|--------------------------------|
| **Document ID**  | POL-CC6-001                    |
| **Version**      | 1.0                            |
| **Status**       | DRAFT – Pending Approval       |
| **Effective Date** | 2026-06-20                   |
| **Owner**        | [ORGANIZATION] Security Team   |
| **Review Cycle** | Quarterly                      |
| **SOC2 Controls**| CC6.1, CC6.2, CC6.3            |

---

## 1. Purpose and Scope

This policy establishes the logical access control framework for [ORGANIZATION]'s Rocket Craft monorepo, its associated Rust workspaces, TypeScript PWA (`pwa-staff`), and Supabase-backed services. It governs how access to system resources is granted, reviewed, and revoked in support of SOC2 Type II CC6.1 through CC6.3.

---

## 2. Role Definitions (CC6.1)

Access is granted strictly on the basis of organisational role. The following roles are defined and enforced programmatically by the `access_control` module in `tools/rocket-sdk/src/access_control.rs`:

| Role               | Description                                                                     |
|--------------------|---------------------------------------------------------------------------------|
| **ReadOnly**       | View UE4 project source and manifest; no modification rights.                   |
| **Developer**      | Read and write UE4 projects and associated assets; read manifest.               |
| **LeadDeveloper**  | All Developer rights plus manifest management, keystore access, UE4 project deletion. |
| **DevOps**         | Infrastructure operator; executes CI pipelines, reads keystores, and accesses secrets vault under MFA+VPN conditions. |
| **SecurityAuditor**| Read-only audit access to all resources; may invoke the `Audit` action on the audit log. |
| **Admin**          | Full administrative rights across all resources; restricted to named individuals. |
| **ServiceAccount** | Scoped machine identity (e.g., CI bot); limited to manifest read and CI pipeline execution. |

---

## 3. Permission Matrix (CC6.1)

The table below is the normative permission matrix enforced by `PolicyEngine::default()`. It is generated programmatically via `PolicyEngine::export_matrix()` and must be regenerated and resubmitted for approval upon any role or resource change.

> **Note:** Run `rocket policy matrix` to regenerate this table as a current SOC2 evidence artifact.

| Role               | UE4Proj / read | UE4Proj / write | UE4Proj / delete | Manifest / read | Manifest / write | Keystore / read | Keystore / write | AuditLog / read | AuditLog / audit | CI / execute | Secrets / read  | SupaAdmin / admin |
|--------------------|:--------------:|:---------------:|:----------------:|:---------------:|:----------------:|:---------------:|:----------------:|:---------------:|:----------------:|:------------:|:---------------:|:-----------------:|
| Read-Only Viewer   | ✓              | ✗               | ✗                | ✓               | ✗                | ✗               | ✗                | ✗               | ✗                | ✗            | ✗               | ✗                 |
| Developer          | ✓              | ✓               | ✗                | ✓               | ✗                | ✗               | ✗                | ✗               | ✗                | ✗            | ✗               | ✗                 |
| Lead Developer     | ✓              | ✓               | ✓                | ✓               | ✓                | ✓               | ✓                | ✓               | ✗                | ✗            | ✗               | ✗                 |
| DevOps Engineer    | ✓              | ✗               | ✗                | ✓               | ✗                | ✓               | ✗                | ✓               | ✗                | ✓            | ⚠ cond (MFA+VPN) | ✗                |
| Security Auditor   | ✓              | ✗               | ✗                | ✓               | ✗                | ✗               | ✗                | ✓               | ✓                | ✗            | ✗               | ✗                 |
| Administrator      | ✓              | ✓               | ✓                | ✓               | ✓                | ✓               | ✓                | ✓               | ✓                | ✓            | ✓               | ✓                 |

Legend: ✓ Allow · ✗ Deny · ⚠ cond = ConditionalAllow (conditions must be verified out-of-band)

---

## 4. Access Provisioning (CC6.1)

4.1 Access is provisioned through a formal request submitted to the Security Team via the internal ticketing system.

4.2 All provisioning requests must be approved by the individual's direct manager and, for roles of LeadDeveloper or above, the Head of Engineering.

4.3 The least-privilege principle applies: the minimum role that satisfies a business need must be assigned. Developers must not be granted DevOps or Admin roles absent a documented temporary-elevation procedure.

4.4 Service accounts are named and scoped to `ServiceAccount("<identifier>")` in the policy engine. Wildcard service account tokens are prohibited in production.

---

## 5. Multi-Factor Authentication (CC6.3)

5.1 MFA is mandatory for all human users accessing Supabase admin interfaces, secrets vault, and any production infrastructure.

5.2 The policy engine enforces MFA at the conditional-allow layer: `SecretsVault` read/write for DevOps roles returns `ConditionalAllow { conditions: [RequireMfa, RequireVpn] }`. Systems integrating the policy engine must verify these conditions before granting access.

5.3 The `ROCKET_MFA_VERIFIED=true` environment variable is the integration signal checked by `EnvironmentIsolation::check_isolation()`. It must only be set by an authenticated session handler.

---

## 6. Access Reviews (CC6.1, CC6.2)

6.1 Role assignments are reviewed quarterly by the Security Auditor role using `rocket audit`, which emits an affidavit-compatible BLAKE3-chained receipt.

6.2 Any role that has not been used within 90 days is automatically flagged for revocation in the quarterly review.

6.3 Admin role holders are reviewed monthly.

---

## 7. Termination and Transfer (CC6.2)

7.1 Upon employment termination, all access must be revoked within four (4) business hours. The Security Team is responsible for verifying removal from the identity provider and removing the principal's policy entries.

7.2 Role transfers require the old role to be explicitly revoked before the new role is granted to prevent privilege accumulation.

---

## 8. Technical Enforcement

This policy is technically enforced via:

- `tools/rocket-sdk/src/access_control.rs` — `PolicyEngine`, `Role`, `Resource`, `Action`, `Decision`
- `./rocket audit` — compliance check and BLAKE3-chained audit receipt generation
- `pwa-staff/src/auth.ts` — Supabase session gating; fires `auth-change` events consumed by all PWA modules
- `.github/workflows/ci.yml` — branch protection and CI gate enforce change control (see CC8 policy)

---

*This document is subject to quarterly review. Substantive changes require re-approval by the Head of Engineering and the designated SOC2 auditor.*
