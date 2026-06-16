# Explorer Handoff Report

## 1. Observation
- Verified that typescript files in `pwa-staff/src` have these properties:
  - `pwa-staff/src/lib/supabaseClient.ts:3-4` contains:
    ```typescript
    const supabaseUrl = process.env.SUPABASE_URL || 'YOUR_SUPABASE_URL'
    const supabaseAnonKey = process.env.SUPABASE_ANON_KEY || 'YOUR_SUPABASE_ANON_KEY'
    ```
  - `pwa-staff/src/auth.ts:16-18` stores local storage keys for mock auth:
    ```typescript
    let currentSession: Session | null = null;
    const SESSION_STORAGE_KEY = 'rocket-craft-session';
    ```
  - `pwa-staff/src/profile.ts:6-9` uses synchronous checking of Promise output:
    ```typescript
    const session = supabase.auth.getSession()
    if (!session) {
    ```
  - `pwa-staff/src/admin.ts:89-91` queries non-existent database columns `name` and `email` on `players`:
    ```typescript
    const { data, error } = await supabase
        .from('players')
        .select('id, name, email');
    ```
  - `pwa-staff/src/leaderboard.ts:12-15` queries `leaderboard` table without a join, trying to read `player_name` which is not present in the table schema:
    ```typescript
    const { data: scores, error } = await supabase
        .from('leaderboard')
        .select('*')
    ```
- Verified that relative asset paths in `login.html`, `signup.html`, and `profile.html` reference `../dist/`:
  - `pwa-staff/login.html:7`: `<link rel="stylesheet" href="../dist/style.css">`
  - `pwa-staff/signup.html:25`: `<script type="module" src="../dist/signup.js"></script>`
- Verified migrations under `supabase/migrations/`:
  - `20240401000000_create_players_table.sql` creates:
    ```sql
    CREATE TABLE players (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        username VARCHAR(255) UNIQUE NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    ```
  - `20240401000002_create_leaderboard_table.sql` creates:
    ```sql
    CREATE TABLE leaderboard (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        player_id UUID REFERENCES players(id) ON DELETE CASCADE,
        score INTEGER NOT NULL,
        rank INTEGER,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NOW()
    );
    ```
- Verified `supabase/functions/submit-score/index.ts` is a Deno server function which has:
  - Line 38: `// In a real application, you would save the score to your database here.` and returns a mock JSON response with status 200 without saving data.
- Verified that:
  - `pwa-staff/package.json` script `"start"` is `"local-web-server"`.
  - `pwa-staff/playwright.config.ts` has `use.baseURL: 'http://localhost:3000'`.
  - `pwa-staff/tests-e2e/auth.spec.ts` expects `/signup.html`, `/profile.html`, and `/login.html` URLs under baseURL.

## 2. Logic Chain
1. **Asset Paths**: Because `login.html`, `signup.html`, and `profile.html` are served from the root of `pwa-staff/`, referencing resources using `../dist/` targets a parent directory of the server root, which will cause the browser to fail to fetch them (404). Thus, the paths must be updated to `./dist/` or `dist/`.
2. **Database Schema & Admin Logic**: Since `admin.ts` queries `players` for `name` and `email` columns, and the database schema (`20240401000000_create_players_table.sql`) only has `id`, `username`, and `created_at`, the query will fail. Therefore, the database table must be expanded to support name/email, and a registration trigger is needed to automatically populate them.
3. **Database Schema & Leaderboard Logic**: Since `leaderboard.ts` expects to display `score.player_name` and the `leaderboard` table lacks this column, the client must perform a joined select query on `players` using `player_id` to retrieve the `username`.
4. **Edge Function**: Since `submit-score/index.ts` contains only a validation check and returns a mock success response, it must be updated to insert scores into `game_sessions` and upsert high scores into `leaderboard`.
5. **Local Web Server**: Since the Playwright config expects baseURL to be `http://localhost:3000` but `local-web-server` defaults to port 8000, `local-web-server` must be run with `--port 3000`.

## 3. Caveats
- No caveats. The codebase has been fully explored.

## 4. Conclusion
The codebase is currently in a pre-integration state:
- Static HTML asset paths point to incorrect relative URLs.
- The `players` schema does not match the frontend `admin.ts` model.
- Database sync triggers for registering users are completely missing.
- Leaderboard page queries need to be updated to retrieve usernames via a join.
- The `submit-score` edge function is not hooked up to database storage.
- Local server port configurations must be aligned to port 3000 to enable E2E Playwright test passing.

## 5. Verification Method
- Inspect the generated report at `/Users/sac/rocket-craft/.agents/orchestrator/initial_exploration.md` to confirm all sections match the findings above.
