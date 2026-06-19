/**
 * playwright.ue4.config.ts — Playwright config for REAL UE4 WebGL2 sessions.
 *
 * Used by: `rocket html5 e2e --headed` and `npx playwright test --config playwright.ue4.config.ts`
 *
 * Key differences from playwright.config.ts (CI suite):
 *   - headless: FALSE — WebGL2 requires a real GPU context (Apple M3 Metal)
 *   - timeout: 5 minutes — 175 MB WASM takes time on first load
 *   - Only runs real-ue4-game-loop.spec.ts (not the synthetic CI suite)
 *   - GPU process enabled — required for Metal WebGL2 on macOS
 *   - No /manufactured/ blocking — real bundle must load
 *
 * Prerequisites:
 *   1. `rocket html5 serve --project Brm --background`   # start asset server :8080
 *   2. `cd nuxt-shell && pnpm dev`                       # start Nuxt shell :3000
 *   3. `npx playwright test --config playwright.ue4.config.ts`
 *
 * Or one-shot: `rocket html5 e2e --project Brm --headed`
 */

import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  testMatch: '**/real-ue4-game-loop.spec.ts',
  fullyParallel: false,
  forbidOnly: false,
  retries: 0,
  workers: 1,
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'playwright-report/real-ue4-results.json' }],
    ['line'],
  ],
  // 5 minutes: 175 MB WASM load + WebGL2 init + settle time
  timeout: 5 * 60 * 1000,

  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on',
    video: 'on',
    screenshot: 'on',
    // Headed required — headless Chromium does not initialise WebGL2 via Metal
    headless: false,
    launchOptions: {
      args: [
        '--enable-features=SharedArrayBuffer',
        '--use-gl=angle',           // ANGLE → Metal GPU on macOS
        '--use-angle=metal',        // explicitly request Metal backend
        '--enable-gpu',
        '--disable-gpu-sandbox',    // required for Metal in Playwright subprocess
        '--disable-web-security',   // cross-origin UE4 module load in dev
        // SharedArrayBuffer / wasm-threads requires COOP+COEP — asset server provides these
        '--enable-features=SharedArrayBuffer,WebAssemblyThreads',
      ],
    },
  },

  projects: [
    {
      name: 'real-ue4-chromium',
      use: {
        browserName: 'chromium',
        viewport: { width: 1280, height: 720 },
      },
    },
  ],
});
