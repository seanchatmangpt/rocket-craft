/**
 * Pure lie detection logic — extracted so it can be unit-tested
 * without Nitro globals (defineEventHandler, createError, etc.).
 *
 * Invariants:
 *   LIE-1: PASS receipt with ocel_event_count=0 — evidence-free claim
 *   LIE-2: session alive >10 min with no close — stale session leak
 *   LIE-4: engine_source='synthetic' in DB — cook-receipt guard bypass
 *   LIE-5: FAIL receipt where events form a lawful chain (false-negative rejection)
 *   LIE-6: game_receipts row with verdict=NULL (never finalized — stuck pipeline)
 */

export type LieCode = 'LIE-1' | 'LIE-2' | 'LIE-4' | 'LIE-5' | 'LIE-6'

export interface HealthLie {
  code: LieCode
  description: string
  evidence: Record<string, unknown>
}

export function detectLies(
  lie1Rows: Array<{ id: string }> | null,
  lie2Rows: Array<{ id: string; project_name?: string }> | null,
  lie4Rows: Array<{ id: string }> | null,
  lie5Rows?: Array<{ id: string; session_id: string }> | null,
  lie6Rows?: Array<{ id: string; session_id: string }> | null,
): HealthLie[] {
  const lies: HealthLie[] = []

  if (lie1Rows?.length) {
    lies.push({
      code: 'LIE-1',
      description: `${lie1Rows.length} PASS receipt(s) claim zero OCEL events — impossible without evidence`,
      evidence: { receipts: lie1Rows.map(r => r.id) },
    })
  }

  if (lie2Rows?.length) {
    lies.push({
      code: 'LIE-2',
      description: `${lie2Rows.length} session(s) alive >10 min with no close — stale session leak`,
      evidence: { sessions: lie2Rows.map(s => ({ id: s.id, project: s.project_name })) },
    })
  }

  if (lie4Rows?.length) {
    lies.push({
      code: 'LIE-4',
      description: `${lie4Rows.length} receipt(s) with engine_source=synthetic bypassed the guard trigger`,
      evidence: { receipts: lie4Rows.map(r => r.id) },
    })
  }

  // LIE-5: receipts marked FAIL but the session has a non-zero lawful event chain
  // Indicates a false-negative rejection — proof gate may be misconfigured
  if (lie5Rows?.length) {
    lies.push({
      code: 'LIE-5',
      description: `${lie5Rows.length} FAIL receipt(s) on sessions that have ≥1 OCEL event — possible false rejection`,
      evidence: { receipts: lie5Rows.map(r => ({ receipt_id: r.id, session_id: r.session_id })) },
    })
  }

  // LIE-6: receipts with verdict IS NULL — pipeline stalled, never finalized
  if (lie6Rows?.length) {
    lies.push({
      code: 'LIE-6',
      description: `${lie6Rows.length} receipt(s) with verdict=NULL — pipeline never finalized`,
      evidence: { receipts: lie6Rows.map(r => ({ receipt_id: r.id, session_id: r.session_id })) },
    })
  }

  return lies
}
