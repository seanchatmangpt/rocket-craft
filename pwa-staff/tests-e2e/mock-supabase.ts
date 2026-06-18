import { Page } from '@playwright/test';

export async function setupSupabaseMock(page: Page) {
  await page.route('**/auth/v1/signup*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        access_token: 'fake-token',
        user: { id: '12345678-1234-1234-1234-123456789012', email: 'test@example.com', role: 'authenticated' },
      }),
    });
  });

  await page.route('**/auth/v1/token?grant_type=password*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        access_token: 'fake-token',
        user: { id: '12345678-1234-1234-1234-123456789012', email: 'test@example.com', role: 'authenticated' },
      }),
    });
  });

  await page.route('**/auth/v1/user*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        id: '12345678-1234-1234-1234-123456789012', email: 'test@example.com', role: 'authenticated'
      }),
    });
  });

  await page.route('**/auth/v1/logout*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({}),
    });
  });

  await page.route('**/rest/v1/telemetry_logs*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([]),
    });
  });

  await page.route('**/rest/v1/players*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      headers: {
        'content-range': '0-0/42'
      },
      body: JSON.stringify([{count: 42}]),
    });
  });

  await page.route('**/rest/v1/game_sessions*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      headers: {
        'content-range': '0-0/42'
      },
      body: JSON.stringify([{count: 42}]),
    });
  });

  await page.route('**/rest/v1/world_specs*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([{ player_id: '12345678-1234-1234-1234-123456789012', spec: { places: [] } }]),
    });
  });

  await page.route('**/rest/v1/rpc/submit_mock_score*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true }),
    });
  });
}
