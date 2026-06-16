## 2026-06-15T22:07:02Z
You are a teamwork_preview_worker.
Your task is to fix a critical runtime bug in the frontend PWA where using `process.env` literally throws a browser `ReferenceError` (since `process` is not defined in the browser window context).

Please apply the following changes:
1. In `pwa-staff/src/lib/supabaseClient.ts`, replace the process.env lookup with a browser-safe typeof check:
```typescript
import { createClient } from '@supabase/supabase-js'

const supabaseUrl = (typeof process !== 'undefined' && process.env?.SUPABASE_URL) || 'http://127.0.0.1:54321'
const supabaseAnonKey = (typeof process !== 'undefined' && process.env?.SUPABASE_ANON_KEY) || 'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH'

export const supabase = createClient(supabaseUrl, supabaseAnonKey)
```

2. Run `npm run build` from the `pwa-staff/` folder to rebuild the assets and ensure that the bundled JavaScript files under `dist/` no longer reference raw `process.env` directly in a way that causes runtime errors.
3. Run `npm run test` from the `pwa-staff/` folder to verify that unit tests continue to pass.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Your working directory is `/Users/sac/rocket-craft/.agents/worker_auth_frontend_fix/`. Please write your metadata only there. Do NOT write metadata to the project source directory.
When finished, write a detailed report of the changes and build/test output, then send a message back to the parent.
