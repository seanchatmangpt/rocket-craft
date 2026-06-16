## 2026-06-15T21:53:22Z
You are a Worker. Your task is to update the database migration file at `/Users/sac/rocket-craft/supabase/migrations/20240401000003_sync_auth_users_to_players.sql` to fix a trimming bypass bug.

Specifically:
- In `public.handle_new_user()`, replace the PostgreSQL native `TRIM()` calls with `trim(both E' \\t\\r\\n' from ...)` to ensure that tabs, newlines, and carriage returns are also trimmed along with spaces.
- The two lines to change are:
  1. `proposed_name := TRIM(proposed_name);` -> `proposed_name := trim(both E' \\t\\r\\n' from proposed_name);`
  2. `base_username := TRIM(base_username);` -> `base_username := trim(both E' \\t\\r\\n' from base_username);`

After making the change:
1. Run `supabase db reset` and `supabase db lint` to verify syntax and execution.
2. Run the Python verification script `/Users/sac/rocket-craft/.agents/challenger_db_schema_1/verify_sync_auth_users.py` to confirm that all 7 test paths (including the Trimming Path with tabs and newlines) now pass successfully.
3. Write a handoff report documenting the changes and test output to `/Users/sac/rocket-craft/.agents/worker_db_schema_3/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.
