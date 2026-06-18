## 2026-06-15T23:56:27Z

Your objective is to resolve the remaining gaps for the production release of the Progressive Web App (PWA) with local Supabase integration and ensure 100% successful end-to-end testing with Playwright.

Your working directory is `/Users/sac/rocket-craft/.agents/worker_production_release_gaps`.

Please follow these instructions:
1. Modify the Playwright configuration in `pwa-staff/playwright.config.ts` to run E2E tests exclusively on the `chromium` browser project (removing firefox and webkit to avoid missing browser binary issues on the host system).
2. Fix the test in `pwa-staff/tests-e2e/example.spec.ts` by updating the expected title regex match from `/PWA Staff/` to `/Rocket Craft/`.
3. Verify that Vitest unit tests in the `pwa-staff` workspace run and pass. Run `npm run test` (which executes `vitest run`) in the `/Users/sac/rocket-craft/pwa-staff` directory.
4. Verify that Playwright E2E tests run and pass without throwing browser configuration errors. Run `npx playwright test` in the `/Users/sac/rocket-craft/pwa-staff` directory.

Scope Boundaries:
- Modify only `pwa-staff/playwright.config.ts` and `pwa-staff/tests-e2e/example.spec.ts`.
- Ensure all other tests are unaffected.

Verification and Handoff Requirements:
- You must run the Vitest unit tests and Playwright E2E tests and verify they pass.
- Write a detailed handoff report in `/Users/sac/rocket-craft/.agents/worker_production_release_gaps/handoff.md` with:
  - Exact modifications made to both files.
  - The build and test command outputs.
  - Attestation of clean execution.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## 2026-06-17T19:35:28Z

Your task is to implement all recommended changes from the Gap Analysis Report at /Users/sac/rocket-craft/.agents/explorer_1/handoff.md to complete the Rocket-Craft ecosystem integration.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Please execute the following steps:

1. R1/R2: PWA Frontend Canvas & Receipt Integration
   - Edit `pwa-staff/profile.html` to add the canvas (`#canvas`), coordinates (`#coords`), and cryptographic receipt details (`#receipt-details`) inside a styled flex dashboard layout in the main `.container` card. Make sure the container card style has a wider max-width (e.g. `max-width: 1200px; width: 95%;` or inline style) so everything fits comfortably.
   - Edit `pwa-staff/src/profile.ts` to:
     - Fetch `/api/spec`.
     - Display the latest receipt details (hash, timestamp/issued_at) in `#receipt-details`.
     - Persist the spec JSON to Supabase `world_specs` table:
       ```typescript
       const { error } = await supabase.from('world_specs').upsert({
         player_id: user.id,
         spec: spec,
         updated_at: new Date().toISOString()
       }, { onConflict: 'player_id' });
       ```
       Ensure any errors from this upsert are logged to `console.error` and do not block the app.
     - Dynamically append the script `/manufactured/Brm-HTML5-Shipping.js` to the page only after user auth has been validated, to start the engine simulation on `#canvas`.
   - Edit `/Users/sac/ue4-sim/Engine/Templates/HTML5/Brm-HTML5-Shipping.js` fetches:
     - Update relative paths `Brm-HTML5-Shipping.data` -> `/manufactured/Brm-HTML5-Shipping.data`
     - Update relative paths `Brm-HTML5-Shipping.wasm` -> `/manufactured/Brm-HTML5-Shipping.wasm`

2. R2: Supabase Database Migration
   - Create a new migration file `supabase/migrations/20260617000000_create_world_specs.sql` with table `public.world_specs`:
     - Columns: `id` (UUID primary key with default gen_random_uuid()), `player_id` (UUID unique, foreign key to players.id/users.id), `spec` (JSONB not null default '{}'), `updated_at` (TIMESTAMPTZ default now()).
     - Add index on `player_id`.
     - Grant permissions: `GRANT ALL ON public.world_specs TO anon, authenticated, service_role;`
   - Apply the migration to the local Supabase container if needed (e.g., you can run a SQL command on the postgres container via docker psql command or the local server will read it on restart/init, but applying it to the database is best. Check how the other tables are migrated, or write to database using docker exec psql).

3. R3: Multiplatform Standalone Builds
   - Edit `/Users/sac/ue4-sim/Engine/Build/BatchFiles/run_uat.py` to:
     - Parse `-platform` argument.
     - If `-platform=Win64`, create staged directory `Saved/StagedBuilds/Win64` and write mock executable `Brm-Windows-Shipping.exe`.
     - If `-platform=Linux`, create staged directory `Saved/StagedBuilds/Linux` and write mock binary `Brm-Linux-Shipping.sh`.
     - If `-platform=HTML5`, perform the original HTML5 staging.
   - Edit `unify-rs/unify-wasm/src/packager.rs` to implement `package_windows` and `package_linux` copying files from Win64/Linux staging dirs to the destination.
   - Edit `unify-rs/genie-core/src/deployment.rs` to call `package_windows` and `package_linux` on deploy.

4. R4: Service Worker Caching & Offline Support
   - Edit `pwa-staff/worker.ts`:
     - Fix `STATIC_ASSETS` to prefix `/manufactured/` files:
       `'/manufactured/Brm-HTML5-Shipping.html'`, `'/manufactured/Brm-HTML5-Shipping.js'`, `'/manufactured/Brm-HTML5-Shipping.wasm'`, `'/manufactured/Brm-HTML5-Shipping.data'`, `'/manufactured/receipt.json'`
     - Intercept fetches to `/manufactured/*` using Network First strategy, falling back to cache if offline.
   - Run `npm run build` in `pwa-staff/` to compile all TypeScript files (`src/profile.ts`, `worker.ts`, `cache.ts`).

5. Verification:
   - Run `cargo build` in `unify-rs/`.
   - Run the verify script `./verify_html5_pipeline.sh` (or `verify_genie.sh` or playright tests directly) to trigger deployment and verify the HTML5 builds.
   - Add/update E2E tests in `pwa-staff/tests-e2e/` (or write a new `persistence.spec.ts`) to:
     - Register mock user, load profile.html, assert canvas mounts and receipt displays.
     - Assert that WorldSpec JSON was saved under correct user ID in Supabase `world_specs` table.
     - Set page offline, reload, and verify that the canvas still mounts/loads successfully.
   - Ensure all Vitest, Deno, and Playwright tests pass cleanly.

Write your implementation report to your handoff.md file when completed.
