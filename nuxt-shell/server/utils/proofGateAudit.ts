/**
 * server/utils/proofGateAudit.ts
 *
 * Proof gate audit log — records every gate evaluation to `proof_gate_audits`.
 * Van der Aalst doctrine: if the gate ran but the log cannot prove it, the gate
 * did not run. Audit rows are the event evidence for the proof-gate process.
 *
 * Design: fire-and-forget insert (never blocks the hot path). Failures are
 * logged to stderr but do not propagate — the gate result is the source of
 * truth, not the audit row.
 */

import { SupabaseClient } from '@supabase/supabase-js';

export type GateOutcome = 'pass' | 'fail';

export interface GateAuditEntry {
  session_id: string;
  gate_name: string;
  outcome: GateOutcome;
  /** The input value that was evaluated (stringified, may be truncated) */
  input_summary: string;
  /** Human-readable reason — populated on fail */
  reason: string | null;
  evaluated_at: string;
}

function truncate(v: unknown, maxLen = 200): string {
  const s = typeof v === 'string' ? v : JSON.stringify(v);
  return s.length > maxLen ? s.slice(0, maxLen) + '…' : s;
}

/**
 * Write a gate audit row.  Non-blocking — returns a Promise<void> that
 * callers can optionally await but usually should not (hot path).
 */
export async function recordGateAudit(
  sb: SupabaseClient,
  entry: GateAuditEntry,
): Promise<void> {
  try {
    await sb.from('proof_gate_audits').insert({
      session_id: entry.session_id,
      gate_name: entry.gate_name,
      outcome: entry.outcome,
      input_summary: entry.input_summary,
      reason: entry.reason,
      evaluated_at: entry.evaluated_at,
    });
  } catch (err) {
    console.error('[proof-gate-audit] insert failed:', err);
  }
}

/**
 * Convenience: record all gates that ran for a session from the standard
 * `runProofGates` inputs. Pass the final GateResult to determine outcome.
 *
 * Emits one row per gate; each row is independent so partial audit trails
 * are still queryable.
 */
export function auditGateRun(
  sb: SupabaseClient,
  sessionId: string,
  input: {
    verdict?: unknown;
    milestone?: unknown;
    engine_source?: unknown;
    receipt_hash?: unknown;
    ocel_lifecycle?: unknown;
  },
  failedGate: string | null,
  failReason: string | null,
): void {
  const now = new Date().toISOString();

  const gates: Array<{ name: string; value: unknown }> = [
    { name: 'required_fields', value: { verdict: input.verdict, milestone: input.milestone, engine_source: input.engine_source, receipt_hash: !!input.receipt_hash } },
    { name: 'receipt_hash_format', value: input.receipt_hash },
    { name: 'not_synthetic', value: input.engine_source },
    { name: 'lifecycle_complete', value: input.ocel_lifecycle },
  ];

  for (const g of gates) {
    const failed = g.name === failedGate;
    // Once a gate fails, all subsequent gates are considered not-reached (skip them)
    const reached = failedGate === null || gates.indexOf(g) <= gates.findIndex(x => x.name === failedGate);
    if (!reached) continue;

    void recordGateAudit(sb, {
      session_id: sessionId,
      gate_name: g.name,
      outcome: failed ? 'fail' : 'pass',
      input_summary: truncate(g.value),
      reason: failed ? failReason : null,
      evaluated_at: now,
    });
  }
}
