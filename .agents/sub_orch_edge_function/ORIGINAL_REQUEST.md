# Original User Request

## Initial Request — 2026-06-15T15:38:03-07:00

You are a Sub-orchestrator for Milestone 4: Edge Function Submit Score.
Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_edge_function/`.
Your parent is conversation ID `51eb4be3-e539-4e5f-87d9-4d687e04cd83` (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Your mission:
Implement the score submission Deno edge function:
1. Implement the Deno edge function in `supabase/functions/submit-score/index.ts` to process POST requests containing `{ score: number }`.
2. Extract the authorization JWT token from the headers (`Authorization: Bearer <token>`). Use a Supabase client inside the edge function (using `Deno.env.get("SUPABASE_URL")` and `Deno.env.get("SUPABASE_ANON_KEY")` or the Authorization header token) to verify the user's identity and obtain their UUID.
3. Validate that the score is a valid number between 0 and 1000 inclusive. Return an error response (e.g., status 400) if the score is out of bounds or invalid.
4. Save the score to the `public.game_sessions` table (`player_id` matches the user UUID, `score` matches the submitted score).
5. Query the `public.leaderboard` table for the player's current high score. If no high score exists or the new score is higher than the existing one, update or upsert the player's entry in the `public.leaderboard` table.

Please perform the following steps:
1. Create your `BRIEFING.md` and `progress.md` files in your working directory `/Users/sac/rocket-craft/.agents/sub_orch_edge_function/`.
2. Create your `SCOPE.md` defining this milestone, the interfaces, and the changes to be made.
3. Run the iteration loop:
   - Spawn an Explorer to review the `submit-score` Deno code skeleton, local environment, and design code updates.
   - Spawn a Worker to implement changes. Include the MANDATORY INTEGRITY WARNING in the worker's prompt. Make sure the worker runs Deno compilation / lint check or tests if available to verify compilation.
   - Spawn a Reviewer to verify the authentication verification, score validation logic, and database insertion/upsert SQL safety.
   - Spawn a Challenger and Forensic Auditor to test the logic (e.g. testing score bounds, valid/invalid tokens) and verify integrity.
4. When the gate passes, write `handoff.md` and send a message reporting status back to parent (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Never write or edit code files directly; always delegate to workers. You may write to metadata files inside your folder `/Users/sac/rocket-craft/.agents/sub_orch_edge_function/`.
