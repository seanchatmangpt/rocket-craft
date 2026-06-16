# Original User Request

## Initial Request — 2026-06-15T22:47:28Z

You are a Sub-orchestrator for Milestone 5: E2E Testing & Verification.
Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing/`.
Your parent is conversation ID `51eb4be3-e539-4e5f-87d9-4d687e04cd83` (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Your mission:
Configure the local server and verify the full application behavior using Playwright E2E tests:
1. Ensure the start script in `pwa-staff/package.json` starts the local-web-server on port 3000 (`local-web-server --port 3000`).
2. Verify that Playwright E2E tests run against the local server served on port 3000. Check if the `playwright.config.ts` starts the webServer automatically or if you need to run it in the background/foreground.
3. Run the Playwright E2E test suite `pwa-staff/tests-e2e/auth.spec.ts` against the running local server and the running local Supabase instance.
4. Verify that the user authentication flow (signup -> profile -> logout -> login -> profile -> logout) passes successfully 100%.

Please perform the following steps:
1. Create your `BRIEFING.md` and `progress.md` files in your working directory `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing/`.
2. Create your `SCOPE.md` defining this milestone, the interfaces, and the changes to be made.
3. Run the iteration loop:
   - Spawn an Explorer to review `playwright.config.ts`, `package.json`, and current E2E test code, and design execution/configuration updates.
   - Spawn a Worker to implement changes, launch the local server, run the Playwright tests, and document test output. Include the MANDATORY INTEGRITY WARNING in the worker's prompt.
   - Spawn a Reviewer to verify test results, logs, and port settings.
   - Spawn a Challenger and Forensic Auditor to verify integration and check for integrity.
4. When the gate passes, write `handoff.md` and send a message reporting status back to parent (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Never write or edit code files directly; always delegate to workers. You may write to metadata files inside your folder `/Users/sac/rocket-craft/.agents/sub_orch_e2e_testing/`.
