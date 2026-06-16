# Plan - Production Release Gaps for Rocket Craft PWA E2E

This plan outlines the steps to resolve remaining gaps for the production release of the Progressive Web App (PWA) with local Supabase integration and ensure 100% successful E2E testing with Playwright.

## Milestones

### Milestone 1: Playwright Configuration Modification
- Target File: `pwa-staff/playwright.config.ts`
- Objective: Modify the configuration to run E2E tests exclusively on the `chromium` browser project (removing `firefox` and `webkit` to avoid missing browser binary issues on the host system).
- Verification: Inspection of config file.

### Milestone 2: Fix Test Title Regex Expectation
- Target File: `pwa-staff/tests-e2e/example.spec.ts`
- Objective: Update the expected title regex match from `/PWA Staff/` to `/Rocket Craft/` to match the actual application title in `index.html`.
- Verification: Inspection of test file.

### Milestone 3: Run and Verify Vitest Unit Tests
- Workspace: `pwa-staff`
- Command: `npm run test` (which maps to `vitest run`)
- Verification: Verify unit tests execute and pass successfully.

### Milestone 4: Run and Verify Playwright E2E Tests
- Workspace: `pwa-staff`
- Command: `npx playwright test`
- Verification: Verify Playwright E2E tests execute and pass successfully on the Chromium browser. Check that the webServer command correctly boots the PWA on port 3000 during test execution and no browser configuration or launch errors are present.

## Subagent Dispatch Plan
We will dispatch a `teamwork_preview_worker` agent to perform these actions and verification.
1. The worker will modify the two target files in the `pwa-staff` folder.
2. The worker will run `npm run test` (Vitest) in the `pwa-staff` directory and verify it passes.
3. The worker will run `npx playwright test` (Playwright) in the `pwa-staff` directory and verify it passes.
4. The worker will report back with logs showing clean test execution.
