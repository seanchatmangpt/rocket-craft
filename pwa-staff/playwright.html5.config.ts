import { defineConfig, devices } from '@playwright/test';

// Playwright config for real UE4 HTML5 WebGL2 proof (Stage 6).
// Serves the cooked package via `rocket html5 serve` on port 8080.
// Run with: TARGET_GAME_URL=/Brm.html npx playwright test --config playwright.html5.config.ts

export default defineConfig({
  testDir: './tests-e2e',
  testMatch: '**/tps-dflss.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  timeout: 240000, // 4 min — UE4 HTML5 needs 30s+ to stream assets and render first frame; extra for baseline+input
  reporter: [['html', { outputFolder: 'html5-report' }], ['list']],
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
        // headless: false uses macOS Metal GPU directly → real WebGL2.
        // Chromium's new headless mode disables the GPU process entirely;
        // --use-gl=swiftshader does not override that. On a Mac dev machine
        // with a display, non-headless is the correct approach.
        headless: false,
      },
    },
  ],
  webServer: {
    // rocket html5 serve will be started externally by verify_html5_pipeline.sh
    // reuseExistingServer allows the test to connect to it
    command: 'rocket html5 serve --port 8080',
    url: 'http://localhost:8080',
    reuseExistingServer: true,
    stdout: 'pipe',
    stderr: 'pipe',
    timeout: 10000,
  },
});
