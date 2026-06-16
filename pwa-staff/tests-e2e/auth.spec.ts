import { test, expect } from '@playwright/test';

test('user authentication flow', async ({ page }) => {
  const randomSuffix = Math.random().toString(36).substring(7);
  const email = `user-${randomSuffix}@example.com`;
  const password = 'password123';

  // --- Sign up ---
  await page.goto('/signup.html');
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/profile.html');

  // --- Verify profile ---
  await expect(page.locator('body')).toContainText(email);

  // --- Logout ---
  await page.click('button:has-text("Logout")');
  await page.waitForURL('**/login.html');

  // --- Login ---
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/profile.html');

  // --- Verify profile again ---
  await expect(page.locator('body')).toContainText(email);

  // --- Logout again ---
  await page.click('button:has-text("Logout")');
  await page.waitForURL('**/login.html');
});
