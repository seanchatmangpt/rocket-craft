# Original User Request

## Initial Request — 2026-06-15T22:00:00Z

Implement the frontend Supabase Auth integration and fix asset paths:
1. Configure `pwa-staff/src/lib/supabaseClient.ts` with local Supabase URL (`http://127.0.0.1:54321`) and anon key (`sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`).
2. Integrate actual Supabase client auth in `pwa-staff/src/auth.ts`, `pwa-staff/src/login.ts`, `pwa-staff/src/signup.ts`, and `pwa-staff/src/profile.ts`.
3. Fix relative asset paths in `login.html`, `signup.html`, and `profile.html` to reference `dist/` or `./dist/` instead of `../dist/` since they are served from root.
4. Correct the critical bug in `pwa-staff/src/profile.ts` where it processes `supabase.auth.getSession()` synchronously instead of awaiting it (or use `supabase.auth.getUser()`).
5. Ensure logout redirects to `login.html`, and login/signup redirects to `profile.html` successfully showing the logged-in user's email.

Please perform the following steps:
1. Create your `BRIEFING.md` and `progress.md` files in your working directory `/Users/sac/rocket-craft/.agents/sub_orch_auth_frontend/`.
2. Create your `SCOPE.md` defining this milestone, the interfaces, and the changes to be made.
3. Run the iteration loop:
   - Spawn an Explorer to review frontend files and design code changes.
   - Spawn a Worker to implement changes. Include the MANDATORY INTEGRITY WARNING in the worker's prompt. Make sure the worker runs frontend builds (`npm run build` or similar) to verify compilation.
   - Spawn a Reviewer to verify auth flows, redirects, and path correctness.
   - Spawn a Challenger and Forensic Auditor to test the flows and verify integrity.
4. When the gate passes, write `handoff.md` and send a message reporting status back to parent (Recipient: 51eb4be3-e539-4e5f-87d9-4d687e04cd83).

Never write or edit code files directly; always delegate to workers. You may write to metadata files inside your folder `/Users/sac/rocket-craft/.agents/sub_orch_auth_frontend/`.
