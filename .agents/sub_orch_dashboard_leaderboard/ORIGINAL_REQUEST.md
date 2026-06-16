# Original User Request

## Initial Request — 2026-06-15T15:09:20-07:00

You are a Sub-orchestrator for Milestone 3: Admin Dashboard & Leaderboard.
Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_dashboard_leaderboard/`.
Your parent is conversation ID `51eb4be3-e539-4e5f-87d9-4d687e04cd83` (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Your mission:
Implement the query and rendering updates in the Admin Dashboard and Leaderboard:
1. Update `pwa-staff/src/admin.ts` to fetch registered players (`id`, `name`, `email`) from the `public.players` table and correctly render them.
2. Update `pwa-staff/src/leaderboard.ts` to fetch high scores from the `leaderboard` table joined with the player's `username` from the `public.players` table, and render the player names on the leaderboard instead of blank/missing names.

Please perform the following steps:
1. Create your `BRIEFING.md` and `progress.md` files in your working directory `/Users/sac/rocket-craft/.agents/sub_orch_dashboard_leaderboard/`.
2. Create your `SCOPE.md` defining this milestone, the interfaces, and the changes to be made.
3. Run the iteration loop:
   - Spawn an Explorer to review `admin.ts`, `leaderboard.ts`, and project schemas, and design code updates.
   - Spawn a Worker to implement changes. Include the MANDATORY INTEGRITY WARNING in the worker's prompt. Make sure the worker runs frontend builds (`npm run build` or similar) to verify compilation.
   - Spawn a Reviewer to verify queries and rendering.
   - Spawn a Challenger and Forensic Auditor to test the logic and verify integrity.
4. When the gate passes, write `handoff.md` and send a message reporting status back to parent (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Never write or edit code files directly; always delegate to workers. You may write to metadata files inside your folder `/Users/sac/rocket-craft/.agents/sub_orch_dashboard_leaderboard/`.
