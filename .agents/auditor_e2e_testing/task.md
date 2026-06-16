# Forensic Auditor Task: Integrity Verification

## Objective
Independently audit the codebase, configuration changes, and verification outputs to ensure full integrity compliance.

## Key Audit Checks
1. Analyze the changes made in `pwa-staff/package.json` and `pwa-staff/playwright.config.ts`.
2. Inspect the E2E test files (e.g., `pwa-staff/tests-e2e/auth.spec.ts`) to ensure there are no hardcoded test results, mock behaviors that bypass the real database, dummy/facade implementations, or other shortcuts designed to fabricate a passing state.
3. Confirm that the verification logs and outcomes presented by the Worker and Reviewer are genuine and represent actual test runs against the real running server and local Supabase instance.
4. Report a binary verdict of CLEAN or INTEGRITY VIOLATION with detailed evidence.
