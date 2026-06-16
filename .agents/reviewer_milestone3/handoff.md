# Milestone 3 Review & Handoff Report

## 1. Review Summary
- **Verdict**: REQUEST_CHANGES
- **Overall Risk Assessment**: HIGH (due to XSS and broken RLS policies)

---

## 2. Findings (Quality Review)

### [Critical] Finding 1: Cross-Site Scripting (XSS) in Player List Rendering
- **What**: HTML injection via `player.name` and `player.email` inside `table.innerHTML`.
- **Where**: `pwa-staff/src/admin.ts`, lines 116-125
- **Why**: Malicious users can register or update their names with HTML scripts (e.g., `<script>...</script>` or onload payloads), which will execute in the context of the admin user when viewing the dashboard.
- **Suggestion**: Create table rows and cells dynamically using `document.createElement`, `row.insertCell`, and populate them using `textContent`.

### [Critical] Finding 2: Cross-Site Scripting (XSS) in Leaderboard Rendering
- **What**: HTML injection via `playerName` inside `row.innerHTML`.
- **Where**: `pwa-staff/src/leaderboard.ts`, lines 37-41
- **Why**: Similar to the admin dashboard, players can update their usernames to contain XSS payloads. When the leaderboard renders their name, the payload executes.
- **Suggestion**: Use `row.insertCell()` and assign the player's name via `.textContent` instead of setting `row.innerHTML`.

### [Critical] Finding 3: RLS Policies Configured for Non-Existent Tables
- **What**: Row Level Security (RLS) is enabled and configured for tables `profiles` and `scores`, which do not exist in the database schema.
- **Where**: `pwa-staff/supabase/migrations/20240426000000_rls_policies.sql`, lines 1-18
- **Why**: The schema uses `players`, `leaderboard`, and `game_sessions` tables. Therefore, the actual tables holding player, score, and session details are completely unprotected by RLS.
- **Suggestion**: Update RLS migration to apply rules to `players` (allow select by anyone, update by authenticated self), `leaderboard` (allow select by anyone), and `game_sessions` (allow select by anyone/admin, update/insert by authenticated self or service_role).

### [Major] Finding 4: Unhandled Promise Rejections in Click Handlers
- **What**: Missing `try...catch` blocks around `getPlayer()` in `handleViewClick` and `handleEditClick`.
- **Where**: `pwa-staff/src/admin.ts`, lines 131-167
- **Why**: If a database error or network timeout occurs while fetching a single player, the click handler fails silently with an unhandled promise rejection in the browser console. The UI hangs without letting the user know something failed.
- **Suggestion**: Wrap the body of the click handlers in a `try...catch` and present a user-friendly alert or error indicator.

### [Major] Finding 5: Broken ESLint Configuration for ESM/TypeScript
- **What**: ESLint configuration specifies `"sourceType": "script"` instead of `"sourceType": "module"`.
- **Where**: `pwa-staff/.eslintrc.json`, line 14
- **Why**: Running `npm run lint` fails with parsing errors: `Parsing error: 'import' and 'export' may appear only with 'sourceType: module'` for all TS files in `src/`.
- **Suggestion**: Update `.eslintrc.json` to configure `"sourceType": "module"`.

### [Minor] Finding 6: Missing Pagination & Scale Risk
- **What**: `admin.ts` fetches all players and all game sessions in one batch.
- **Where**: `pwa-staff/src/admin.ts`, lines 19-29, 87-97
- **Why**: As the user base grows, fetching thousands of players/sessions simultaneously will lead to performance degradation, high memory usage, and possible query timeout errors.
- **Suggestion**: Implement offset/limit pagination or search filter inputs.

---

## 3. Verified Claims
- **Claim**: The build works and compiles without errors -> verified via `npm run build` and `npx tsc --noEmit` -> PASS (0 warnings, 0 type safety errors)
- **Claim**: Unit tests run and pass -> verified via `npm run test` (Vitest suite) -> PASS (9/9 tests passed in `auth.test.ts` and `worker.test.ts`)
- **Claim**: Realtime subscription works for leaderboard -> verified via visual code trace in `leaderboard.ts:46-50` -> PASS (uses postgres_changes filter correctly)

---

## 4. Coverage Gaps
- **Playwright E2E Tests for Admin/Leaderboard** - risk level: Medium - recommendation: Add E2E tests in `tests-e2e` verifying the leaderboard renders scores and the admin panel loads players/sessions. Currently, only auth is covered.

---

## 5. Adversarial Challenge Report

### [Critical] Challenge 1: HTML/JS Injection on Leaderboard and Admin Dashboard
- **Assumption Challenged**: User-provided usernames, names, and emails are clean/safe to render directly as HTML.
- **Attack Scenario**: A malicious user registers with a name like `<img src=x onerror="alert(document.cookie)">`.
- **Blast Radius**: Administrator session takeover. When the admin opens `admin.html` to review players, the injected script runs in their context, potentially exfiltrating sensitive data (e.g. Supabase tokens/cookies).
- **Mitigation**: Escape all user strings before rendering, or preferably use DOM APIs (`textContent`, `innerText`, `insertCell`) instead of `.innerHTML`.

### [Critical] Challenge 2: Total Absence of Access Control on Admin Panel
- **Assumption Challenged**: Only staff/admins will access `admin.html`.
- **Attack Scenario**: Any non-authenticated user or low-privilege player accesses `/admin.html` directly in the browser.
- **Blast Radius**: Because RLS is not enabled on `players` and `game_sessions` tables, the anonymous client can successfully fetch all player emails, names, and edit them. This represents a complete breach of user privacy and data integrity.
- **Mitigation**: Enable RLS policies on `players` and `game_sessions`. Require authentication and verify user role (e.g. via app_metadata role or a staff/admin table check) before rendering the page content or serving the data.

---

## 6. 5-Component Handoff Details

### 1. Observation
- `pwa-staff/src/admin.ts` lines 116-125:
```ts
            ${players.map(player => `
                <tr>
                    <td>${player.name ?? ''}</td>
                    <td>${player.email ?? ''}</td>
                    <td>
                        <button class="view-button" data-id="${player.id}">View</button>
                        <button class="edit-button" data-id="${player.id}">Edit</button>
                    </td>
                </tr>
            `).join('')}
```
- `pwa-staff/src/leaderboard.ts` lines 37-41:
```ts
            row.innerHTML = `
                <td>${index + 1}</td>
                <td>${playerName}</td>
                <td>${score.score}</td>
            `;
```
- `pwa-staff/supabase/migrations/20240426000000_rls_policies.sql` lines 2, 11:
```sql
ALTER TABLE profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE scores ENABLE ROW LEVEL SECURITY;
```
- `pwa-staff/.eslintrc.json` line 14:
```json
    "sourceType": "script"
```
- Command `npx eslint src/admin.ts` output:
```
/Users/sac/rocket-craft/pwa-staff/src/admin.ts
  1:1  error  Parsing error: 'import' and 'export' may appear only with 'sourceType: module'
```

### 2. Logic Chain
1. Directly injecting user-controlled data (`player.name`, `player.email`, `score.players.username`) into elements via `.innerHTML` allows browser interpretation of HTML tags.
2. Injected HTML tags including `<script>` or event handlers (like `onload`, `onerror`) will execute in the context of the user viewing the page.
3. Therefore, XSS vulnerabilities exist on both pages.
4. The migration file `20240426000000_rls_policies.sql` alters table names `profiles` and `scores` which do not exist in the database schema.
5. Therefore, the actual tables `players`, `leaderboard`, and `game_sessions` are left unprotected.
6. The ESLint config defaults to `"sourceType": "script"`, which throws parsing errors on ES Module imports and exports.

### 3. Caveats
No live Supabase database instance was accessed during this review. The RLS policy issue was determined through database migration files analysis.

### 4. Conclusion
The codebase is functionally complete and successfully passes build (`npm run build`) and test suites (`npm run test`). However, due to critical security risks (XSS vulnerabilities, missing database RLS protections) and a broken linting configuration, the final verdict is `REQUEST_CHANGES`.

### 5. Verification Method
- To verify the ESLint issue, run: `npx eslint src/admin.ts` or `npm run lint`.
- To verify type-safety, run: `npx tsc --noEmit`.
- To verify the unit tests, run: `npm run test`.
- To verify database schema names, inspect files under `/Users/sac/rocket-craft/supabase/migrations/`.
