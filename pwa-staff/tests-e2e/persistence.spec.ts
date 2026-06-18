import { test, expect } from '@playwright/test';
import { createClient } from '@supabase/supabase-js';

if (typeof globalThis.WebSocket === 'undefined') {
  class MockWebSocket {
    constructor(url: string, protocols?: string | string[]) {}
    addEventListener() {}
    removeEventListener() {}
  }
  globalThis.WebSocket = MockWebSocket as any;
}

const supabaseUrl = process.env.SUPABASE_URL || 'http://127.0.0.1:54321';
const supabaseAnonKey = process.env.SUPABASE_ANON_KEY || 'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH';
const supabase = createClient(supabaseUrl, supabaseAnonKey);

test('PWA canvas mounts, receipt displays, spec persists, and page works offline', async ({ page }) => {
  const randomSuffix = Math.random().toString(36).substring(7);
  const email = `persistence-${randomSuffix}@example.com`;
  const password = 'password123';

  // 1. Sign up user
  await page.goto('/signup.html');
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForURL('**/profile.html');

  // 2. Assert canvas mounts and receipt details are visible
  const canvas = page.locator('#canvas');
  await expect(canvas).toBeVisible();

  const receiptDetails = page.locator('#receipt-details');
  await expect(receiptDetails).not.toContainText('Loading receipt', { timeout: 10000 });
  await expect(receiptDetails).toContainText('Hash:');
  await expect(receiptDetails).toContainText('Issued At:');

  // 3. Authenticate Supabase client and verify spec persistence in database
  const { data: authData, error: authError } = await supabase.auth.signInWithPassword({
    email,
    password,
  });
  expect(authError).toBeNull();
  const userId = authData.user?.id;
  expect(userId).toBeDefined();

  // Allow some time for database write
  await page.waitForTimeout(1000);

  const { data: specData, error: specError } = await supabase
    .from('world_specs')
    .select('*')
    .eq('player_id', userId)
    .single();

  expect(specError).toBeNull();
  expect(specData).toBeDefined();
  expect(specData.player_id).toBe(userId);
  expect(specData.spec).toBeDefined();
  expect(specData.spec.places).toBeDefined();

  // 4. Test offline loading support
  await page.context().setOffline(true);
  await page.reload();

  // Verify that the canvas and profile are still loaded successfully
  const offlineCanvas = page.locator('#canvas');
  await expect(offlineCanvas).toBeVisible();
  await expect(page.locator('body')).toContainText(email);
});
