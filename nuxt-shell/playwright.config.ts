import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false, // game loop tests need ordered execution
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'playwright-report/results.json' }],
  ],
  timeout: 60_000,
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on',
    video: 'on', // record video of every run — proof of life
    screenshot: 'on',
    // SharedArrayBuffer requires these launch args (UE4 wasm-threads)
    launchOptions: {
      args: [
        '--enable-features=SharedArrayBuffer',
        '--disable-web-security', // needed for cross-origin UE4 module load in dev
      ],
    },
  },
  projects: [
    {
      name: 'game-loop',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1280, height: 720 },
      },
    },
  ],
  // Nuxt dev server — reuse if already running (e.g. from `rocket html5 e2e`)
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
