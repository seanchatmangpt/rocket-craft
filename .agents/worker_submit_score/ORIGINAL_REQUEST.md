## 2026-06-15T22:41:01Z

You are a teamwork_preview_worker.
Your role: Worker for Milestone 4: Edge Function Submit Score.
Your working directory is `/Users/sac/rocket-craft/.agents/worker_submit_score/`.
Your parent is conversation ID ed8d8902-d2f5-42cf-b523-51bb5e89696b.

Objective:
Implement the score submission Deno edge function in `/Users/sac/rocket-craft/supabase/functions/submit-score/index.ts`.

Requirements:
1. Implement the Deno edge function to process POST requests containing `{ score: number }`.
2. Extract the authorization JWT token from the headers (`Authorization: Bearer <token>`). Check headers case-insensitively (e.g. `req.headers.get("Authorization") || req.headers.get("authorization")`).
3. Construct a Supabase client inside the edge function (using `Deno.env.get("SUPABASE_URL")` and `Deno.env.get("SUPABASE_ANON_KEY")` with the Authorization header token) to verify the user's identity and obtain their UUID via `supabaseClient.auth.getUser()`.
4. Validate that the score is a valid, non-NaN integer between 0 and 1000 inclusive. Return status 400 (Bad Request) if the score is out of bounds or invalid.
5. Save the score to the `public.game_sessions` table (`player_id` matches the user UUID, `score` matches the submitted score).
6. Query the `public.leaderboard` table for the player's current high score. If no high score exists or the new score is higher than the existing maximum high score, update/insert the player's entry in the `public.leaderboard` table.
7. Return a success response: `{ "message": "Score of <score> submitted successfully!", "score": <score> }` with status 200.
8. Support CORS preflight requests (OPTIONS method) and return standard CORS headers:
   `Access-Control-Allow-Origin: *`
   `Access-Control-Allow-Headers: authorization, x-client-info, apikey, content-type`
9. Make sure code compiles cleanly under `deno check` and lints cleanly under `deno lint --rules-exclude=no-import-prefix`.
10. Ensure error handling is type-safe under Deno 2 TypeScript (e.g., cast caught error object properties appropriately or use `error instanceof Error ? error.message : String(error)`). Include `// deno-lint-ignore-file no-import-prefix` at the top of the file.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.

Verification commands:
- Run `deno check /Users/sac/rocket-craft/supabase/functions/submit-score/index.ts` to verify compilation.
- Run `deno lint --rules-exclude=no-import-prefix /Users/sac/rocket-craft/supabase/functions/submit-score/index.ts` to verify linting.

Handoff:
Write your implementation summary and the command execution outputs to `/Users/sac/rocket-craft/.agents/worker_submit_score/handoff.md`, and reply with a message when done.
