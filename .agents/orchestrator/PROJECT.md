# Project: Rocket Craft PWA Integration

## Architecture
- **Frontend**: Vanilla TypeScript/HTML progressive web app (PWA) served on port 3000.
- **Backend/DB**: Local Supabase instance (PostgreSQL database, GoTrue Auth, Deno Edge Functions).
- **Communication**: Supabase JS Client (`@supabase/supabase-js`) on the frontend, Deno edge functions on the backend.

## Milestones
| # | Name | Scope | Dependencies | Status | Conv ID |
|---|------|-------|-------------|--------|---------|
| 1 | DB Schema & Trigger | Add columns to `players`, create postgres trigger to sync `auth.users` to `public.players` | None | DONE | 3a6147ec-4c41-42b0-8013-c0f248348234 |
| 2 | Auth & Frontend Setup | Config supabaseClient, actual auth flow, fix HTML asset paths | M1 | DONE | 7acf1108-b1f0-483b-a28a-06538b60f5c6 |
| 3 | Admin Dashboard & Leaderboard | Query `players` for admin, join query for leaderboard | M1, M2 | DONE | 75a28482-a733-41c6-a29e-137b1c05a6b3 |
| 4 | Edge Function Submit Score | Implement `submit-score` edge function to write to DB and leaderboard | M1 | DONE | ed8d8902-d2f5-42cf-b523-51bb5e89696b |
| 5 | E2E Testing & Verification | Configure port 3000, run Playwright E2E tests, verify all flows | M1, M2, M3, M4 | DONE | 24a37630-5370-426a-95af-f89bda39a1ef |
| 6 | Production Release Gaps | Playwright configuration chromium only, fix example.spec.ts regex, verify vitest and playwright E2E | M5 | DONE | 62170365-3e1f-4235-87b7-1cad9be5968a |




## Interface Contracts
### Frontend ↔ Supabase Auth
- Sign-up: `supabase.auth.signUp({ email, password })` -> triggers `public.handle_new_user()`
- Login: `supabase.auth.signInWithPassword({ email, password })`
- Logout: `supabase.auth.signOut()`
- Session: `supabase.auth.getSession()` and `supabase.auth.onAuthStateChange()`

### Frontend ↔ Supabase Database
- Admin dashboard queries `players` table: `select('id, name, email')`
- Leaderboard queries `leaderboard` table joined with `players` username: `select('score, rank, players(username)')`

### Edge Function ↔ Database
- Request: `POST /submit-score` with header `Authorization: Bearer <user_token>` and body `{ score: number }`
- Logic: Verify `score` is between 0 and 1000. Insert into `game_sessions` and upsert into `leaderboard`.

## Code Layout
- `supabase/migrations/` - PostgreSQL migration scripts
- `supabase/functions/submit-score/index.ts` - Score submission Deno edge function
- `pwa-staff/src/` - Frontend TypeScript sources
- `pwa-staff/*.html` - Frontend HTML templates
- `pwa-staff/package.json` - Frontend configuration and scripts
- `pwa-staff/playwright.config.ts` - Playwright E2E tests configuration
- `pwa-staff/tests-e2e/` - Playwright E2E test suites
