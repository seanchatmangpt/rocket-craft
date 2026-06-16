## 2026-06-15T22:23:20Z

Please make the following codebase modifications:
1. Fix Unhandled Promise Rejections in `pwa-staff/src/leaderboard.ts`:
   - Wrap the entire async select block inside `fetchScores` in a `try...catch` block. Log any errors to the console.
2. Fix Unhandled Promise Rejections in `pwa-staff/src/admin.ts`:
   - Wrap the update database query inside `handleEditFormSubmit(event)` in a `try...catch` block. Log any errors to the console and alert the user.
3. Fix Unhandled Promise Rejections in other auth helper scripts inside `pwa-staff/src/`:
   - In `login.ts`: Wrap `await supabase.auth.signInWithPassword(...)` in a `try...catch` block.
   - In `signup.ts`: Wrap `await supabase.auth.signUp(...)` in a `try...catch` block.
   - In `profile.ts`: Wrap `await supabase.auth.signOut(...)` in a `try...catch` block inside the logout button event listener.
   - In `auth.ts`: Add `.catch(...)` to both `supabase.auth.getSession()` and `supabase.auth.signOut()` promise chains.
4. Run verification commands in `pwa-staff/`:
   - Run `npm run build` to verify clean compilation.
   - Run `npm run lint` to verify eslint checks pass successfully.
   - Run `npm run test` to verify unit tests pass successfully.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your findings and test logs to `handoff.md` within your directory, and send a message when done (Recipient: 75a28482-a733-41c6-a29e-137b1c05a6b3).
