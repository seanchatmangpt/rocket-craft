/**
 * playwright.ue4.ci.config.ts — HEADLESS real-UE4 WebGL2 config for CI.
 *
 * The headed config (playwright.ue4.config.ts) needs a real GPU (Metal). This
 * variant runs HEADLESS with SwiftShader (software WebGL2) so the real-UE4 loop
 * can run on a GPU-less CI runner — closing the last "needs a human/GPU" gap in
 * end-to-end testability. Same spec, same assertions (incl. control-plane → canvas
 * key delivery); only the GL backend differs.
 *
 * Run: npx playwright test --config playwright.ue4.ci.config.ts
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
    ['json', { outputFile: 'playwright-report/real-ue4-ci-results.json' }],
    ['line'],
  ],
  timeout: 5 * 60 * 1000,

  use: {
    baseURL: 'http://localhost:3000',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    headless: true,
    launchOptions: {
      args: [
        // Software WebGL2 — no physical GPU needed (works on Linux/macOS CI runners).
        '--use-gl=angle',
        '--use-angle=swiftshader',
        '--enable-unsafe-swiftshader',
        '--enable-features=SharedArrayBuffer,WebAssemblyThreads',
        '--disable-web-security',
      ],
    },
  },

  projects: [
    {
      name: 'real-ue4-ci',
      use: {
        browserName: 'chromium',
        viewport: { width: 1280, height: 720 },
      },
    },
  ],
});
