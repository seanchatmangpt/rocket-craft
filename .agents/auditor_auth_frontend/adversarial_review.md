## Challenge Summary

**Overall risk assessment**: CRITICAL

## Challenges

### [Critical] Challenge 1: Browser Runtime Crash due to undefined `process.env`

- **Assumption challenged**: That Node-style environment variables (`process.env.SUPABASE_URL` and `process.env.SUPABASE_ANON_KEY`) are natively supported or automatically replaced by the bundler when running in a browser environment.
- **Attack scenario**: When a user loads `login.html`, `signup.html`, or `profile.html` in the browser, the bundled script is executed. The evaluation of `process.env` immediately throws `Uncaught ReferenceError: process is not defined`.
- **Blast radius**: This crash stops execution of the entire script immediately, preventing any event listeners (such as form submission or logout button handlers) from registering. The forms default to standard HTML submission (page reload with query params) rather than Supabase Client interaction. This results in the complete failure of the authentication flows and causes Playwright E2E tests to time out.
- **Mitigation**: Update the `build:ts` script in `pwa-staff/package.json` to pass environment variables to `esbuild` using `--define`, for example:
  ```json
  "build:ts": "esbuild src/*.ts --bundle --outdir=dist --define:process.env.SUPABASE_URL='\"http://127.0.0.1:54321\"' --define:process.env.SUPABASE_ANON_KEY='\"sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH\"' && ..."
  ```

### [Medium] Challenge 2: Redundant and Orphaned `auth.ts` File

- **Assumption challenged**: That the helper module `pwa-staff/src/auth.ts` is a core part of the authentication integration.
- **Attack scenario**: Developers attempting to maintain or refactor the auth system will assume `auth.ts` controls session management and auth listeners.
- **Blast radius**: In reality, `auth.ts` is an orphan file. It is never imported by `login.ts`, `signup.ts`, `profile.ts`, or any other module in the codebase. It has dead code that mimics auth management but is bypassed by direct Supabase client calls.
- **Mitigation**: Either delete `auth.ts` if it is not needed, or refactor `login.ts`, `signup.ts`, and `profile.ts` to import and utilize the functions/state exported by `auth.ts`.
