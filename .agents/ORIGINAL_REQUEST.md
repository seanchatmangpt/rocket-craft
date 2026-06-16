# Original User Request

## Initial Request — 2026-06-15T14:34:16-07:00

Implement a working progressive web app (PWA) integrated with a local Supabase instance, including fully functioning user authentication, profiles, player management admin dashboard, leaderboard, and end-to-end testing with Playwright.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Supabase Client and PWA Authentication
Update `pwa-staff/src/lib/supabaseClient.ts` with the local Supabase URL (`http://127.0.0.1:54321`) and anon key (`sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`). Implement actual Supabase authentication in `pwa-staff/src/auth.ts`, `pwa-staff/src/login.ts`, `pwa-staff/src/signup.ts`, and `pwa-staff/src/profile.ts` using the Supabase JS client. Ensure that after sign-up or log-in, users are redirected to `profile.html` where their email is correctly displayed, and log-out redirects them back to `login.html`.

### R2. HTML Asset Paths
Fix relative asset paths in `login.html`, `signup.html`, and `profile.html` so they reference the generated dist directory as `dist/` or `./dist/` instead of `../dist/` since these files are served from the root.

### R3. DB Sync Trigger and Schema Alignment
Create a new migration or update migrations in `supabase/migrations` to sync `auth.users` to the `public.players` table upon user registration. The `public.players` table must support storing the player's email and name/username. Update `pwa-staff/src/admin.ts` to query and display player details from the `players` table. Update `pwa-staff/src/leaderboard.ts` to fetch leaderboard entries joined with the player's username so the leaderboard displays actual player names.

### R4. Edge Function Implementation
Implement the Supabase edge function `supabase/functions/submit-score/index.ts` to parse the request body, validate the score (must be a number between 0 and 1000), and save it into the database (`game_sessions` and update the player's high score in `leaderboard`).

### R5. Local Server & E2E Testing
Configure `local-web-server` or `npm run start` to serve the `pwa-staff` frontend on port 3000. Run the Playwright end-to-end test suite (`tests-e2e/auth.spec.ts`) against the running local server and local Supabase instance to verify the full registration, profile display, login, logout flow passes successfully.

## Acceptance Criteria

### Authentication & UI
- [ ] Sign-up, login, profile view, and logout flows are fully implemented using Supabase client in `pwa-staff/src`.
- [ ] HTML pages properly load CSS and JS bundles without 404 path errors.
- [ ] Profiles display the registered user's actual email from Supabase session.

### Database Sync & Edge Functions
- [ ] Registering a user triggers a Postgres trigger that inserts the user's ID, email, and username/email prefix into the `public.players` table.
- [ ] Admin dashboard successfully fetches and displays registered players list without Postgres column missing errors.
- [ ] Leaderboard page successfully displays player usernames and their high scores.
- [ ] Edge function `submit-score` writes records to `game_sessions` and updates `leaderboard` table.

### E2E Verification
- [ ] Playwright E2E test `user authentication flow` runs successfully and all steps pass.

## Follow-up — 2026-06-15T16:55:03-07:00

Resolve all remaining gaps for production release of the Progressive Web App (PWA) with local Supabase integration and ensure 100% successful end-to-end testing with Playwright.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Resolve Playwright E2E Test Failures and Browser Constraints
- Modify the Playwright configuration in `pwa-staff/playwright.config.ts` to run E2E tests exclusively on the `chromium` browser project (removing firefox and webkit to avoid missing browser binary issues on the host system).
- Fix the test in `pwa-staff/tests-e2e/example.spec.ts` by updating the expected title regex match from `/PWA Staff/` to `/Rocket Craft/` to match the actual application title in `index.html`.

### R2. Verify Application and Test Suite Health
- Verify that Vitest unit tests in the `pwa-staff` workspace run and pass.
- Verify that Playwright E2E tests run and pass without throwing browser configuration errors.

## Acceptance Criteria

### E2E Testing
- [ ] Playwright E2E tests execute and pass successfully on the Chromium browser.
- [ ] No browser launch or executable errors are present in the test logs.
- [ ] The webServer command correctly boots the PWA on port 3000 during test execution.

### Unit Testing
- [ ] Vitest unit tests in `pwa-staff/` pass successfully.

## Follow-up — 2026-06-15T17:31:19-07:00

Upgrade the Rocket Craft PWA launcher with a premium cyberpunk gamer-centric UI/UX, implement a collapsible in-app developer debug HUD (DX/QoL), and add database indexes and telemetry logging to the Supabase schema.

Working directory: /Users/sac/rocket-craft
Integrity mode: benchmark

## Requirements

### R1. Cyberpunk Gaming UI/UX
- Upgrade all PWA pages (`index.html`, `login.html`, `signup.html`, `profile.html`, `admin.html`, `leaderboard.html`) using vanilla CSS to implement a premium cyberpunk neon dark mode.
- Use glassmorphic card layouts, responsive layouts, glowing neon button hover effects, custom gaming-oriented typography, and subtle micro-animations.

### R2. Collapsible In-App Developer Console HUD
- Implement a collapsible developer debugging HUD available on all frontend pages when toggled (e.g., via a floating debug button in the corner).
- The HUD must display:
  - Active session details (decoded JWT values like email, user ID, role, and expiration timestamp).
  - Quick triggers to test mock score submissions.
  - Database stats fetched from backend endpoints (e.g., number of registered players and total game sessions).

### R3. Database Optimization & Telemetry Schema
- Create a new migration in `supabase/migrations/` that adds database indexes to:
  - `public.players(high_score DESC)`
  - `public.game_sessions(player_id, score)`
- Create a `public.telemetry_logs` table in the migration:
  - Fields: `id` (uuid primary key), `player_id` (foreign key to `players.id`, nullable for unauthenticated events), `event_type` (text, e.g., 'login', 'registration', 'profile_view', 'score_submission'), `payload` (jsonb), and `created_at` (timestamp with timezone).
- Integrate backend client logic to log records into `public.telemetry_logs` whenever a player registers, logs in, views their profile, or submits a score.

### R4. Verification & Testing
- Update Vitest unit tests in `pwa-staff/` to cover new helper functions and console components.
- Update Playwright E2E tests to verify that the Developer Debug HUD is present, can be toggled open, and that new page layouts load without JavaScript console errors.

## Acceptance Criteria

### PWA UI/UX
- [ ] All pages render with the new cyberpunk neon dark theme, including layout grids, forms, tables, and buttons.
- [ ] Responsive UI fits mobile, tablet, and desktop screens with zero overlapping text.

### Developer HUD (DX/QoL)
- [ ] Collapsible debug panel is present on all pages and can be toggled.
- [ ] HUD displays decoded JWT state when user is logged in, and shows an unauthenticated state when logged out.
- [ ] Stats display correct count of registered players.

### Database & Telemetry
- [ ] Supabase schema contains the new indexes on `players` and `game_sessions`, and the `telemetry_logs` table.
- [ ] Performing auth operations (signup, login, logout), profile views, and score submissions creates corresponding rows in `public.telemetry_logs`.

### Test Suite Execution
- [ ] Vitest unit tests in `pwa-staff/` execute and pass successfully.
- [ ] Playwright E2E tests execute and pass successfully on Chromium.
