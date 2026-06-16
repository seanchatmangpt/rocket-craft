# Initial Codebase Exploration Report

## 1. Executive Summary
This report details the current state of the Rocket Craft codebase (specifically the PWA frontend, Supabase configuration, migrations, and edge functions). Significant gaps exist between the current mock implementation and the requirements, including missing database columns, lack of user sync triggers, incorrect HTML asset paths, and incomplete edge function logic.

---

## 2. PWA Frontend Source Code (`pwa-staff/src/`)

### `lib/supabaseClient.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/lib/supabaseClient.ts`
- **Current State**: Uses placeholders for `supabaseUrl` and `supabaseAnonKey`.
- **Code snippet**:
  ```typescript
  const supabaseUrl = process.env.SUPABASE_URL || 'YOUR_SUPABASE_URL'
  const supabaseAnonKey = process.env.SUPABASE_ANON_KEY || 'YOUR_SUPABASE_ANON_KEY'
  ```
- **Action Required**: Replace with local Supabase URL (`http://127.0.0.1:54321`) and the provided anon key.

### `auth.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/auth.ts`
- **Current State**: Implements a mock local storage auth store, entirely bypassing Supabase.
- **Action Required**: Re-implement to use the actual Supabase client authentication API (`supabase.auth`) or bridge the current interface to use Supabase sessions.

### `login.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/login.ts`
- **Current State**: Contains a basic listener on `#login-form` that calls `supabase.auth.signInWithPassword`.
- **Action Required**: Ensure it functions with the correct credentials and properly redirects.

### `signup.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/signup.ts`
- **Current State**: Contains a basic listener on `#signup-form` that calls `supabase.auth.signUp`.
- **Action Required**: Ensure it works with the updated trigger sync logic.

### `profile.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/profile.ts`
- **Current State**: Retrieves session using `supabase.auth.getSession()` synchronously.
- **Critical Bug**: `supabase.auth.getSession()` returns a `Promise`. Doing `const session = supabase.auth.getSession(); if (!session) { ... }` will always evaluate to false since the `Promise` object itself is truthy.
- **Action Required**: Await the `getSession()` call or use `supabase.auth.getUser()` to synchronously/asynchronously get session and handle redirection correctly.

### `admin.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/admin.ts`
- **Current State**: Queries `id, name, email` from the `players` table and tries to update them.
- **Mismatch**: The database schema for `players` only has `id`, `username`, and `created_at`. It lacks `name` and `email` columns.
- **Action Required**: Update the database schema to match this query, or adjust the query. (R3 specifies the database must support email and name/username).

### `leaderboard.ts`
- **Path**: `/Users/sac/rocket-craft/pwa-staff/src/leaderboard.ts`
- **Current State**: Fetches all scores from `leaderboard` and attempts to display `score.player_name`.
- **Mismatch**: The `leaderboard` table does NOT contain a `player_name` column; it contains a `player_id` foreign key.
- **Action Required**: Modify `fetchScores` to execute a select with a join on the `players` relation (e.g., `players(username)`) and map the output.

---

## 3. HTML Pages & Asset Paths (`pwa-staff/`)

### `login.html`, `signup.html`, `profile.html`
- **Paths**:
  - `/Users/sac/rocket-craft/pwa-staff/login.html`
  - `/Users/sac/rocket-craft/pwa-staff/signup.html`
  - `/Users/sac/rocket-craft/pwa-staff/profile.html`
- **Asset Paths Issue**: These files reference stylesheets and scripts via `../dist/style.css` and `../dist/*.js`. Since they are served from the root of the app, `../` goes outside the root, leading to 404 errors.
- **Action Required**: Fix asset paths to `./dist/` or `dist/`.

### `admin.html` and `leaderboard.html`
- **Paths**:
  - `/Users/sac/rocket-craft/pwa-staff/admin.html` (Correctly references `dist/style.css` and `dist/admin.js`)
  - `/Users/sac/rocket-craft/pwa-staff/leaderboard.html` (References `css/style.css` and `dist/leaderboard.js`)
- **Action Required**: Standardize CSS/JS links across all pages.

---

## 4. Supabase Configurations & Migrations (`supabase/`)

### Supabase Working Directory Layout
- **Location**: Supabase is initialized within `/Users/sac/rocket-craft/supabase/`. A nested subdirectory `/Users/sac/rocket-craft/supabase/supabase/` contains the `config.toml` file.
- **Implication**: Any Supabase CLI commands (e.g., `supabase start`, `supabase migration new`) must be run with a working directory of `/Users/sac/rocket-craft/supabase/`.

### Migrations
1. **`20240401000000_create_players_table.sql`**:
   - Creates the `players` table with columns: `id` (UUID PRIMARY KEY), `username` (VARCHAR(255) UNIQUE NOT NULL), and `created_at` (TIMESTAMPTZ).
2. **`20240401000001_create_game_sessions_table.sql`**:
   - Creates the `game_sessions` table with columns: `id`, `player_id` (foreign key), `score` (INTEGER), and `created_at`.
3. **`20240401000002_create_leaderboard_table.sql`**:
   - Creates the `leaderboard` table with columns: `id`, `player_id` (foreign key), `score` (INTEGER), `rank` (INTEGER), `created_at`, and `updated_at`. Contains an `updated_at` trigger.

### Schema Gaps / Gaps Identified:
- **Missing Columns**: The `players` table lacks `email` and `name` columns.
- **Missing Auth Sync Trigger**: There is currently no database trigger syncing newly registered users in `auth.users` to `public.players`.

---

## 5. Edge Functions (`supabase/functions/`)

### `submit-score/index.ts`
- **Path**: `/Users/sac/rocket-craft/supabase/functions/submit-score/index.ts`
- **Current State**: Standard Deno edge function skeleton. It performs validation (ensuring score is a number between 0 and 1000) but does NOT execute any database insertions or updates.
- **Action Required**: Initialize a Supabase client inside the edge function, parse the authorization header, fetch user details, insert a new row in `game_sessions`, and upsert/update the score in `leaderboard`.

### `get-player-rank/index.ts`
- **Path**: `/Users/sac/rocket-craft/supabase/functions/get-player-rank/index.ts`
- **Current State**: Fetches all `leaderboard` entries ordered by score descending, calculates the index of the requested `player_id`, and returns `rank: index + 1`. This functions as a helper but is not explicitly called by the requirements.

---

## 6. Local Server & E2E Testing Config

### Local Web Server
- **Path**: `/Users/sac/rocket-craft/pwa-staff/package.json`
- **Command**: `"start": "local-web-server"`
- **Config**: No default port config exists in `package.json`. If launched using `npm run start`, it runs on port 8000 by default.
- **Action Required**: Modify start script to serve on port 3000 (e.g., `local-web-server --port 3000`) to match Playwright expectations.

### Playwright Config
- **Path**: `/Users/sac/rocket-craft/pwa-staff/playwright.config.ts`
- **Configuration**:
  - `testDir`: `./tests-e2e`
  - `use.baseURL`: `http://localhost:3000`
  - Projects: Chromium, Firefox, Webkit.

### Playwright E2E Spec
- **Path**: `/Users/sac/rocket-craft/pwa-staff/tests-e2e/auth.spec.ts`
- **Test Details**:
  - Test Name: `'user authentication flow'`
  - Flow:
    1. Navigate to `/signup.html` -> fill email/password -> Submit.
    2. Wait for redirect to `**/profile.html`.
    3. Expect email text in page body.
    4. Click "Logout" -> Wait for redirect to `**/login.html`.
    5. Fill email/password -> Submit.
    6. Wait for redirect to `**/profile.html`.
    7. Expect email text in page body.
    8. Click "Logout" -> Wait for redirect to `**/login.html`.

---

## 7. Recommended Action Plan

1. **Database Migrations**:
   - Create a new migration file `20260615000000_update_schema_and_trigger.sql` inside `/Users/sac/rocket-craft/supabase/migrations/`.
   - Modify the `players` table to include `email` (VARCHAR(255)) and `name` (VARCHAR(255)) columns.
   - Implement a trigger function (e.g. `public.handle_new_user()`) that inserts a new record into `public.players` when a new record is created in `auth.users`, extracting the email and setting the name/username (e.g. using the email prefix).
   - Attach the trigger to `auth.users` (`AFTER INSERT`).

2. **Frontend Adjustments**:
   - Update `pwa-staff/src/lib/supabaseClient.ts` with local Supabase URL and anon key.
   - Update `pwa-staff/src/profile.ts` to fetch and handle the user session asynchronously (using `await`).
   - Standardize asset paths in `login.html`, `signup.html`, and `profile.html` to reference `dist/` or `./dist/` instead of `../dist/`.
   - Update `pwa-staff/src/leaderboard.ts` to perform a join relation query to fetch `username` from the `players` table, and use it in rendering the ranking table.
   - Modify the start script in `pwa-staff/package.json` to start on port 3000: `"start": "local-web-server --port 3000"`.

3. **Edge Function Integration**:
   - Implement Deno database query in `supabase/functions/submit-score/index.ts` to save score submissions.
