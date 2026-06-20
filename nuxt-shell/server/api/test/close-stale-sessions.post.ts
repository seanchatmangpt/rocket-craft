/**
 * POST /api/test/close-stale-sessions
 *
 * Headless trigger for the close_stale_sessions() stored procedure (migration
 * 0008). In production this runs via pg_cron every 5 minutes (migration 0009),
 * but pg_cron is NOT available in local Supabase — so the zombie-session cleanup
 * had no way to be exercised or asserted in tests. This endpoint closes that gap.
 *
 * Body: { timeout_minutes?: number }  (default 10; tests pass 0 to close
 *        just-created alive sessions immediately)
 * Returns: { closed_count, closed: [{ closed_session_id, age_minutes, event_count }] }
 *
 * Security: test/dev only (same gate as the other test endpoints), localhost
 * Supabase only. Uses the service-role key.
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);

  const nodeEnv = process.env.NODE_ENV ?? 'production';
  const allow = process.env.ALLOW_SESSION_SEED === '1' || nodeEnv === 'test' || nodeEnv === 'development';
  if (!allow) {
    throw createError({ statusCode: 403, statusMessage: 'close-stale-sessions is test/dev only (set ALLOW_SESSION_SEED=1)' });
  }

  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  if (!/(localhost|127\.0\.0\.1|::1)/.test(supabaseUrl)) {
    throw createError({ statusCode: 403, statusMessage: `refuses non-local SUPABASE_URL (${supabaseUrl})` });
  }
  const serviceKey = config.supabaseServiceRoleKey as string;
  if (!serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'SUPABASE_SERVICE_ROLE_KEY not set' });
  }

  const body = await readBody(event).catch(() => ({}));
  const timeoutMinutes = typeof body?.timeout_minutes === 'number' ? body.timeout_minutes : 10;

  const sb = createClient<any>(supabaseUrl, serviceKey);
  const { data, error } = await sb.rpc('close_stale_sessions', { p_timeout_minutes: timeoutMinutes });
  if (error) {
    throw createError({ statusCode: 500, statusMessage: `close_stale_sessions failed: ${error.message}` });
  }

  const closed = (data ?? []) as Array<{ closed_session_id: string; age_minutes: number; event_count: number }>;
  return { closed_count: closed.length, timeout_minutes: timeoutMinutes, closed };
});
