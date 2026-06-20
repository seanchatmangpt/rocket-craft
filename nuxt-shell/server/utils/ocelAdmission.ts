/**
 * server/utils/ocelAdmission.ts
 *
 * Pre-Admission Tension Queue (PATQ) gate for OCEL events.
 * Ported from truex/TrueXSyncReplicationEngine AdmissionGate pattern.
 *
 * Every event batch MUST pass admission before insertion. Gates check:
 *   1. event_hash format — 64-char lowercase hex (BLAKE3)
 *   2. seq contiguity — batch must have no gaps or duplicates
 *   3. prev_hash threading — each event's prev_hash equals predecessor's event_hash
 *   4. session_id consistency — all events in batch share the same session_id
 *   5. required fields — activity, timestamp_ms, event_hash, seq present
 *
 * These checks happen BEFORE upsert so broken chains are never silently stored.
 * The idempotent-receiver upsert (on conflict ignore) means retried valid batches
 * are safe; retried invalid batches are rejected before they touch the DB.
 */

export interface OcelEventCandidate {
  activity: string
  timestamp_ms: number
  object_refs?: string[]
  attributes?: Record<string, unknown>
  event_hash: string
  prev_hash: string | null
  seq: number
  session_id?: string
}

export type AdmissionVerdict = 'admit' | 'reject' | 'quarantine'

export interface AdmissionResult {
  verdict: AdmissionVerdict
  /** HTTP status code to return on rejection */
  statusCode?: number
  /** Human-readable reason for reject/quarantine */
  reason?: string
  /** Index of the first offending event (0-based) */
  offending_index?: number
}

const BLAKE3_HEX_RE = /^[0-9a-f]{64}$/

// ── Individual event validators ──────────────────────────────────────────────

export function validateEventHash(hash: unknown): boolean {
  return typeof hash === 'string' && BLAKE3_HEX_RE.test(hash)
}

export function validateRequiredFields(evt: Partial<OcelEventCandidate>): boolean {
  return (
    typeof evt.activity === 'string' && evt.activity.length > 0 &&
    typeof evt.timestamp_ms === 'number' && evt.timestamp_ms > 0 &&
    typeof evt.seq === 'number' && evt.seq >= 0 &&
    validateEventHash(evt.event_hash)
  )
}

// ── Batch validators ─────────────────────────────────────────────────────────

/**
 * Verify seq numbers form a contiguous range starting at `expectedStartSeq`.
 * Returns the index of the first gap/duplicate, or -1 if contiguous.
 */
export function findSeqGap(events: OcelEventCandidate[], expectedStartSeq: number): number {
  for (let i = 0; i < events.length; i++) {
    if (events[i]!.seq !== expectedStartSeq + i) return i
  }
  return -1
}

/**
 * Verify prev_hash chain threading within this batch.
 * `incomingPrevHash` is the expected prev_hash of the first event
 * (null for a fresh session, or the last stored event_hash for appends).
 * Returns the index of the first broken link, or -1 if intact.
 */
export function findChainBreak(
  events: OcelEventCandidate[],
  incomingPrevHash: string | null,
): number {
  let expected = incomingPrevHash
  for (let i = 0; i < events.length; i++) {
    if (events[i]!.prev_hash !== expected) return i
    expected = events[i]!.event_hash
  }
  return -1
}

// ── Full batch admission gate ────────────────────────────────────────────────

export interface BatchAdmissionInput {
  session_id: string
  events: OcelEventCandidate[]
  /** Expected seq of first event (0 for new session, or max_seq+1 for append). */
  expectedStartSeq?: number
  /** Expected prev_hash of first event (null for seq=0). */
  incomingPrevHash?: string | null
}

/**
 * Run all admission gates on a batch.
 * Returns admit if all pass, reject with reason if any fail.
 */
export function admitBatch(input: BatchAdmissionInput): AdmissionResult {
  const { session_id, events, expectedStartSeq = 0, incomingPrevHash = null } = input

  if (!session_id || typeof session_id !== 'string') {
    return { verdict: 'reject', statusCode: 400, reason: 'session_id is required' }
  }

  if (!Array.isArray(events) || events.length === 0) {
    return { verdict: 'reject', statusCode: 400, reason: 'events[] must be non-empty' }
  }

  // Gate 1: required fields on every event
  for (let i = 0; i < events.length; i++) {
    if (!validateRequiredFields(events[i]!)) {
      return {
        verdict: 'reject',
        statusCode: 400,
        reason: `events[${i}] missing required fields (activity, timestamp_ms, seq, event_hash must be 64-char hex)`,
        offending_index: i,
      }
    }
  }

  // Gate 2: seq contiguity
  const gapIdx = findSeqGap(events, expectedStartSeq)
  if (gapIdx !== -1) {
    return {
      verdict: 'reject',
      statusCode: 422,
      reason: `seq gap at index ${gapIdx}: expected ${expectedStartSeq + gapIdx}, got ${events[gapIdx]!.seq}`,
      offending_index: gapIdx,
    }
  }

  // Gate 3: prev_hash chain threading
  const breakIdx = findChainBreak(events, incomingPrevHash)
  if (breakIdx !== -1) {
    return {
      verdict: 'quarantine',
      statusCode: 422,
      reason: `prev_hash mismatch at index ${breakIdx} (seq=${events[breakIdx]!.seq}): expected ${incomingPrevHash ?? 'null'}, got ${events[breakIdx]!.prev_hash}`,
      offending_index: breakIdx,
    }
  }

  // Gate 4: session_id consistency (if provided per-event)
  for (let i = 0; i < events.length; i++) {
    const evtSid = events[i]!.session_id
    if (evtSid !== undefined && evtSid !== session_id) {
      return {
        verdict: 'reject',
        statusCode: 422,
        reason: `events[${i}].session_id (${evtSid}) does not match batch session_id (${session_id})`,
        offending_index: i,
      }
    }
  }

  return { verdict: 'admit' }
}
