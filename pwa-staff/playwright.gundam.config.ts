import { defineConfig, devices } from '@playwright/test';

// Playwright config for Gundam Factory Walkthrough (milestone GC-GUNDAM-FACTORY-001).
// Serves the cooked package via `rocket html5 serve` on port 8080.
// Run with: TARGET_GAME_URL=/Brm.html npx playwright test --config playwright.gundam.config.ts

export default defineConfig({
  testDir: './tests-e2e',
  testMatch: '**/gundam_factory_walkthrough_projection.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  timeout: 240000, // 4 min
  reporter: [['html', { outputFolder: 'gundam-report' }], ['list']],
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on',
    screenshot: 'on',
    video: { mode: 'on', size: { width: 1280, height: 720 } },
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        headless: false,
      },
    },
  ],
  webServer: {
    command: 'rocket html5 serve --port 8080',
    url: 'http://localhost:8080',
    reuseExistingServer: true,
    stdout: 'pipe',
    stderr: 'pipe',
    timeout: 10000,
  },
});
