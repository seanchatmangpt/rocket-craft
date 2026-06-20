import { test, expect } from '@playwright/test';
import { blockUe4Assets } from './helpers';

/**
 * Operator-facing pipeline dashboard smoke test (was untested — the main 704-line
 * read surface). Proves /pipeline renders LIVE health data end-to-end (SSR +
 * client revalidate against /api/game/pipeline-health), not an error/loading state.
 *
 * Seeds a session first so the metrics are non-trivial. Runs DOM-only (UE4 wasm
 * blocked) — the dashboard doesn't need the engine.
 */

test.describe('pipeline dashboard', () => {
  test('renders live health metrics after a seeded session', async ({ page, request }) => {
    // Seed a proven session so total_receipts > 0.
    const seed = await request.post('/api/game/session-seed', { data: { create_test_player: true } });
    if (seed.status() === 503) test.skip(true, 'Supabase unavailable');
    expect(seed.ok()).toBeTruthy();

    await blockUe4Assets(page);
    await page.goto('/pipeline');

    // Health score card + metrics grid must render (not the loading/error state).
    await expect(page.getByTestId('health-score')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('metrics-grid')).toBeVisible({ timeout: 15_000 });

    // Total receipts must be a real number ≥ 1 (the seeded receipt is counted) —
    // proves the dashboard is wired to live pipeline-health data, not a placeholder.
    const totalText = (await page.getByTestId('metric-total-receipts').textContent())?.trim() ?? '';
    expect(totalText).toMatch(/^\d+$/);
    expect(Number(totalText)).toBeGreaterThanOrEqual(1);

    // The page header confirms we're on the dashboard, not bounced to login.
    await expect(page.getByRole('heading', { name: 'Pipeline Health' })).toBeVisible();
  });
});
