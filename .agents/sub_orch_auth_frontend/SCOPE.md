# Scope: Milestone 2 - Auth & Frontend Setup

## Architecture
- **Supabase Client Layer (`pwa-staff/src/lib/supabaseClient.ts`)**: Initialized with local Supabase configuration. It serves as the primary gateway for all client-side authentication requests.
- **Authentication Services (`pwa-staff/src/auth.ts`, `pwa-staff/src/login.ts`, `pwa-staff/src/signup.ts`, `pwa-staff/src/profile.ts`)**: Implementation of login, registration, logout, session retrieval, and redirection logic.
- **User Interface (`login.html`, `signup.html`, `profile.html`)**: HTML pages serving the frontend, loading assets, and displaying UI components based on user session status.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Supabase Client Config | Configure client URL and anon key | None | DONE |
| 2 | Auth Service Integration | Implement login, signup, logout functions | M1 | DONE |
| 3 | HTML Path Corrections | Fix relative asset paths in html files | None | DONE |
| 4 | Profile Session Bug Fix | Await `getSession()`/`getUser()` in profile.ts | M2 | DONE |
| 5 | Compile & Verify | Run frontend builds and verify compilation | M1, M2, M3, M4 | DONE |

## Interface Contracts
### Supabase Auth Functions
- `signUp(email, password)`: Registers a user with email and password via Supabase.
- `signIn(email, password)`: Logs a user in.
- `signOut()`: Logs a user out and redirects to `login.html`.
- `getSession()` / `getUser()`: Retrieve the current authenticated user session asynchronously and update the profile UI or redirect to `login.html` if unauthenticated.
