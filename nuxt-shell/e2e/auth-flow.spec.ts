/**
 * auth-flow.spec.ts — proves the REAL Supabase auth path end-to-end, headlessly.
 *
 * Until now the game was only reachable via the ALLOW_ANON_GAME dev bypass, so the
 * login → session → protected-route flow was never actually exercised. This test:
 *   1. Seeds a real auth.users row via POST /api/test/seed-auth-user (service role).
 *   2. Drives the actual login form (index.vue) with those credentials.
 *   3. Asserts the user lands on /game authenticated (not bounced back to login).
 *
 * Fast — no UE4 wasm load required; it stops at the authenticated shell. Run with:
 *   npx playwright test e2e/auth-flow.spec.ts
 *
 * Requires a running stack (Supabase + Nuxt on :3000) — the default config reuses it.
 */

import { test, expect } from '@playwright/test';
import { typeInto } from './helpers';

test.describe('Supabase auth flow', () => {
  test('seeded user can log in through the form and reach /game', async ({ page, request }) => {
    // 1. Seed a real auth user (test/dev-gated endpoint). Returns canonical creds.
    const seedRes = await request.post('/api/test/seed-auth-user', { data: {} });
    // If the stack isn't wired for auth (no service role), skip gracefully.
    if (seedRes.status() === 503) test.skip(true, 'Supabase service role not configured');
    expect(seedRes.ok()).toBeTruthy();
    const seeded = await seedRes.json();
    expect(seeded.user_id).toMatch(/[0-9a-f-]{36}/);

    const email = seeded.email as string;
    const password = 'rocket-craft-dev-pilot'; // default seeded password (see seed-auth-user)


    // 2. Drive the real login form. Target the native <input> elements (the
    // data-testid sits on the Nuxt UI wrapper, so fill the real inputs by
    // placeholder to update v-model and enable the submit button).
    await page.goto('/');
    // Wait for hydration to settle — early keystrokes are dropped while Vue is
    // still attaching listeners (Nuxt UI ignores .fill(), so we must type).
    await page.waitForLoadState('networkidle');
    const emailInput = page.locator('input[type="email"]');
    const passwordInput = page.locator('input[type="password"]');
    await expect(emailInput).toBeVisible({ timeout: 15_000 });

    await typeInto(page, emailInput, email);
    await typeInto(page, passwordInput, password);
    await passwordInput.blur();

    const submit = page.getByTestId('btn-auth-submit');
    await expect(submit).toBeEnabled({ timeout: 5_000 });
    await submit.click();

    // 3. Successful sign-in routes to /game (index.vue submit → router.push('/game')).
    await page.waitForURL('**/game', { timeout: 20_000 });
    expect(page.url()).toContain('/game');

    // The authenticated shell renders its engine-status HUD — proves we were NOT
    // bounced back to the login form by the auth guard.
    await expect(page.getByTestId('engine-status')).toBeVisible({ timeout: 15_000 });
  });
});
