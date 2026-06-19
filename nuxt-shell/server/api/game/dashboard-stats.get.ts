/**
 * GET /api/game/dashboard-stats
 *
 * Returns daily pipeline_dashboard rollup stats for the last 7 days.
 * Backed by the materialized views from migration 000010 (session_stats_daily +
 * receipt_stats_daily) and the pipeline_dashboard view.
 *
 * Used by:
 * - pipeline.vue dashboard cards (replaces ad-hoc computed values)
 * - gameplay-loop-contract.test.ts (CONTRACT: pipeline health visibility)
 * - Playwright assertions in tps-dflss.spec.ts (Gap 3 complement)
 *
 * Pattern: dashboard.bak/server/api/stats/overview.get.ts (Evidence Center rollups)
 *
 * Returns: Array<{ day, sessions, unique_players, receipts, pass_receipts,
 *                  fail_receipts, real_ue4_receipts, avg_ocel_events, pass_rate_pct }>
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);
  const supabaseUrl = config.public.supabaseUrl as string;
  const serviceKey = config.supabaseServiceRoleKey as string;

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = createClient<any>(supabaseUrl, serviceKey);

  // Try the pipeline_dashboard view first (requires migration 000010)
  const { data: dashData, error: dashErr } = await sb
    .from('pipeline_dashboard')
    .select('*')
    .limit(7);

  if (!dashErr && dashData) {
    return { source: 'pipeline_dashboard', rows: dashData };
  }

  // Fallback: derive from game_receipts directly (works without the mat views)
  const { data: fallback, error: fbErr } = await sb
    .from('game_receipts')
    .select('proven_at, verdict, engine_source, ocel_event_count')
    .not('proven_at', 'is', null)
    .order('proven_at', { ascending: false })
    .limit(200);

  if (fbErr || !fallback) {
    throw createError({ statusCode: 500, statusMessage: dashErr?.message ?? 'Stats unavailable' });
  }

  // Aggregate by day in JS (fallback when mat views not yet migrated)
  const byDay = new Map<string, {
    receipts: number; pass_receipts: number; fail_receipts: number;
    real_ue4_receipts: number; avg_ocel_events: number; ocel_total: number;
  }>();

  for (const row of fallback as Array<{
    proven_at: string; verdict: string; engine_source: string; ocel_event_count: number;
  }>) {
    const day = row.proven_at.slice(0, 10);
    const bucket = byDay.get(day) ?? { receipts: 0, pass_receipts: 0, fail_receipts: 0, real_ue4_receipts: 0, avg_ocel_events: 0, ocel_total: 0 };
    bucket.receipts++;
    if (row.verdict === 'PASS') bucket.pass_receipts++;
    if (row.verdict === 'FAIL') bucket.fail_receipts++;
    if (row.engine_source === 'rocket_cli' || row.engine_source === 'real_ue4') bucket.real_ue4_receipts++;
    bucket.ocel_total += row.ocel_event_count ?? 0;
    bucket.avg_ocel_events = bucket.ocel_total / bucket.receipts;
    byDay.set(day, bucket);
  }

  const rows = [...byDay.entries()]
    .map(([day, b]) => ({
      day,
      sessions: null,
      unique_players: null,
      ...b,
      pass_rate_pct: b.receipts > 0 ? Math.round(100 * b.pass_receipts / b.receipts) : null,
    }))
    .sort((a, b) => b.day.localeCompare(a.day))
    .slice(0, 7);

  return { source: 'game_receipts_fallback', rows };
});
