/**
 * GET /api/game/health-lies
 *
 * Server-side pipeline invariant scanner — replaces 3 parallel client-side
 * Supabase queries in useHealthLieDetector. Service role key never leaks.
 *
 * Checks:
 *   LIE-1: PASS receipt with ocel_event_count=0 (impossible without evidence)
 *   LIE-2: session alive > 10 min with no close (stale session leak)
 *   LIE-4: engine_source='synthetic' in DB (guard trigger should block these)
 *
 * Returns:
 *   {
 *     lies: HealthLie[],
 *     scanned_at: string,
 *     all_clear: boolean,
 *   }
 */

import { createClient } from '@supabase/supabase-js';

interface HealthLie {
  code: 'LIE-1' | 'LIE-2' | 'LIE-4';
  description: string;
  evidence: Record<string, unknown>;
}

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string);

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);
  const tenMinAgo = new Date(Date.now() - 10 * 60 * 1000).toISOString();

  const [lie1Res, lie2Res, lie4Res] = await Promise.allSettled([
    sb.from('game_receipts')
      .select('id, verdict, ocel_event_count')
      .eq('verdict', 'PASS')
      .eq('ocel_event_count', 0)
      .limit(5),
    sb.from('game_sessions')
      .select('id, session_started_at, project_name')
      .eq('is_alive', true)
      .lt('session_started_at', tenMinAgo)
      .limit(5),
    sb.from('game_receipts')
      .select('id, engine_source')
      .eq('engine_source', 'synthetic')
      .limit(5),
  ]);

  const lies: HealthLie[] = [];

  if (lie1Res.status === 'fulfilled' && lie1Res.value.data?.length) {
    lies.push({
      code: 'LIE-1',
      description: `${lie1Res.value.data.length} PASS receipt(s) claim zero OCEL events — impossible without evidence`,
      evidence: { receipts: lie1Res.value.data.map((r: { id: string }) => r.id) },
    });
  }

  if (lie2Res.status === 'fulfilled' && lie2Res.value.data?.length) {
    lies.push({
      code: 'LIE-2',
      description: `${lie2Res.value.data.length} session(s) alive >10 min with no close — stale session leak`,
      evidence: {
        sessions: lie2Res.value.data.map((s: { id: string; project_name: string }) => ({
          id: s.id, project: s.project_name,
        })),
      },
    });
  }

  if (lie4Res.status === 'fulfilled' && lie4Res.value.data?.length) {
    lies.push({
      code: 'LIE-4',
      description: `${lie4Res.value.data.length} receipt(s) with engine_source=synthetic bypassed the guard trigger`,
      evidence: { receipts: lie4Res.value.data.map((r: { id: string }) => r.id) },
    });
  }

  return {
    lies,
    scanned_at: new Date().toISOString(),
    all_clear: lies.length === 0,
  };
});
