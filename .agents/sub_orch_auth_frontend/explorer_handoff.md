# Explorer Handoff Report: Frontend Supabase Auth Integration & Asset Path Corrections

## 1. Observation
We have inspected the following files in the `/Users/sac/rocket-craft/pwa-staff` directory:
- `pwa-staff/src/lib/supabaseClient.ts`
- `pwa-staff/src/auth.ts`
- `pwa-staff/src/login.ts`
- `pwa-staff/src/signup.ts`
- `pwa-staff/src/profile.ts`
- `pwa-staff/login.html`
- `pwa-staff/signup.html`
- `pwa-staff/profile.html`
- `pwa-staff/package.json`

### Current Contents of Files

#### `pwa-staff/src/lib/supabaseClient.ts`
```typescript
import { createClient } from '@supabase/supabase-js'

const supabaseUrl = process.env.SUPABASE_URL || 'YOUR_SUPABASE_URL'
const supabaseAnonKey = process.env.SUPABASE_ANON_KEY || 'YOUR_SUPABASE_ANON_KEY'

export const supabase = createClient(supabaseUrl, supabaseAnonKey)
```

#### `pwa-staff/src/auth.ts`
```typescript
/**
 * @fileoverview A simple authentication store for the PWA.
 * Manages user session state and provides it to the rest of the application.
 */

interface User {
  name: string;
  email: string;
}

interface Session {
  user: User;
  token: string;
}

let currentSession: Session | null = null;

const SESSION_STORAGE_KEY = 'rocket-craft-session';

/**
 * Dispatches an event to notify about authentication changes.
 */
function dispatchAuthChange() {
  window.dispatchEvent(new CustomEvent('auth-change', { detail: { session: currentSession } }));
}

/**
 * Initializes the auth store by loading the session from localStorage.
 */
function initializeAuth() {
  const storedSession = localStorage.getItem(SESSION_STORAGE_KEY);
  if (storedSession) {
    try {
      currentSession = JSON.parse(storedSession);
      dispatchAuthChange();
    } catch (error) {
      console.error('Failed to parse session from localStorage', error);
      localStorage.removeItem(SESSION_STORAGE_KEY);
    }
  }
}

/**
 * Returns the current user session.
 * @returns The current session or null if not authenticated.
 */
export function getSession(): Session | null {
  return currentSession;
}

/**
 * Simulates a login by setting the user session.
 * @param user The user object.
 * @param token A JWT or similar session token.
 */
export function login(user: User, token: string) {
  currentSession = { user, token };
  try {
    localStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(currentSession));
  } catch (error) {
    console.error('Failed to save session to localStorage', error);
  }
  dispatchAuthChange();
}

/**
 * Logs the user out by clearing the session.
 */
export function logout() {
  currentSession = null;
  try {
    localStorage.removeItem(SESSION_STORAGE_KEY);
  } catch (error) {
    console.error('Failed to remove session from localStorage', error);
  }
  dispatchAuthChange();
}

/**
 * A simple "hook" to subscribe to authentication changes.
 * @param callback The function to call when the auth state changes.
 * @returns A function to unsubscribe from the changes.
 */
export function useAuth(callback: (session: Session | null) => void): () => void {
  const handler = (event: Event) => {
    callback((event as CustomEvent).detail.session);
  };

  window.addEventListener('auth-change', handler);

  // Immediately call back with the current state
  callback(currentSession);

  return () => {
    window.removeEventListener('auth-change', handler);
  };
}

// Initialize on load
initializeAuth();
```

#### `pwa-staff/src/login.ts`
```typescript
import { supabase } from './lib/supabaseClient'

const loginForm = document.getElementById('login-form')

loginForm!.addEventListener('submit', async (event) => {
  event.preventDefault()

  const email = (document.getElementById('email') as HTMLInputElement).value
  const password = (document.getElementById('password') as HTMLInputElement).value

  const { error } = await supabase.auth.signInWithPassword({
    email,
    password,
  })

  if (error) {
    alert(error.message)
  } else {
    window.location.href = 'profile.html'
  }
})
```

#### `pwa-staff/src/signup.ts`
```typescript
import { supabase } from './lib/supabaseClient'

const signupForm = document.getElementById('signup-form')

signupForm!.addEventListener('submit', async (event) => {
  event.preventDefault()

  const email = (document.getElementById('email') as HTMLInputElement).value
  const password = (document.getElementById('password') as HTMLInputElement).value

  const { error } = await supabase.auth.signUp({
    email,
    password,
  })

  if (error) {
    alert(error.message)
  } else {
    window.location.href = 'profile.html'
  }
})
```

#### `pwa-staff/src/profile.ts`
```typescript
import { supabase } from './lib/supabaseClient'

const userEmailElement = document.getElementById('user-email')
const logoutButton = document.getElementById('logout-button')

const session = supabase.auth.getSession()

if (!session) {
  window.location.href = 'login.html'
} else {
  // The type casting is necessary because getSession does not return a user object
  // in the session, we need to make another call to get the user.
  supabase.auth.getUser().then(({ data: { user } }) => {
    if (user && userEmailElement) {
      userEmailElement.textContent = user.email || '';
    }
  })
}

logoutButton!.addEventListener('click', async () => {
  const { error } = await supabase.auth.signOut()

  if (error) {
    alert(error.message)
  } else {
    window.location.href = 'login.html'
  }
})
```

#### `pwa-staff/login.html` (Asset path lines)
- Line 7: `<link rel="stylesheet" href="../dist/style.css">`
- Line 25: `<script type="module" src="../dist/login.js"></script>`

#### `pwa-staff/signup.html` (Asset path lines)
- Line 7: `<link rel="stylesheet" href="../dist/style.css">`
- Line 25: `<script type="module" src="../dist/signup.js"></script>`

#### `pwa-staff/profile.html` (Asset path lines)
- Line 7: `<link rel="stylesheet" href="../dist/style.css">`
- Line 15: `<script type="module" src="../dist/profile.js"></script>`

### Build System and Setup
In `pwa-staff/package.json`, we observed:
- Build Command: `npm run build` which runs `npm run build:css && npm run build:ts`.
- CSS compilation: `postcss css/style.css -o dist/style.css`
- TS compilation/bundler: `esbuild src/*.ts --bundle --outdir=dist && esbuild worker.ts --outfile=worker.js && esbuild cache.ts --outfile=cache.js`
- Test runner: `npm run test` which executes `vitest run` on `worker.test.ts`.
- Dependencies: `@supabase/supabase-js` is installed.

Running `npm run build` and `npm run test` succeeded:
```bash
> pwa-staff@1.0.0 build
> npm run build:css && npm run build:ts

⚡ Done in 34ms

> vitest run
✓ worker.test.ts (3 tests) 5ms
```

---

## 2. Logic Chain
1. **Supabase Client Credentials Configuration (`supabaseClient.ts`)**:
   - The current values `'YOUR_SUPABASE_URL'` and `'YOUR_SUPABASE_ANON_KEY'` are placeholders.
   - They must be updated to the local Supabase environment:
     - URL: `http://127.0.0.1:54321`
     - Anon Key: `sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`
   - Since the application is running completely client-side in the browser and bundled using `esbuild` without environment injection, hardcoding these as fallback values in `supabaseClient.ts` ensures they are correctly packaged at build time.

2. **Integration of Supabase Auth Client Methods**:
   - `login.ts` currently implements `supabase.auth.signInWithPassword({ email, password })`. This is the correct method for log-in.
   - `signup.ts` currently implements `supabase.auth.signUp({ email, password })`. This is the correct method for user registration.
   - `profile.ts` currently implements `supabase.auth.signOut()` for logging out. This is the correct method for terminating a session.
   - `profile.ts` implements session and user queries using `getSession()` and `getUser()`.
   - `auth.ts` is a simulated custom state management store that uses local storage and does not interface with Supabase. Because the active pages (`login.html`, `signup.html`, `profile.html`) load `dist/login.js`, `dist/signup.js`, and `dist/profile.js` directly and do not import or execute `auth.ts`, `auth.ts` is unused and requires no changes.

3. **Analysis of the Profile Page Verification Bug (`profile.ts`)**:
   - `supabase.auth.getSession()` is an asynchronous function returning a Promise: `Promise<{ data: { session: Session | null }, error: AuthError | null }>`.
   - Line 6 executes: `const session = supabase.auth.getSession()`. This sets the variable `session` to a Promise object.
   - Line 8 checks: `if (!session) { ... }`. A Promise is always truthy, so `!session` is always `false`.
   - As a result, the code *always* enters the `else` block and attempts to fetch the user information with `supabase.auth.getUser()`.
   - If the user is unauthenticated, `supabase.auth.getUser()` resolves with `user` as `null` or an error, but the `else` block only updates `userEmailElement.textContent` if `user` exists and does not redirect. The page remains blank/partially rendered, failing to redirect unauthenticated users to `login.html`.
   - **Solution**: Wrap the initialization in an `async` function and `await` the asynchronous `supabase.auth.getUser()` check. `getUser()` is the secure way to check authorization status because it validates the session JWT on the client and contacts the Supabase API if necessary. If it returns an error or `user === null`, the page must redirect to `login.html` immediately.

4. **Asset Paths Corrections in HTML Files**:
   - When serving the PWA from the root directory (`pwa-staff/` served as root), relative paths starting with `../` reference directories outside the server's root.
   - To serve files correctly, all references to `../dist/...` must be updated to `dist/...` or `./dist/...`.
   - References in `login.html`, `signup.html`, and `profile.html` must be adjusted accordingly.

---

## 3. Caveats
- **Supabase Backend State**: We assume the local Supabase container/server is running and listening at `http://127.0.0.1:54321` with anon key `sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH`.
- **Custom Auth Helper**: `pwa-staff/src/auth.ts` is identified as unused legacy code. Our design proposal ignores `auth.ts` because the app uses direct Supabase Client APIs directly, which is the standard implementation strategy here.

---

## 4. Conclusion
To integrate Supabase Auth and fix the critical issues, the following code revisions are recommended.

### A. Update `pwa-staff/src/lib/supabaseClient.ts`
Configure the Supabase URL and anonymous key:
```typescript
import { createClient } from '@supabase/supabase-js'

const supabaseUrl = process.env.SUPABASE_URL || 'http://127.0.0.1:54321'
const supabaseAnonKey = process.env.SUPABASE_ANON_KEY || 'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH'

export const supabase = createClient(supabaseUrl, supabaseAnonKey)
```

### B. Fix Async Redirection Bug in `pwa-staff/src/profile.ts`
Implement asynchronous verification using `supabase.auth.getUser()` and redirect on failure:
```typescript
import { supabase } from './lib/supabaseClient'

const userEmailElement = document.getElementById('user-email')
const logoutButton = document.getElementById('logout-button')

async function initProfile() {
  const { data: { user }, error } = await supabase.auth.getUser()

  if (error || !user) {
    window.location.href = 'login.html'
    return
  }

  if (userEmailElement) {
    userEmailElement.textContent = user.email || ''
  }
}

initProfile().catch((err) => {
  console.error('Failed to initialize profile:', err)
  window.location.href = 'login.html'
})

if (logoutButton) {
  logoutButton.addEventListener('click', async () => {
    const { error } = await supabase.auth.signOut()

    if (error) {
      alert(error.message)
    } else {
      window.location.href = 'login.html'
    }
  })
}
```

### C. Change Asset Paths in HTML Files
Modify `login.html`, `signup.html`, and `profile.html` to load files relative to the server root:
- **`login.html`**:
  - Replace `../dist/style.css` with `dist/style.css`
  - Replace `../dist/login.js` with `dist/login.js`
- **`signup.html`**:
  - Replace `../dist/style.css` with `dist/style.css`
  - Replace `../dist/signup.js` with `dist/signup.js`
- **`profile.html`**:
  - Replace `../dist/style.css` with `dist/style.css`
  - Replace `../dist/profile.js` with `dist/profile.js`

Proposed replacement files containing these full implementations have been prepared at:
- `/Users/sac/rocket-craft/.agents/explorer_auth_frontend/proposed_supabaseClient.ts`
- `/Users/sac/rocket-craft/.agents/explorer_auth_frontend/proposed_profile.ts`
- `/Users/sac/rocket-craft/.agents/explorer_auth_frontend/proposed_login.html`
- `/Users/sac/rocket-craft/.agents/explorer_auth_frontend/proposed_signup.html`
- `/Users/sac/rocket-craft/.agents/explorer_auth_frontend/proposed_profile.html`

---

## 5. Verification Method
1. **Compilation Check**:
   - Run `npm run build` from the `pwa-staff` directory to ensure that TypeScript files compile cleanly with esbuild and style sheets build with postcss.
2. **Test Check**:
   - Run `npm run test` to execute vitest and check that service worker caching logic is unhampered.
3. **Redirection and Login verification**:
   - Start the server using `npm run start` (uses `local-web-server`) or another HTTP server at `pwa-staff`.
   - Access `http://localhost:8000/profile.html` while signed out. The browser must immediately redirect to `login.html`.
   - Sign up or log in via Supabase; upon successful authentication, verify that the browser redirects to `profile.html` and displays the correct user email.
