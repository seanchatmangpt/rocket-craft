import { test, expect } from '@playwright/test';

/**
 * Nuxt shell E2E — tests the browser-native control plane.
 *
 * Law: DOM buttons must emit rocket:intent events. UE4 canvas is not required
 * for these tests — the control-plane surface is independently testable.
 */

test.describe('login page', () => {
  test('renders sign-in form', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByText('ROCKET-CRAFT')).toBeVisible();
    await expect(page.getByTestId('input-email')).toBeVisible();
    await expect(page.getByTestId('input-password')).toBeVisible();
    await expect(page.getByTestId('btn-auth-submit')).toBeVisible();
  });

  test('submit button disabled when fields empty', async ({ page }) => {
    await page.goto('/');
    const btn = page.getByTestId('btn-auth-submit');
    await expect(btn).toBeDisabled();
  });

  test('submit button enabled when email and password filled', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('input-email').fill('test@example.com');
    await page.getByTestId('input-password').fill('password123');
    await expect(page.getByTestId('btn-auth-submit')).toBeEnabled();
  });

  test('toggle mode switches between Sign In and Sign Up', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByTestId('btn-auth-submit')).toContainText('Sign In');
    await page.getByTestId('btn-toggle-mode').click();
    await expect(page.getByTestId('btn-auth-submit')).toContainText('Create Account');
    await page.getByTestId('btn-toggle-mode').click();
    await expect(page.getByTestId('btn-auth-submit')).toContainText('Sign In');
  });
});

test.describe('game control panel', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate directly — skip auth in dev (no Supabase URL means no redirect)
    await page.goto('/game');
  });

  test('renders all control buttons', async ({ page }) => {
    await expect(page.getByTestId('btn-start-walkthrough')).toBeVisible();
    await expect(page.getByTestId('btn-pause')).toBeVisible();
    await expect(page.getByTestId('btn-next-station')).toBeVisible();
    await expect(page.getByTestId('btn-interact')).toBeVisible();
    await expect(page.getByTestId('btn-receipts')).toBeVisible();
  });

  test('start-walkthrough button emits rocket:intent event', async ({ page }) => {
    // Capture the DOM event before clicking
    const intentPromise = page.evaluate(() =>
      new Promise<{ type: string; source: string }>((resolve) => {
        window.addEventListener('rocket:intent', (e) => {
          const detail = (e as CustomEvent).detail;
          resolve({ type: detail.intent.type, source: detail.intent.source });
        }, { once: true });
      })
    );

    await page.getByTestId('btn-start-walkthrough').click();
    const intent = await intentPromise;

    expect(intent.type).toBe('StartWalkthrough');
    expect(intent.source).toContain('start-walkthrough');
  });

  test('interact button emits Interact intent', async ({ page }) => {
    const intentPromise = page.evaluate(() =>
      new Promise<string>((resolve) => {
        window.addEventListener('rocket:intent', (e) => {
          resolve((e as CustomEvent).detail.intent.type);
        }, { once: true });
      })
    );
    await page.getByTestId('btn-interact').click();
    expect(await intentPromise).toBe('Interact');
  });

  test('keyboard E key emits Interact intent', async ({ page }) => {
    const intentPromise = page.evaluate(() =>
      new Promise<string>((resolve) => {
        window.addEventListener('rocket:intent', (e) => {
          resolve((e as CustomEvent).detail.intent.type);
        }, { once: true });
      })
    );
    // Focus body to ensure the keydown listener fires
    await page.locator('body').click();
    await page.keyboard.press('e');
    expect(await intentPromise).toBe('Interact');
  });

  test('keyboard R key emits OpenReceiptPanel intent', async ({ page }) => {
    const intentPromise = page.evaluate(() =>
      new Promise<string>((resolve) => {
        window.addEventListener('rocket:intent', (e) => {
          resolve((e as CustomEvent).detail.intent.type);
        }, { once: true });
      })
    );
    await page.locator('body').click();
    await page.keyboard.press('r');
    expect(await intentPromise).toBe('OpenReceiptPanel');
  });

  test('last-intent debug display updates after button click', async ({ page }) => {
    await page.getByTestId('btn-interact').click();
    const debug = page.getByTestId('last-intent');
    await expect(debug).toContainText('Interact');
  });
});

test.describe('receipt drawer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/game');
  });

  test('opens when R key pressed', async ({ page }) => {
    await page.locator('body').click();
    await page.keyboard.press('r');
    // Drawer should appear
    await expect(page.getByTestId('receipt-empty').or(page.getByTestId('receipt-timeline'))).toBeVisible({ timeout: 3000 });
  });

  test('opens when receipts button clicked', async ({ page }) => {
    await page.getByTestId('btn-receipts').click();
    await expect(page.getByTestId('receipt-empty').or(page.getByTestId('receipt-timeline'))).toBeVisible({ timeout: 3000 });
  });
});
