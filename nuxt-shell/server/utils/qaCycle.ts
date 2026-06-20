/**
 * Pure QA cycle logic — extracted so it can be unit-tested
 * without Nitro globals (defineEventHandler, createError, etc.).
 *
 * Invariants checked per session:
 *   RECEIPT_CHAIN_INTACT: verify_event_chain RPC returns ok=true
 *   LIFECYCLE_COMPLETE: ocel_events has all three required activities
 *   NO_SYNTHETIC_SOURCE: game_receipts.engine_source !== 'synthetic'
 *   MERKLE_ROOT_PRESENT: event_hash values exist and merkle_root is computable
 */

export type QaOverall = 'HEALTHY' | 'DEGRADED' | 'CRITICAL';

export interface QaCheckResult {
  check: string;
  passed: boolean;
  evidence: Record<string, unknown>;
}

/**
 * Classify overall health from check results.
 * CRITICAL if RECEIPT_CHAIN_INTACT failed (chain integrity beats everything).
 * DEGRADED if any other check failed.
 * HEALTHY if all pass.
 */
export function classifyOverall(results: Array<{ check: string; passed: boolean }>): QaOverall {
  const chainCheck = results.find(r => r.check === 'RECEIPT_CHAIN_INTACT');
  if (chainCheck && !chainCheck.passed) return 'CRITICAL';
  if (results.some(r => !r.passed)) return 'DEGRADED';
  return 'HEALTHY';
}

export interface RawQaInputs {
  chainOk: boolean | null;
  activities: string[];
  engineSource: string | null;
  eventHashes: string[];
  merkleRoot: string | null;
}

const REQUIRED_ACTIVITIES = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'];

/**
 * Build the results[] array from raw query data — pure, no Supabase calls.
 */
export function buildCycleResult(inputs: RawQaInputs): QaCheckResult[] {
  const { chainOk, activities, engineSource, eventHashes, merkleRoot } = inputs;

  const chainIntact = chainOk === true;
  const lifecycleComplete = REQUIRED_ACTIVITIES.every(a => activities.includes(a));
  const notSynthetic = engineSource !== null && engineSource !== 'synthetic';
  const merklePresent = eventHashes.length > 0 && merkleRoot !== null && merkleRoot.length === 64;

  return [
    {
      check: 'RECEIPT_CHAIN_INTACT',
      passed: chainIntact,
      evidence: { chain_ok: chainOk },
    },
    {
      check: 'LIFECYCLE_COMPLETE',
      passed: lifecycleComplete,
      evidence: {
        required: REQUIRED_ACTIVITIES,
        found: activities,
        missing: REQUIRED_ACTIVITIES.filter(a => !activities.includes(a)),
      },
    },
    {
      check: 'NO_SYNTHETIC_SOURCE',
      passed: notSynthetic,
      evidence: { engine_source: engineSource },
    },
    {
      check: 'MERKLE_ROOT_PRESENT',
      passed: merklePresent,
      evidence: {
        event_hash_count: eventHashes.length,
        merkle_root: merkleRoot,
      },
    },
  ];
}
