/**
 * POST /api/test/reset-data
 *
 * Truncates the gameplay tables for a deterministic E2E baseline. Without this,
 * sessions / receipts / OCEL rows / leaderboard entries accumulate across runs,
 * making count-based assertions (leaderboard rank, pass_rate) drift and flake.
 *
 * Body: { confirm: true, scope?: 'all' | 'sessions' }
 *   confirm:true is required — a guard against accidental invocation.
 *   scope 'all' (default) clears ocel_events + game_receipts + game_sessions +
 *   leaderboard. scope 'sessions' keeps the leaderboard/players intact.
 *
 * Security: test/dev only (same gate as session-seed) AND only ever touches a
 * LOCAL Supabase URL. Refuses to run against a non-localhost SUPABASE_URL so it
 * can never wipe a hosted project. Uses the service-role key (server-side only).
 *
 * Pattern: dashboard.bak/tests/setup.ts (TestCleanup.cleanupTestData).
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);

  const nodeEnv = process.env.NODE_ENV ?? 'production';
  const allow = process.env.ALLOW_SESSION_SEED === '1' || nodeEnv === 'test' || nodeEnv === 'development';
  if (!allow) {
    throw createError({ statusCode: 403, statusMessage: 'reset-data is test/dev only (set ALLOW_SESSION_SEED=1)' });
  }

  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  // Hard refuse to wipe anything that is not an explicitly local instance.
  if (!/(localhost|127\.0\.0\.1|::1)/.test(supabaseUrl)) {
    throw createError({ statusCode: 403, statusMessage: `reset-data refuses non-local SUPABASE_URL (${supabaseUrl})` });
  }

  const serviceKey = config.supabaseServiceRoleKey as string;
  if (!serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'SUPABASE_SERVICE_ROLE_KEY not set' });
  }

  const body = await readBody(event).catch(() => ({}));
  if (body?.confirm !== true) {
    throw createError({ statusCode: 400, statusMessage: 'reset-data requires { confirm: true }' });
  }
  const scope: string = body?.scope ?? 'all';

  const sb = createClient(supabaseUrl, serviceKey, { auth: { persistSession: false } });

  // Delete in FK-safe order: ocel_events → game_receipts → game_sessions.
  // (ocel_events + game_receipts both reference game_sessions.)
  const deleted: Record<string, boolean> = {};
  const nukeAll = async (table: string) => {
    // `id IS NOT NULL` matches every row and works for both bigint and uuid ids.
    const { error } = await sb.from(table).delete().not('id', 'is', null);
    deleted[table] = !error;
    if (error) deleted[`${table}_error`] = true;
  };

  await nukeAll('ocel_events');
  await nukeAll('game_receipts');
  await nukeAll('game_sessions');
  if (scope === 'all') {
    await nukeAll('leaderboard');
  }

  return { ok: true, scope, deleted, supabase_url: supabaseUrl };
});
