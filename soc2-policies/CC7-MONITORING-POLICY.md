# CC7 — System Monitoring and Anomaly Detection Policy

| Attribute        | Value                          |
|------------------|--------------------------------|
| **Document ID**  | POL-CC7-001                    |
| **Version**      | 1.0                            |
| **Status**       | DRAFT – Pending Approval       |
| **Effective Date** | 2026-06-20                   |
| **Owner**        | [ORGANIZATION] Security Team   |
| **Review Cycle** | Quarterly                      |
| **SOC2 Controls**| CC7.1, CC7.2, CC7.3, CC7.4, CC7.5 |

---

## 1. Purpose and Scope

This policy defines [ORGANIZATION]'s approach to continuous monitoring of the Rocket Craft monorepo, its CI/CD pipeline, Supabase-backed services, and associated infrastructure. It covers audit log retention, tamper-evident log chain integrity, anomaly detection, alert thresholds, and log review cadence in support of SOC2 Type II CC7.1 through CC7.5.

---

## 2. Audit Log Architecture (CC7.1)

2.1 All access decisions evaluated by the `PolicyEngine` in `tools/rocket-sdk/src/access_control.rs` produce `AccessDecision` records. These records are serialised to JSON and appended to the BLAKE3-chained audit log managed by `tools/rocket-sdk/src/audit_affidavit.rs`.

2.2 The chain uses `FORMAT_VERSION = "core/v1"` and a fixed genesis seed, producing a deterministic, tamper-evident receipt sequence. Any post-write modification to an entry changes the chain hash, which is verified at log-review time.

2.3 Each audit run via `./rocket audit` produces a timestamped affidavit receipt at `.ggen/receipts/affidavit-<YYYYMMDD-HHMMSS>.json` and updates `.ggen/receipts/latest.json`.

---

## 3. Audit Log Retention (CC7.2)

3.1 Audit log receipts must be retained for a minimum of **twelve (12) months** from the date of creation.

3.2 Receipts older than twelve months may be archived to long-term cold storage (e.g., encrypted S3 Glacier or equivalent) but must remain retrievable within 48 hours upon request.

3.3 Log deletion is explicitly denied for all non-Admin roles at the policy layer (`ExplicitDeny` on `Resource::AuditLog, Action::Delete`). Retention purges require Admin role and must be documented in a change request.

3.4 Supabase database logs and storage access logs are retained for twelve months via Supabase's built-in log retention settings, supplemented by weekly export to [ORGANIZATION]'s long-term storage.

---

## 4. Tamper-Evident Log Chain (CC7.1)

4.1 The BLAKE3 chain algorithm used in `audit_affidavit.rs` ensures that any insertion, deletion, or modification of an audit entry in the middle of the sequence causes a mismatch between the recomputed chain hash and the stored `chain_hash` field.

4.2 Log integrity is verified on a **weekly** basis by the SecurityAuditor running `./rocket audit` with the `--verify-chain` flag (to be implemented as part of the audit command enhancement described in the CC8 policy).

4.3 Detected chain breaks are treated as Critical security incidents (see the Incident Response Policy).

---

## 5. Security Monitoring (CC7.2)

5.1 The CI pipeline defined in `.github/workflows/ci.yml` runs on every push and includes:
   - ESLint and TypeScript type-check for `pwa-staff/`
   - `cargo build --all-features` and `cargo test --all-features` for `chicago-tdd-tools`
   - Semantic law compliance via `./rocket audit`

5.2 Secret scanning runs on every push via the `run_secret_scanning` GitHub Actions integration. Any detected secret causes an immediate pipeline failure and triggers a High-severity incident.

5.3 Supabase auth events (login, logout, token refresh, role escalation) are captured by `pwa-staff/src/auth.ts` and propagated as `auth-change` CustomEvents. Anomalous auth patterns (e.g., login from a new country, multiple failed attempts) trigger Supabase email alerts to the Security Team.

---

## 6. Alert Thresholds (CC7.3)

| Event                                      | Threshold      | Severity | Response Time |
|--------------------------------------------|----------------|----------|---------------|
| Policy engine `Deny` for Admin-level action| Any occurrence | Critical | 15 minutes    |
| Chain hash mismatch detected               | Any occurrence | Critical | 15 minutes    |
| Secret detected in source push             | Any occurrence | High     | 1 hour        |
| CI pipeline failure (non-test)             | Any occurrence | Medium   | 4 hours       |
| Supabase auth failure (same IP)            | ≥ 5 in 10 min  | Medium   | 4 hours       |
| Unused Admin credential                    | ≥ 30 days      | Low      | Next review   |
| Dependency with known CVE (critical/high)  | Any occurrence | High     | 24 hours      |

---

## 7. Log Review Cadence (CC7.4)

7.1 **Daily**: Automated CI pipeline runs serve as daily monitoring events. Failures are triaged by the on-call DevOps engineer.

7.2 **Weekly**: The SecurityAuditor reviews `.ggen/receipts/latest.json` for chain integrity and scans the `AccessDecision` records for anomalous `Deny` patterns that may indicate probing.

7.3 **Monthly**: The full audit log is reviewed for privilege escalation patterns, new service account usage, and access to `SecretsVault` or `SupabaseAdmin` resources. Findings are documented in the monthly security review ticket.

7.4 **Quarterly**: A full access review (see CC6 policy) cross-references log activity with current role assignments. Inactive accounts are flagged for revocation.

---

## 8. Anomaly Detection (CC7.5)

8.1 Repeated `Deny` decisions for the same actor across multiple resources within a 5-minute window are treated as a potential insider threat or credential compromise and escalate to a Medium incident.

8.2 Any `PolicyEngine.check()` call for `Resource::SecretsVault` or `Resource::SupabaseAdmin` is logged with full `AccessRequest` serialisation regardless of decision outcome. This ensures a complete audit trail for high-value resources.

8.3 `EnvironmentIsolation::check_isolation()` warnings (e.g., keystore write in development, Supabase admin in production) are emitted to the structured log with `WARN` level and surfaced in the weekly review.

---

## 9. Technical Enforcement

This policy is technically enforced via:

- `tools/rocket-sdk/src/audit_affidavit.rs` — BLAKE3-chained receipt generation
- `tools/rocket-sdk/src/access_control.rs` — `AccessDecision` serialisation for every policy check
- `./rocket audit` — compliance check that writes and verifies the chain
- `.github/workflows/ci.yml` — automated monitoring on every push
- Supabase Row Level Security (RLS) policies — enforce data isolation at the database layer

---

*This document is subject to quarterly review. Substantive changes require re-approval by the Head of Engineering and the designated SOC2 auditor.*
