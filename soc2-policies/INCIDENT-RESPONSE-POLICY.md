# Incident Response Policy

| Attribute        | Value                          |
|------------------|--------------------------------|
| **Document ID**  | POL-IR-001                     |
| **Version**      | 1.0                            |
| **Status**       | DRAFT – Pending Approval       |
| **Effective Date** | 2026-06-20                   |
| **Owner**        | [ORGANIZATION] Security Team   |
| **Review Cycle** | Quarterly                      |
| **SOC2 Controls**| CC7.3, CC7.4, CC7.5            |

---

## 1. Purpose and Scope

This policy defines how [ORGANIZATION] identifies, classifies, responds to, and recovers from security and operational incidents affecting the Rocket Craft monorepo, its CI/CD pipeline, Supabase-backed services, and associated infrastructure. It aligns with SOC2 Type II CC7.3 through CC7.5 and incorporates a 72-hour breach notification requirement consistent with GDPR Article 33.

---

## 2. Incident Severity Taxonomy (CC7.3)

| Severity     | Definition                                                                                  | Initial Response | External Notification |
|--------------|---------------------------------------------------------------------------------------------|------------------|-----------------------|
| **Critical** | Confirmed data breach, production credential leak, audit chain tampering, unauthorised Admin access | 15 minutes   | Within 72 hours (GDPR) |
| **High**     | Secret detected in source push, critical CVE in production dependency, CI gate bypass          | 1 hour           | Within 72 hours if PII involved |
| **Medium**   | Repeated policy `Deny` anomalies, Supabase auth anomaly, non-critical CI failure, Medium CVE | 4 hours          | Not required unless escalated |
| **Low**      | Unused credential, misconfigured environment warning, Low CVE, policy documentation gap       | Next business day | Not required          |

---

## 3. Detection Sources (CC7.3)

3.1 **Audit Log** — `AccessDecision` records serialised by `tools/rocket-sdk/src/access_control.rs` and chained via `audit_affidavit.rs`. Anomalous `Deny` patterns, unexpected `ConditionalAllow` violations, or chain hash mismatches trigger immediate investigation.

3.2 **CI Alerts** — `.github/workflows/ci.yml` failures, secret scan findings, and `./rocket audit` law violations are automatically posted to the `#ci-alerts` notification channel.

3.3 **User Report** — Any team member who suspects a security incident must report it immediately to `security@[ORGANIZATION]` or the `#security-incidents` channel with the subject line `[INCIDENT] <brief description>`.

3.4 **Supabase Auth Events** — Failed login bursts, new-country logins, and unexpected role escalations captured by `pwa-staff/src/auth.ts` are propagated to the Security Team via Supabase webhook alerts.

3.5 **External Discovery** — Responsible disclosure reports received at the published security contact are treated as High severity or above pending investigation.

---

## 4. Escalation Matrix (CC7.4)

| Role               | Responsibility in an Incident                                                    |
|--------------------|----------------------------------------------------------------------------------|
| **First Responder** (any engineer) | Detect and report; do not attempt to remediate without Security Team involvement. |
| **DevOps Engineer** | Isolate affected systems, preserve forensic evidence, coordinate containment.    |
| **Security Auditor** | Verify audit chain integrity via `./rocket audit`, assess scope of compromise.  |
| **LeadDeveloper**   | Coordinate code-level remediation, emergency patching, rollback decisions.       |
| **Head of Engineering** | Decision authority for external notifications and post-mortem scheduling.   |
| **Legal / DPO**     | Engaged for any incident with potential GDPR or contractual notification obligations. |

**Critical incidents** immediately engage DevOps, Security Auditor, and Head of Engineering in parallel. **High incidents** engage DevOps and Security Auditor; Head of Engineering is notified within 1 hour.

---

## 5. Response Procedures (CC7.4)

### 5.1 Identification and Triage (0–15 min for Critical)

1. Acknowledge the incident report; assign an incident commander (first available Security Team member).
2. Classify severity using the taxonomy in Section 2.
3. Open an incident ticket in the internal system with label `incident:<severity>` and timestamp.
4. Preserve evidence: do not delete logs, do not restart affected services before forensic capture.

### 5.2 Containment (15 min–1 hour for Critical)

1. For credential leaks: rotate the affected credential immediately using `./rocket crypto generate` (for keystores) or Supabase dashboard (for Supabase keys). Revoke old credential.
2. For unauthorised access: invoke `PolicyEngine.load_policy()` to load an emergency deny override file that explicitly denies the compromised principal's role across all resources.
3. For audit chain tampering: quarantine the affected receipt file; roll back to the last verified `latest.json`; treat all access decisions since the last verified checkpoint as suspect.
4. For CI compromise: disable the affected GitHub Actions workflow; switch to manual deploy gate until integrity is confirmed.

### 5.3 Eradication and Recovery

1. Identify root cause using audit log records and git history.
2. Apply code fix via an emergency pull request (single-reviewer, 30-minute window per CC8 policy section 7.4).
3. Re-run `./rocket audit` to confirm law compliance after the fix.
4. Restore service and monitor for 24 hours for recurrence.

### 5.4 72-Hour Notification Requirement (GDPR CC7.5)

If the incident involves personal data of EU/EEA data subjects (including any data in the Supabase-backed PWA), [ORGANIZATION] must notify the relevant supervisory authority within **72 hours** of becoming aware of the breach. Notification must include:

- Nature and categories of data affected.
- Approximate number of data subjects and records affected.
- Contact details of the Data Protection Officer.
- Likely consequences and measures taken to address the breach.

The Head of Engineering and Legal/DPO are jointly responsible for preparing and filing this notification.

---

## 6. Post-Mortem Process (CC7.5)

A post-mortem is required for all Critical and High incidents and is optional but recommended for Medium incidents. It must be completed within **5 business days** of incident resolution.

### Post-Mortem Template

```markdown
## Incident Post-Mortem

**Incident ID**: INC-<YYYY>-<NNN>
**Date**: YYYY-MM-DD
**Severity**: Critical / High / Medium
**Duration**: <start time> – <resolution time> UTC
**Incident Commander**: <name>

### Summary
<2–3 sentence plain-language description of what happened and its impact.>

### Timeline
| Time (UTC) | Event |
|------------|-------|
| HH:MM      | Incident detected via <audit log / CI alert / user report> |
| HH:MM      | Incident commander engaged |
| HH:MM      | Containment action taken: <description> |
| HH:MM      | Root cause identified |
| HH:MM      | Fix deployed via PR #<N> |
| HH:MM      | Incident resolved; monitoring resumed |

### Root Cause
<Technical explanation of what failed and why.>

### Impact
<Systems affected, data exposed, users impacted, duration of degradation.>

### Remediation Actions
- [ ] Immediate: <action taken>
- [ ] Short-term (within 1 week): <action>
- [ ] Long-term (within 1 quarter): <action>

### Process Improvements
<Changes to policy, tooling, or monitoring that prevent recurrence.>

### SOC2 Controls Referenced
CC7.3 / CC7.4 / CC7.5 — as applicable.

### External Notifications
- [ ] GDPR supervisory authority notified: Yes / No / N/A — Date: YYYY-MM-DD
- [ ] Affected users notified: Yes / No / N/A — Date: YYYY-MM-DD
```

---

## 7. Policy Testing (CC7.5)

7.1 The incident response procedure is tested annually via a tabletop exercise. The Security Auditor selects a hypothetical scenario (e.g., credential leak in CI, audit chain tamper) and walks the team through the response steps.

7.2 The ability to restore from backup (audit receipts, project manifest, Supabase schema) is tested quarterly as part of the business continuity review (CC9 policy).

---

## 8. Technical Enforcement

This policy is technically enforced via:

- `tools/rocket-sdk/src/access_control.rs` — `PolicyEngine.load_policy()` for emergency deny overrides
- `tools/rocket-sdk/src/audit_affidavit.rs` — tamper-evident chain for forensic integrity
- `./rocket audit` — chain verification and law compliance check
- `.github/workflows/ci.yml` — secret scan and automated CI gate
- `pwa-staff/src/auth.ts` — Supabase auth event capture and anomaly hooks

---

*This document is subject to quarterly review. Substantive changes require re-approval by the Head of Engineering and the designated SOC2 auditor.*
