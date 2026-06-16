## 2026-06-15T22:01:06Z
The user requested the following frontend Supabase Auth integration changes:
1. Configure `pwa-staff/src/lib/supabaseClient.ts` to use local Supabase URL and anon key.
2. Integrate actual Supabase client auth in `pwa-staff/src/auth.ts`.
3. Integrate actual Supabase client auth in `pwa-staff/src/login.ts` (redirect to `profile.html` on success).
4. Integrate actual Supabase client auth in `pwa-staff/src/signup.ts` (redirect to `profile.html` on success).
5. Correct the critical bug in `pwa-staff/src/profile.ts` (await `supabase.auth.getUser()` asynchronously, redirect to `login.html` if user not found, and log out with redirect to `login.html` on logout click).
6. Fix relative asset paths in `pwa-staff/login.html`, `pwa-staff/signup.html`, and `pwa-staff/profile.html` to reference `dist/` or `./dist/` instead of `../dist/`.
7. Verify by running `npm run build` and `npm run test` in `pwa-staff/` folder.
