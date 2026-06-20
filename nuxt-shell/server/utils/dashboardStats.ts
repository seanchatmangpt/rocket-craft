/**
 * server/utils/dashboardStats.ts
 *
 * Pure aggregation logic extracted from dashboard-stats.get.ts.
 * Testable without Nitro or Supabase.
 */

export interface ReceiptRow {
  proven_at: string;
  verdict: string;
  engine_source: string;
  ocel_event_count: number;
}

export interface DayBucket {
  day: string;
  sessions: null;
  unique_players: null;
  receipts: number;
  pass_receipts: number;
  fail_receipts: number;
  real_ue4_receipts: number;
  avg_ocel_events: number;
  ocel_total: number;
  pass_rate_pct: number | null;
}

export function aggregateByDay(rows: ReceiptRow[], topN = 7): DayBucket[] {
  const byDay = new Map<string, {
    receipts: number; pass_receipts: number; fail_receipts: number;
    real_ue4_receipts: number; avg_ocel_events: number; ocel_total: number;
  }>();

  for (const row of rows) {
    const day = row.proven_at.slice(0, 10);
    const bucket = byDay.get(day) ?? {
      receipts: 0, pass_receipts: 0, fail_receipts: 0,
      real_ue4_receipts: 0, avg_ocel_events: 0, ocel_total: 0,
    };
    bucket.receipts++;
    if (row.verdict === 'PASS') bucket.pass_receipts++;
    if (row.verdict === 'FAIL') bucket.fail_receipts++;
    if (row.engine_source === 'rocket_cli' || row.engine_source === 'real_ue4') bucket.real_ue4_receipts++;
    bucket.ocel_total += row.ocel_event_count ?? 0;
    bucket.avg_ocel_events = bucket.ocel_total / bucket.receipts;
    byDay.set(day, bucket);
  }

  return [...byDay.entries()]
    .map(([day, b]) => ({
      day,
      sessions: null,
      unique_players: null,
      ...b,
      pass_rate_pct: b.receipts > 0 ? Math.round(100 * b.pass_receipts / b.receipts) : null,
    }))
    .sort((a, b) => b.day.localeCompare(a.day))
    .slice(0, topN);
}
