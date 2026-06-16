## 2026-06-15T22:49:57Z
You are a Worker agent (archetype: teamwork_preview_worker).
Your working directory is `/Users/sac/rocket-craft/.agents/worker_e2e_testing/`.
Your parent is conversation ID `24a37630-5370-426a-95af-f89bda39a1ef` (Recipient: 24a37630-5370-426a-95af-f89bda39a1ef).

Your task:
1. Initialize your BRIEFING.md and progress.md in `/Users/sac/rocket-craft/.agents/worker_e2e_testing/`.
2. Apply the following configuration changes:
   - In `pwa-staff/package.json`, update the "start" script to:
     `"start": "local-web-server --port 3000"`
   - In `pwa-staff/playwright.config.ts`, add the webServer configuration under the main configuration block:
     ```typescript
     webServer: {
       command: 'npm run start',
       url: 'http://localhost:3000',
       reuseExistingServer: !process.env.CI,
       stdout: 'ignore',
       stderr: 'pipe',
     },
     ```
3. Verify that the build works. Navigate to `pwa-staff` and run `npm run build`.
4. Run the Playwright test suite `tests-e2e/auth.spec.ts` against the chromium project using:
   `npx playwright test tests-e2e/auth.spec.ts --project=chromium`
   Make sure it is executing against the running local Supabase instance and the newly configured local server port 3000.
5. Record the exact output of the test run, verify if it passes 100%, and document this in your handoff report `handoff.md` in your working directory.
6. Send a completion message back to the parent conversation ID when done.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
