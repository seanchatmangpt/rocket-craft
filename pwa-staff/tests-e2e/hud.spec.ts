import { test, expect } from '@playwright/test';

test.use({ serviceWorkers: 'block' });

let errors: Error[] = [];
let consoleErrors: string[] = [];

test.beforeEach(({ page }) => {
  errors = [];
  consoleErrors = [];
  page.on('pageerror', (err) => {
    errors.push(err);
  });
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      const text = msg.text();
      const isExpected =
        text.includes('Failed to load resource') ||
        text.includes('Failed to fetch database stats') ||
        text.includes('Cannot submit score: Unauthenticated');
      if (!isExpected) {
        consoleErrors.push(text);
      }
    }
  });
});

test.afterEach(() => {
  expect(errors).toEqual([]);
  expect(consoleErrors).toEqual([]);
});

test('HUD functionality e2e flow', async ({ page }) => {
  // Navigate to /login.html
  await page.goto('/login.html');

  // Verify toggle button is present
  const toggleBtn = page.locator('.hud-toggle-btn');
  await expect(toggleBtn).toBeVisible();

  // Verify drawer starts with class hud-collapsed
  const drawer = page.locator('.hud-drawer');
  await expect(drawer).toHaveClass(/hud-collapsed/);

  // Click toggle button and verify drawer is visible and loses hud-collapsed
  await toggleBtn.click();
  await expect(drawer).toBeVisible();
  await expect(drawer).not.toHaveClass(/hud-collapsed/);

  // Verify #hud-auth-details contains 'Unauthenticated'
  const authDetails = page.locator('#hud-auth-details');
  await expect(authDetails).toContainText('Unauthenticated');

  // Handle and verify the browser alert block when clicking #dev-hud-submit-score-btn while unauthenticated
  let dialogMessage = '';
  page.once('dialog', async (dialog) => {
    dialogMessage = dialog.message();
    await dialog.dismiss();
  });
  await page.click('#dev-hud-submit-score-btn');
  expect(dialogMessage).toBe('You must be logged in to submit a score.');

  // Close drawer using .hud-close-btn and verify it gets the hud-collapsed class back
  await page.click('.hud-close-btn');
  await expect(drawer).toHaveClass(/hud-collapsed/);

  // Navigate to /signup.html, register a new user dynamically with a random suffix, redirecting to /profile.html
  await page.goto('/signup.html');
  const randomSuffix = Math.random().toString(36).substring(7);
  const email = `hud-user-${randomSuffix}@example.com`;
  const password = 'password123';

  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/profile.html');

  // Open HUD drawer on /profile.html
  await page.click('.hud-toggle-btn');
  await expect(drawer).toBeVisible();
  await expect(drawer).not.toHaveClass(/hud-collapsed/);

  // Verify HUD display contains email, role, and User ID (UUID)
  await expect(authDetails).toContainText(`Email:${email}`);
  await expect(authDetails).toContainText('Role:authenticated');
  await expect(authDetails).toContainText(
    /User ID:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i
  );

  // Verify database stats elements display fetched numbers (no 'Loading...' or 'Error')
  const playersVal = page.locator('#hud-stats-players');
  const sessionsVal = page.locator('#hud-stats-sessions');

  await expect(playersVal).not.toContainText('Loading');
  await expect(playersVal).not.toContainText('Error');
  await expect(playersVal).toHaveText(/^\d+$/);

  await expect(sessionsVal).not.toContainText('Loading');
  await expect(sessionsVal).not.toContainText('Error');
  await expect(sessionsVal).toHaveText(/^\d+$/);

  // Click #dev-hud-refresh-stats-btn and verify
  await page.click('#dev-hud-refresh-stats-btn');
  await expect(playersVal).not.toContainText('Loading');
  await expect(playersVal).not.toContainText('Error');
  await expect(playersVal).toHaveText(/^\d+$/);

  await expect(sessionsVal).not.toContainText('Loading');
  await expect(sessionsVal).not.toContainText('Error');
  await expect(sessionsVal).toHaveText(/^\d+$/);

  // Click #dev-hud-submit-score-btn and verify #dev-hud-console logs "Mock score submission successful"
  await page.click('#dev-hud-submit-score-btn');
  const devHudConsole = page.locator('#dev-hud-console');
  await expect(devHudConsole).toContainText('Mock score submission successful');

  // Logout from profile page and verify navigation to /login.html and that opening HUD displays 'Unauthenticated'
  await page.click('#logout-button');
  await page.waitForURL('**/login.html');

  await page.click('.hud-toggle-btn');
  await expect(drawer).toBeVisible();
  await expect(drawer).not.toHaveClass(/hud-collapsed/);
  await expect(authDetails).toContainText('Unauthenticated');
});
