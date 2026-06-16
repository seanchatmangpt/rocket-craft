# Scope: Milestone 5: E2E Testing & Verification

## Architecture
- **PWA Staff App**: React-based PWA application located in `pwa-staff`.
- **Local Web Server**: Starts using `local-web-server --port 3000` (via package.json script).
- **Playwright Test Runner**: Runs tests in `pwa-staff/tests-e2e/auth.spec.ts` against the web server at port 3000 and the local Supabase instance.
- **Supabase Instance**: Local Supabase services (Auth, Database) running and reachable.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Explore Config | Investigate `pwa-staff/package.json`, `pwa-staff/playwright.config.ts`, and `auth.spec.ts` | None | DONE |
| 2 | Implement & Run | Configure package.json start script, set up server on port 3000, run Playwright tests | M1 | DONE |
| 3 | Reviewer Check | Review test outputs, verify logs, verify port settings | M2 | DONE |
| 4 | Challenger & Audit | Run challenger test checks and perform forensic audit verification | M3 | DONE |

## Interface Contracts
- **PWA App Port**: 3000 (`http://localhost:3000`)
- **Authentication Flow**: User Signup -> Profile Creation -> Logout -> Login -> Profile Verification -> Logout.
- **Supabase Connectivity**: E2E tests must be configured to point to the local Supabase environment (using appropriate environment variables or config).
