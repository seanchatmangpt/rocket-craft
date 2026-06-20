/**
 * Pure lie detection logic — extracted so it can be unit-tested
 * without Nitro globals (defineEventHandler, createError, etc.).
 *
 * Invariants:
 *   LIE-1: PASS receipt with ocel_event_count=0 — evidence-free claim
 *   LIE-2: session alive >10 min with no close — stale session leak
 *   LIE-4: engine_source='synthetic' in DB — cook-receipt guard bypass
 */

export interface HealthLie {
  code: 'LIE-1' | 'LIE-2' | 'LIE-4';
  description: string;
  evidence: Record<string, unknown>;
}

export function detectLies(
  lie1Rows: Array<{ id: string }> | null,
  lie2Rows: Array<{ id: string; project_name?: string }> | null,
  lie4Rows: Array<{ id: string }> | null,
): HealthLie[] {
  const lies: HealthLie[] = [];
  if (lie1Rows?.length) {
    lies.push({
      code: 'LIE-1',
      description: `${lie1Rows.length} PASS receipt(s) claim zero OCEL events — impossible without evidence`,
      evidence: { receipts: lie1Rows.map(r => r.id) },
    });
  }
  if (lie2Rows?.length) {
    lies.push({
      code: 'LIE-2',
      description: `${lie2Rows.length} session(s) alive >10 min with no close — stale session leak`,
      evidence: { sessions: lie2Rows.map(s => ({ id: s.id, project: s.project_name })) },
    });
  }
  if (lie4Rows?.length) {
    lies.push({
      code: 'LIE-4',
      description: `${lie4Rows.length} receipt(s) with engine_source=synthetic bypassed the guard trigger`,
      evidence: { receipts: lie4Rows.map(r => r.id) },
    });
  }
  return lies;
}
