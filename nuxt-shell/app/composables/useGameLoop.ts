/**
 * useGameLoop — headless end-to-end game session lifecycle composable.
 *
 * Chains the full server-side proof arc without requiring UE4 or a browser:
 *   startSession → ingestEvents → finalizeSession → pollSessionState('Proven')
 *
 * Callable from tests (headless-loop.test.ts) and from Vue components alike.
 *
 * Van der Aalst doctrine: the loop is PROVEN only when OCEL event evidence
 * has been mined into a conforming receipt — not when API calls returned 200.
 */

import { blake3 } from '@noble/hashes/blake3.js';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface OcelEvent {
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  event_hash: string;
  prev_hash: string | null;
  seq: number;
  session_id: string;
}

export interface SessionSeedResult {
  session_id: string;
  receipt_hash: string;
  chain_tip: string;
  receipt_id?: string;
  ocel_event_count?: number;
}

export interface IngestResult {
  ingested: number;
  session_id: string;
  trace_id: string | null;
}

export interface FinalizeResult {
  verdict: 'PROVEN' | 'CHAIN_BROKEN' | 'HASH_MISMATCH' | 'NO_EVENTS';
  proven_at: string | null;
  chain_verified: boolean;
  chain_tip_matches_hash: boolean;
}

export interface SessionState {
  session_id: string;
  state: 'Created' | 'Active' | 'Closed' | 'Proven' | 'NOT_FOUND';
  is_alive: boolean | null;
  receipt_hash: string | null;
  ocel_event_count: number;
  proven_at: string | null;
}

export interface FullLoopReceipt {
  session_id: string;
  receipt_hash: string;
  chain_tip: string;
  verdict: FinalizeResult['verdict'];
  proven_at: string | null;
  ingested: number;
  final_state: SessionState['state'];
  conformance?: {
    chain_verified: boolean;
    chain_tip_matches_hash: boolean;
  };
}

// ── BLAKE3 helpers ────────────────────────────────────────────────────────────

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function canonicalize(obj: Record<string, unknown>): string {
  return JSON.stringify(obj, Object.keys(obj).sort());
}

function computeEventHash(
  sessionId: string,
  activity: string,
  timestampMs: number,
  prevHash: string | null,
  attributes: Record<string, unknown>,
): string {
  return blake3Hex(canonicalize({ session_id: sessionId, activity, timestamp_ms: timestampMs, prev_hash: prevHash, attributes }));
}

// ── Composable ────────────────────────────────────────────────────────────────

export function useGameLoop(baseUrl = '') {
  const LAWFUL_ACTIVITIES = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] as const;

  // ── Step 1: start a headless session via session-seed ─────────────────────

  async function startSession(opts: { idempotency_key?: string; create_test_player?: boolean } = {}): Promise<SessionSeedResult> {
    const res = await fetch(`${baseUrl}/api/game/session-seed`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(opts),
    });
    if (!res.ok) {
      const msg = await res.text().catch(() => res.statusText);
      throw new Error(`session-seed failed (${res.status}): ${msg}`);
    }
    return res.json() as Promise<SessionSeedResult>;
  }

  // ── Step 2: compute BLAKE3 chain and ingest OCEL events ──────────────────

  async function ingestEvents(
    sessionId: string,
    events: Omit<OcelEvent, 'event_hash' | 'prev_hash' | 'seq' | 'session_id'>[],
  ): Promise<IngestResult> {
    let prevHash: string | null = null;
    const chained: OcelEvent[] = events.map((ev, seq) => {
      const event_hash = computeEventHash(sessionId, ev.activity, ev.timestamp_ms, prevHash, ev.attributes);
      const row: OcelEvent = { ...ev, session_id: sessionId, seq, event_hash, prev_hash: prevHash };
      prevHash = event_hash;
      return row;
    });

    const res = await fetch(`${baseUrl}/api/game/ocel-ingest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id: sessionId, events: chained }),
    });
    if (!res.ok) {
      const msg = await res.text().catch(() => res.statusText);
      throw new Error(`ocel-ingest failed (${res.status}): ${msg}`);
    }
    return res.json() as Promise<IngestResult>;
  }

  // ── Step 3: finalize and prove the session receipt ────────────────────────

  async function finalizeSession(sessionId: string, receiptHash: string): Promise<FinalizeResult> {
    const res = await fetch(`${baseUrl}/api/game/receipt-finalize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id: sessionId, receipt_hash: receiptHash, update_receipt: true }),
    });
    if (!res.ok) {
      const msg = await res.text().catch(() => res.statusText);
      throw new Error(`receipt-finalize failed (${res.status}): ${msg}`);
    }
    return res.json() as Promise<FinalizeResult>;
  }

  // ── Step 4: poll session state until target or timeout ───────────────────

  async function pollSessionState(
    sessionId: string,
    targetState: 'Proven',
    timeoutMs = 10_000,
  ): Promise<SessionState> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const res = await fetch(`${baseUrl}/api/game/session-state?session_id=${encodeURIComponent(sessionId)}`);
      if (res.ok) {
        const state = await res.json() as SessionState;
        if (state.state === targetState) return state;
      }
      await new Promise(r => setTimeout(r, 500));
    }
    throw new Error(`pollSessionState: timed out after ${timeoutMs}ms waiting for state=${targetState} on session ${sessionId}`);
  }

  // ── Full loop: chains all steps with 3 lawful events ─────────────────────

  async function runFullLoop(
    opts: { idempotency_key?: string; create_test_player?: boolean } = {},
  ): Promise<FullLoopReceipt> {
    const seed = await startSession(opts);
    const baseMs = Date.now();

    const rawEvents = LAWFUL_ACTIVITIES.map((activity, i) => ({
      activity,
      timestamp_ms: baseMs + i * 200,
      object_refs: [seed.session_id],
      attributes: { source: 'useGameLoop', seq: i } as Record<string, unknown>,
    }));

    const ingest = await ingestEvents(seed.session_id, rawEvents);
    const finalize = await finalizeSession(seed.session_id, seed.receipt_hash);
    const finalState = await pollSessionState(seed.session_id, 'Proven');

    return {
      session_id: seed.session_id,
      receipt_hash: seed.receipt_hash,
      chain_tip: seed.chain_tip,
      verdict: finalize.verdict,
      proven_at: finalize.proven_at,
      ingested: ingest.ingested,
      final_state: finalState.state,
      conformance: {
        chain_verified: finalize.chain_verified,
        chain_tip_matches_hash: finalize.chain_tip_matches_hash,
      },
    };
  }

  return {
    startSession,
    ingestEvents,
    finalizeSession,
    pollSessionState,
    runFullLoop,
  };
}
