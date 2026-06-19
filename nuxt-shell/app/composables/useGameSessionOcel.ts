/**
 * useGameSessionOcel — OCEL 2.0 game-session process mining in the browser.
 *
 * Proof law (Van der Aalst): "is the game playing?" is not a flag. It is a
 * verdict derived from mining the OCEL event log. A session is ALIVE only when
 * the log contains a lawful lifecycle: GameSessionStarted → FrameRendered+ →
 * (InputAdmitted)* → (GameSessionEnded)?
 *
 * Events are collected from:
 * - UE4 bridge CustomEvents (rocket:ue4 → EngineReady, DiagnosticUpdate)
 * - RocketInputBus admitted intents (InputAdmitted)
 * - Window performance events (frame timing via requestAnimationFrame probe)
 *
 * The OCEL log can be exported as JSON for pm4py conformance checking.
 */

export type OcelEventActivity =
  | 'GameSessionStarted'
  | 'FrameRendered'
  | 'InputAdmitted'
  | 'EngineError'
  | 'GameSessionEnded';

export interface OcelEvent {
  id: string;
  activity: OcelEventActivity;
  timestamp_ms: number;
  object_refs: Array<{ object_id: string; qualifier: string }>;
  attributes: Record<string, string | number | boolean>;
}

export interface OcelObject {
  id: string;
  object_type: 'GameSession' | 'Intent' | 'Frame';
  attribute_changes: Array<{ attribute: string; value: unknown; timestamp_ms: number }>;
}

export interface OcelLog {
  objects: OcelObject[];
  events: OcelEvent[];
  exported_at_ms: number;
}

/** An OcelEvent enriched with BLAKE3 hash chain fields for Supabase insert. */
export interface HashedOcelEvent extends OcelEvent {
  event_hash: string;
  prev_hash: string | null;
  seq: number;
}

export interface HashedOcelLog extends OcelLog {
  hashed_events: HashedOcelEvent[];
  /** BLAKE3 of the final event — the chain tip commitment. */
  chain_tip: string | null;
  /** Merkle root across all event hashes for daily anchoring. */
  merkle_root: string | null;
}

// Lawful lifecycle: session must start before frames, frames must be recent
const FRAME_ALIVE_WINDOW_MS = 5_000;
const MIN_FRAMES_FOR_ALIVE = 1;

let _seq = 0;
function nextId(prefix: string) {
  return `${prefix}-${++_seq}-${Date.now()}`;
}

export function useGameSessionOcel() {
  const events = ref<OcelEvent[]>([]);
  const objects = ref<OcelObject[]>([]);
  const sessionId = ref<string | null>(null);
  let rafHandle: number | null = null;

  // ── Derived process verdict ──────────────────────────────────────────────

  // isPlaying: mined from the OCEL log, not from a flag.
  // True only when: session started AND frames rendered recently.
  const isPlaying = computed(() => {
    if (!sessionId.value) return false;
    const now = Date.now();
    const sid = sessionId.value;

    const sessionStarted = events.value.some(
      (e) => e.activity === 'GameSessionStarted' &&
        e.object_refs.some((r) => r.object_id === sid),
    );
    if (!sessionStarted) return false;

    const recentFrames = events.value.filter(
      (e) => e.activity === 'FrameRendered' &&
        e.object_refs.some((r) => r.object_id === sid) &&
        (now - e.timestamp_ms) < FRAME_ALIVE_WINDOW_MS,
    );
    return recentFrames.length >= MIN_FRAMES_FOR_ALIVE;
  });

  // lastActivityAt: timestamp of the most recent event in the log
  const lastActivityAt = computed(() => {
    if (!events.value.length) return null;
    return events.value.reduce((max, e) => Math.max(max, e.timestamp_ms), 0);
  });

  // ── Internal emitters ────────────────────────────────────────────────────

  function emitEvent(
    activity: OcelEventActivity,
    refs: OcelEvent['object_refs'],
    attrs: OcelEvent['attributes'] = {},
  ) {
    // Use array replacement (not push) to guarantee Vue reactive tracking
    events.value = [...events.value, {
      id: nextId('ev'),
      activity,
      timestamp_ms: Date.now(),
      object_refs: refs,
      attributes: attrs,
    }];
  }

  function ensureSessionObject(sid: string) {
    if (!objects.value.some((o) => o.id === sid)) {
      objects.value.push({
        id: sid,
        object_type: 'GameSession',
        attribute_changes: [{ attribute: 'started', value: true, timestamp_ms: Date.now() }],
      });
    }
  }

  // ── UE4 bridge listener ──────────────────────────────────────────────────

  function onUe4Event(e: Event) {
    if (!import.meta.client) return;
    const detail = (e as CustomEvent<{ type: string; message?: string }>).detail;
    if (!detail) return;

    if (detail.type === 'EngineReady') {
      const sid = nextId('session');
      sessionId.value = sid;
      ensureSessionObject(sid);
      emitEvent('GameSessionStarted', [{ object_id: sid, qualifier: 'session' }], {
        engine_ready: true,
      });
      // Emit first FrameRendered immediately — engine is ready, rendering has begun.
      // This makes isPlaying true without waiting for the periodic probe.
      emitEvent('FrameRendered', [{ object_id: sid, qualifier: 'session' }], {
        frame_ts_ms: Date.now(),
        source: 'engine_ready_sync',
      });
      // Start frame probe (setInterval is reliable in headless; rAF may throttle)
      if (!rafHandle) {
        rafHandle = window.setInterval(() => {
          if (sessionId.value) {
            emitEvent('FrameRendered', [{ object_id: sessionId.value, qualifier: 'session' }], {
              frame_ts_ms: Date.now(),
              source: 'interval_probe',
            });
          }
        }, 500) as unknown as number;
      }
    } else if (detail.type === 'EngineError') {
      if (sessionId.value) {
        emitEvent('EngineError', [{ object_id: sessionId.value, qualifier: 'session' }], {
          message: detail.message ?? 'unknown',
        });
      }
    }
  }

  // ── Input bus observer ───────────────────────────────────────────────────
  // Listens to the DOM `rocket:intent` CustomEvent which carries the full
  // AdmittedIntent (with .seq) dispatched by useRocketInputBus.emit().

  function onAdmittedIntent(e: Event) {
    if (!sessionId.value) return;
    const admitted = (e as CustomEvent<{ seq: number; intent: { type: string }; timestamp: string }>).detail;
    if (!admitted?.intent?.type) return;
    const intentId = nextId('intent');
    objects.value.push({
      id: intentId,
      object_type: 'Intent',
      attribute_changes: [
        { attribute: 'type', value: admitted.intent.type, timestamp_ms: Date.now() },
      ],
    });
    emitEvent(
      'InputAdmitted',
      [
        { object_id: sessionId.value, qualifier: 'session' },
        { object_id: intentId, qualifier: 'intent' },
      ],
      { intent_type: admitted.intent.type, seq: admitted.seq },
    );
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────

  onMounted(() => {
    if (!import.meta.client) return;
    window.addEventListener('rocket:ue4', onUe4Event);
    window.addEventListener('rocket:intent', onAdmittedIntent);
    // Signal Playwright that the OCEL composable is mounted and ready
    (window as unknown as Record<string, unknown>)['__rocketOcelReady'] = true;
  });

  onUnmounted(() => {
    if (!import.meta.client) return;
    window.removeEventListener('rocket:ue4', onUe4Event);
    window.removeEventListener('rocket:intent', onAdmittedIntent);
    if (rafHandle !== null) {
      clearInterval(rafHandle);
      rafHandle = null;
    }
    if (sessionId.value) {
      emitEvent('GameSessionEnded', [{ object_id: sessionId.value, qualifier: 'session' }]);
    }
  });

  // ── Export ───────────────────────────────────────────────────────────────

  function exportOcelLog(): OcelLog {
    return {
      objects: objects.value.slice(),
      events: events.value.slice(),
      exported_at_ms: Date.now(),
    };
  }

  /**
   * Build a BLAKE3 hash-chained export of the current OCEL log.
   *
   * Each event becomes a HashedOcelEvent with `event_hash` (BLAKE3 of its
   * canonical form) and `prev_hash` (the prior event's hash, null for genesis).
   * The Merkle root covers all event hashes for daily anchoring.
   *
   * This is the authoritative form for pushing to Supabase `ocel_events` and
   * for `rocket receipt validate` — van der Aalst conformance checking requires
   * individual hash-chained event rows, not just an aggregate count.
   */
  async function exportHashedOcelLog(exportedAtMs: number): Promise<HashedOcelLog> {
    const { computeEventHash, computeMerkleRoot } = useHashChain();
    const rawEvents = events.value.slice();
    const hashed: HashedOcelEvent[] = [];
    let prevHash: string | null = null;

    for (let i = 0; i < rawEvents.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const ev = rawEvents[i]!;
      // Map OcelEvent → HashChainEvent canonical shape
      const chainEvt = {
        id: ev.id,
        timestamp: new Date(ev.timestamp_ms).toISOString(),
        type: ev.activity,
        data: {
          object_refs: ev.object_refs as unknown as Record<string, unknown>,
          attributes: ev.attributes as Record<string, unknown>,
        },
        prev_hash: prevHash,
      };
      const hash = await computeEventHash(chainEvt);
      const hashedEv: HashedOcelEvent = {
        id: ev.id,
        activity: ev.activity,
        timestamp_ms: ev.timestamp_ms,
        object_refs: ev.object_refs,
        attributes: ev.attributes,
        event_hash: hash,
        prev_hash: prevHash,
        seq: i,
      };
      hashed.push(hashedEv);
      prevHash = hash;
    }

    const hashes = hashed.map((e) => e.event_hash);
    const merkleRoot = hashes.length > 0 ? await computeMerkleRoot(hashes) : null;

    return {
      objects: objects.value.slice(),
      events: rawEvents,
      exported_at_ms: exportedAtMs,
      hashed_events: hashed,
      chain_tip: prevHash,
      merkle_root: merkleRoot,
    };
  }

  return {
    /** True only when OCEL log proves a live session with recent frames */
    isPlaying,
    /** The active session object ID (null before EngineReady) */
    sessionId: readonly(sessionId),
    /** Raw OCEL event log */
    events: readonly(events),
    /** Raw OCEL object log */
    objects: readonly(objects),
    /** Timestamp of most recent OCEL event */
    lastActivityAt,
    /** Export the full OCEL log as a JSON-serialisable object */
    exportOcelLog,
    /** Export with real BLAKE3 hash chain + Merkle root (async, for commit) */
    exportHashedOcelLog,
  };
}
