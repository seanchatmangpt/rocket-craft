# rocket-craft-test-utils

> `rocket-craft-test-utils` is the microframework that prevents every layer from inventing its own definition of "passed."

It gives the stack one testing grammar: **admitted → refused → residual → receipt → replay**.

## What it is NOT

It does not replace Vitest, Playwright, `@nuxt/test-utils`, Supabase CLI, or `cargo test`. It wraps them with Rocket-Craft-specific fixtures, assertions, and receipt helpers.

## Install

```bash
pnpm add -D rocket-craft-test-utils
```

Peer dependencies (all optional — only install what your surface needs):

```bash
pnpm add -D @playwright/test @nuxt/test-utils @supabase/supabase-js
```

## Modules

| Module | Purpose |
|--------|---------|
| `types` | Canonical types: `RocketStatus`, `AdmissionStatus`, `RocketReceipt`, `GameIntent`, `VisualDeltaResult`, `CommandReceipt`, `UE4BridgeEvent` |
| `assertions` | Pure throw-on-failure assertions: receipts, residuals, intents, visual delta, PWA, Supabase |
| `receipts` | Build, validate, and mutation-detect receipt chains |
| `residuals` | Create, publish, filter residuals and blockers |
| `visual-delta` | Pixel-buffer hashing and delta computation |
| `commands` | Shell command wrappers returning `CommandReceipt` |
| `ue4-bridge` | Bridge mock/spy, admitted-intent routing, raw-browser-event law enforcement |
| `playwright` | `waitForRocketShellReady`, `emitGameIntent`, `assertCanvasDeltaAfterIntent` |
| `nuxt` | `mountRocketSuspended`, `setupRocketNuxtE2E` (requires `@nuxt/test-utils`) |
| `supabase` | `createTestUser`, `insertGameIntent`, `expectLocalSupabaseReady` |
| `fixtures` | `loadFixture(name)` — loads JSON from `fixtures/` |
| `reports` | `createVerifierReport`, `writeMarkdownReport`, `writeJsonReport` |

## Test project separation

```
test/unit/        — environment: node, no Nuxt runtime
test/nuxt/        — environment: @nuxt/test-utils (happy-dom)
test/e2e/         — Playwright, separate from Nuxt runtime helpers
test/integration/ — cross-layer (Supabase + bridge + canvas)
```

**Law:** Nuxt runtime helpers (`mountSuspended`, `mockNuxtImport`) must not appear in e2e files. Playwright calls must not appear in Nuxt runtime files.

## Receipt grammar

```
observe → admit → refuse → receipt → replay → publish residuals
```

A receipt is not a receipt unless the chain validates. A passing test is not evidence unless it emits a `RocketReceipt` with a valid `prev_hash` link.

## Forbidden collapse

```
test passed ≠ standing
screenshot changed ≠ game law admitted
UE4 loaded ≠ actuation admitted
receipt file exists ≠ receipt chain valid
```

## Milestone

`GC-TEST-UTILS-001` — `PARTIAL_ALIVE_CANDIDATE`

Next falsifier: `GC-TEST-UTILS-002` — use this package to prove the full actuation trace:
DOM control → GameIntent → Supabase persistence → realtime event → UE4 bridge → canvas visual delta → Playwright evidence → receipt chain replay.
