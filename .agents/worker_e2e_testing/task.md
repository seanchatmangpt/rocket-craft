# Worker Task: Implement E2E Configuration and Run Tests

## Objective
Implement configuration changes to package.json and playwright.config.ts, build the project, run Playwright E2E tests, and verify success.

## Instructions
1. Modify `pwa-staff/package.json` start script to `"start": "local-web-server --port 3000"`.
2. Modify `pwa-staff/playwright.config.ts` to add a `webServer` block starting the app automatically.
3. Build the frontend client resources:
   ```bash
   npm run build
   ```
4. Run the Playwright E2E tests in `pwa-staff/tests-e2e/auth.spec.ts` against the running local server and local Supabase. Run using:
   ```bash
   npx playwright test tests-e2e/auth.spec.ts --project=chromium
   ```
5. Document execution results and output log in your handoff report (`/Users/sac/rocket-craft/.agents/worker_e2e_testing/handoff.md`).

## Verification
Ensure the tests run and pass 100% without errors.
