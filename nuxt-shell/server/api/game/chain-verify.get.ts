/**
 * GET /api/game/chain-verify?session_id=<uuid>
 *
 * Calls the `verify_event_chain` Postgres RPC to validate that the ocel_events
 * hash chain for the given session is intact.
 *
 * Van der Aalst doctrine: server-side proof, not browser-side assertion.
 * Returns { ok, message, broken_at, session_id } per session.
 */
import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = typeof query.session_id === 'string' ? query.session_id : null;

  const config = useRuntimeConfig(event);
  const supabaseUrl = config.public.supabaseUrl as string;
  const serviceKey = config.supabaseServiceRoleKey as string;

  if (!supabaseUrl || !serviceKey) {
    throw createError({
      statusCode: 503,
      message: 'Supabase not configured — set SUPABASE_URL and SUPABASE_SERVICE_ROLE_KEY',
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const supabase = createClient<any>(supabaseUrl, serviceKey);

  const { data, error } = sessionId
    ? await supabase.rpc('verify_event_chain', { p_session_id: sessionId })
    : await supabase.rpc('verify_event_chain', {});

  if (error) {
    throw createError({ statusCode: 500, message: error.message });
  }

  // data is an array of { ok, message, broken_at, session_id } rows
  const rows = (data ?? []) as Array<{
    ok: boolean;
    message: string;
    broken_at: number | null;
    session_id: string;
  }>;

  const allOk = rows.every((r) => r.ok);
  const breaks = rows.filter((r) => !r.ok);

  return {
    overall: allOk ? 'PASS' : 'FAIL',
    sessions_checked: rows.length,
    breaks,
    rows,
  };
});
