# Original User Request

## Initial Request — 2026-06-15T14:38:12-07:00

You are a Sub-orchestrator for Milestone 1: DB Schema & Trigger.
Your working directory is `/Users/sac/rocket-craft/.agents/sub_orch_db_schema/`.
Your parent is conversation ID `51eb4be3-e539-4e5f-87d9-4d687e04cd83` (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Your mission:
Implement the database migrations to:
1. Update `public.players` to support `email` (VARCHAR(255)) and `name` (VARCHAR(255)) columns.
2. Implement a PostgreSQL trigger function that automatically syncs a newly created user in `auth.users` to `public.players` upon registration (extracting email and setting the username/name using the email prefix).
3. The migrations must be added to `supabase/migrations/` in a clean, non-conflicting way.

Please perform the following steps:
1. Create your `BRIEFING.md` and `progress.md` files in your working directory `/Users/sac/rocket-craft/.agents/sub_orch_db_schema/`.
2. Create your `SCOPE.md` defining this milestone, the interfaces, and the changes to be made.
3. Run the iteration loop:
   - Spawn an Explorer to design the migration sql and check existing migrations in `supabase/migrations/`.
   - Spawn a Worker to create/update the migration files in `supabase/migrations/`. Include the MANDATORY INTEGRITY WARNING in the worker's prompt.
   - Spawn a Reviewer to verify the sql logic and schema correctness.
   - Spawn a Challenger and Forensic Auditor to verify integration and check for integrity.
4. When the gate passes, write `handoff.md` and send a message reporting status back to parent (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Never write or edit code files directly; always delegate to workers. You may write to metadata files inside your folder `/Users/sac/rocket-craft/.agents/sub_orch_db_schema/`.
