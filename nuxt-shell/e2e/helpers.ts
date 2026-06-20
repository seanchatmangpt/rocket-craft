/**
 * Shared Playwright helpers for rocket-craft E2E specs.
 */
import type { Page, Locator } from '@playwright/test';

/**
 * Type a value into a Nuxt UI UInput reliably.
 *
 * Nuxt UI v4's UInput ignores Playwright's .fill() (v-model never updates, so the
 * submit button stays disabled), and pressSequentially drops leading characters
 * while Vue is still hydrating. This helper waits for hydration, types via real
 * keystrokes, and self-heals if the value came out incomplete.
 */
export async function typeInto(page: Page, locator: Locator, value: string): Promise<void> {
  await locator.click();
  await page.waitForTimeout(120);
  await locator.pressSequentially(value, { delay: 15 });
  if ((await locator.inputValue()) !== value) {
    await locator.press('Control+a');
    await locator.press('Backspace');
    await locator.pressSequentially(value, { delay: 25 });
  }
}

/**
 * Block the heavy UE4 HTML5 assets (/manufactured/** → 183 MB wasm) so control-
 * plane tests that only need the Nuxt DOM load fast and don't flake under
 * parallelism. Call before navigating to /game when the canvas isn't under test.
 */
export async function blockUe4Assets(page: Page): Promise<void> {
  await page.route('**/manufactured/**', (route) => route.abort());
}
