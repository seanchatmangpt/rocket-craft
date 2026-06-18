# pwa-staff — CLAUDE.md

## Purpose

TypeScript Progressive Web App serving as the staff/player-facing front-end for
Rocket Craft. Provides authentication, leaderboards, player HUD, profile management,
and admin tooling — all backed by Supabase. Also hosts the pre-built HTML5/WebAssembly
UE4 game builds (ShooterGame, SurvivalGame, Brm, RealisticRendering, FullSpectrum)
for in-browser play. Includes a service worker for offline support and PWA installation.

## Directory Structure

```
pwa-staff/
├── package.json             # npm scripts, deps (esbuild, vitest, Playwright)
├── tsconfig.json            # TypeScript compiler config
├── vitest.config.ts         # Unit test config
├── playwright.config.ts     # E2E test config
├── postcss.config.js        # PostCSS (autoprefixer)
├── manifest.json            # PWA web app manifest
├── favicon.ico
│
├── src/                     # TypeScript source (compiled → dist/)
│   ├── auth.ts              # Supabase sign-in/sign-up/sign-out flows
│   ├── login.ts             # Login page logic
│   ├── signup.ts            # Signup page logic
│   ├── admin.ts             # Admin panel: user management, audit logs
│   ├── hud.ts               # In-game HUD overlay (health, score, inventory)
│   ├── leaderboard.ts       # Leaderboard fetch + render
│   ├── profile.ts           # Player profile page
│   └── lib/
│       └── supabaseClient.ts # Supabase client singleton (reads SUPABASE_URL, SUPABASE_ANON_KEY)
│
├── worker.ts                # Service worker (compiled → worker.js)
├── cache.ts                 # Cache strategy helpers (compiled → cache.js)
│
├── css/
│   └── style.css            # Source CSS (compiled → dist/style.css via PostCSS)
│
├── dist/                    # Build output (gitignored in normal repos; committed here)
│   ├── admin.js  auth.js  hud.js  leaderboard.js
│   ├── login.js  profile.js  signup.js
│   └── style.css
│
├── js/                      # Legacy / manually-maintained JS
│   ├── admin.js
│   └── auth.js
│
├── *.html                   # One HTML file per page
│   ├── index.html           # Landing page
│   ├── login.html
│   ├── signup.html
│   ├── profile.html
│   ├── leaderboard.html
│   ├── admin.html
│   └── offline.html         # Service worker offline fallback
│
├── tests-e2e/               # Playwright end-to-end tests
│   ├── auth.spec.ts
│   ├── hud.spec.ts
│   └── example.spec.ts
│
├── *.test.ts                # Vitest unit tests (co-located at root)
│   ├── auth.test.ts
│   ├── admin-leaderboard.test.ts
│   ├── hud.test.ts
│   └── worker.test.ts
│
└── *-HTML5-Shipping.*       # Pre-built UE4 HTML5 game bundles (binary, do not edit)
    ├── Brm-HTML5-Shipping.{html,js,data,wasm}
    ├── FullSpectrum-HTML5-Shipping.{html,js,data,wasm}
    ├── RealisticRendering-HTML5-Shipping.{html,js,data,wasm}
    ├── ShooterGame-HTML5-Shipping.{html,js,data,wasm}
    └── SurvivalGame-HTML5-Shipping.{html,js,data,wasm}
```

## Key Commands

```bash
# Install dependencies
npm install

# Build everything (CSS + TypeScript + service worker)
npm run build

# Build only TypeScript files
npm run build:ts

# Build only CSS
npm run build:css

# Start local dev server (port 3000)
npm start

# Run unit tests (vitest)
npm test

# Run E2E tests (Playwright — requires server running)
npx playwright test

# Run E2E tests with UI
npx playwright test --ui

# Lint
npm run lint

# Format
npm run format

# Type-check without emitting
npx tsc --noEmit
```

## Build Pipeline

1. `build:css` — PostCSS processes `css/style.css` → `dist/style.css` (autoprefixer)
2. `build:ts` — esbuild bundles each `src/*.ts` → `dist/*.js`; also compiles
   `worker.ts` → `worker.js` and `cache.ts` → `cache.js`

esbuild is the only bundler — no webpack, no vite. Each entry point is bundled
independently. `--bundle` flag means all imports are inlined.

## Environment Variables

| Variable            | Required | Where Used                  | Description            |
| ------------------- | -------- | --------------------------- | ---------------------- |
| `SUPABASE_URL`      | Yes      | `src/lib/supabaseClient.ts` | Supabase project URL   |
| `SUPABASE_ANON_KEY` | Yes      | `src/lib/supabaseClient.ts` | Supabase anonymous key |

For local dev, set these in `.env` and reference them in `supabaseClient.ts`. They
are not bundled by esbuild's `define` — pass them at runtime or via a build step.

## Pages and Their Entry Points

| HTML Page          | TypeScript Entry     | Supabase Tables Used     |
| ------------------ | -------------------- | ------------------------ |
| `index.html`       | (none / inline)      | —                        |
| `login.html`       | `src/login.ts`       | `auth.users`             |
| `signup.html`      | `src/signup.ts`      | `auth.users`, `profiles` |
| `profile.html`     | `src/profile.ts`     | `profiles`               |
| `leaderboard.html` | `src/leaderboard.ts` | `scores`, `profiles`     |
| `admin.html`       | `src/admin.ts`       | `profiles`, `audit_log`  |

## Service Worker (`worker.ts`)

Handles:

- Cache-first strategy for static assets (CSS, JS, HTML)
- Network-first strategy for Supabase API calls
- Offline fallback to `offline.html` when network unavailable
- Cache versioning — update `CACHE_VERSION` in `worker.ts` to bust the cache

After changes to `worker.ts`, run `npm run build` and hard-reload the browser
(Ctrl+Shift+R) to activate the new worker.

## Testing

### Unit Tests (vitest)

```bash
npm test                     # Run all unit tests
npx vitest run src/auth.ts   # Run tests for one module
npx vitest --coverage        # Coverage report
```

Test files: `*.test.ts` at the package root. They mock Supabase via vitest `vi.mock`.

### E2E Tests (Playwright)

```bash
# Must have server running first
npm start &
npx playwright test
```

Tests in `tests-e2e/`:

- `auth.spec.ts` — login, logout, signup flows
- `hud.spec.ts` — HUD rendering with mock game state
- `example.spec.ts` — smoke test

Playwright config in `playwright.config.ts` targets `localhost:3000`.

## UE4 HTML5 Game Bundles

The `*-HTML5-Shipping.*` files are compiled UE4 games. Each game is a set of 4 files:

| Extension | Purpose                                      |
| --------- | -------------------------------------------- |
| `.html`   | Loader page (links JS/WASM/data)             |
| `.js`     | Emscripten JavaScript runtime                |
| `.wasm`   | Compiled game binary                         |
| `.data`   | Packaged game content (textures, maps, etc.) |

**Do not edit these files.** Regenerate them via UE4's HTML5 packaging pipeline
(`RunUAT.sh BuildCookRun -platform=HTML5`). They can be large (100 MB+); ensure
`.gitattributes` marks them for Git LFS if this repo uses it.

## Relation to the Monorepo

- **`tools/rocket-sdk`** (`pwa.rs`) — the `rocket-cmd pwa` subcommand drives
  `npm run build` in this directory.
- **`unify-rs/unify-bp`** (`pwa_export.rs`) — generates Blueprint metadata JSON
  consumed by `hud.ts` to display in-game Blueprint info.
- **`versions/`** — the HTML5 game bundles are produced from UE4 projects in
  `versions/` via the HTML5 shipping package step.

## Caveats and Gotchas

- **No framework**: this is vanilla TypeScript + esbuild. There is no React, Vue, or
  Angular. DOM manipulation is done directly.
- **esbuild does not tree-shake CommonJS**: if you add a CommonJS library, the full
  module is bundled. Prefer ESM-only packages.
- **Supabase client is a singleton**: `supabaseClient.ts` exports a single instance.
  Do not create additional Supabase clients in other modules — reuse the import.
- **dist/ is committed**: unlike most projects, `dist/` appears to be committed.
  Always run `npm run build` before committing TypeScript changes.
- **Service worker scope**: the worker registers at `/` — all pages are in scope.
  If you add new routes, verify the cache strategy handles them correctly.
- **HTML5 `.data` files are huge**: `SurvivalGame-HTML5-Shipping.data` can be
  gigabytes. Do not re-add them to git history if removed — use Git LFS.
- **`lucide.min.js`** is vendored locally — do not `npm install lucide`; use the
  vendored file to avoid version drift.
