/**
 * useGameSessionPersistence — persists OCEL events to Supabase with hash chaining.
 *
 * Pattern: ~/dashboard.bak/supabase/migrations/002_events_raw.sql (tamper-evident hash chain)
 * Each event's hash = SHA-256(prev_hash || activity || timestamp_ms || JSON(attributes))
 * This makes the event log immutable and verifiable by pm4py conformance checking.
 *
 * The composable watches useGameSessionOcel's event array and syncs new events to
 * the game_sessions + ocel_events Supabase tables in real time.
 */


async function sha256Hex(data: string): Promise<string> {
  const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(data));
  return Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2, '0')).join('');
}

export function useGameSessionPersistence() {
  const { client } = useRocketSupabase();
  const { events, sessionId, isPlaying } = useGameSessionOcel();

  const dbSessionId = ref<string | null>(null);
  const lastHash = ref<string | null>(null);
  const syncedCount = ref(0);
  const syncError = ref<string | null>(null);
  const isSyncing = ref(false);

  // Open a DB session row when the game session starts
  watch(sessionId, async (sid) => {
    if (!sid) return;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data, error } = await (client as any)
      .from('game_sessions')
      .insert({
        player_id: null,
        session_started_at: new Date().toISOString(),
        session_ended_at: null,
        engine_source: 'unknown',
        is_alive: false,
        ocel_event_count: 0,
        receipt_hash: null,
        metadata: { browser_session_id: sid },
      })
      .select('id')
      .single();

    if (error) {
      syncError.value = `Failed to create DB session: ${error.message}`;
      return;
    }
    dbSessionId.value = data.id;
    lastHash.value = null;
    syncedCount.value = 0;
  });

  // Sync new OCEL events to Supabase as they arrive
  watch(events, async (all) => {
    if (!dbSessionId.value || isSyncing.value) return;
    const unsync = all.slice(syncedCount.value);
    if (unsync.length === 0) return;

    isSyncing.value = true;
    try {
      for (const evt of unsync) {
        const seq = syncedCount.value + 1;
        const raw = `${lastHash.value ?? ''}|${evt.activity}|${evt.timestamp_ms}|${JSON.stringify(evt.attributes)}`;
        const hash = await sha256Hex(raw);

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const { error } = await (client as any).from('ocel_events').insert({
          session_id: dbSessionId.value,
          activity: evt.activity,
          timestamp_ms: evt.timestamp_ms,
          object_refs: [...evt.object_refs.map(o => o.object_id)],
          attributes: evt.attributes as Record<string, unknown>,
          prev_hash: lastHash.value,
          event_hash: hash,
          seq,
        });
        if (error) {
          syncError.value = `ocel_events insert failed: ${error.message}`;
          break;
        }
        lastHash.value = hash;
        syncedCount.value = seq;
      }

      // Update session alive status and count
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await (client as any)
        .from('game_sessions')
        .update({ is_alive: isPlaying.value, ocel_event_count: syncedCount.value })
        .eq('id', dbSessionId.value);

      // Fire-and-forget: emit OTel spans via server route (non-fatal if collector down)
      const batch = unsync.map((evt, i) => ({
        session_id: dbSessionId.value!,
        activity: evt.activity,
        timestamp_ms: evt.timestamp_ms,
        object_refs: evt.object_refs.map(o => o.object_id),
        attributes: evt.attributes as Record<string, unknown>,
        event_hash: lastHash.value ?? '',
        seq: syncedCount.value - unsync.length + i + 1,
      }));
      $fetch('/api/game/ocel-ingest', { method: 'POST', body: { session_id: dbSessionId.value, events: batch } })
        .catch(() => { /* collector down — Supabase is source of truth */ });
    } finally {
      isSyncing.value = false;
    }
  }, { deep: true });

  // On session end, mark the DB row closed
  async function closeSession(receiptHash?: string) {
    if (!dbSessionId.value) return;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    await (client as any)
      .from('game_sessions')
      .update({
        session_ended_at: new Date().toISOString(),
        is_alive: false,
        ocel_event_count: syncedCount.value,
        receipt_hash: receiptHash ?? null,
      })
      .eq('id', dbSessionId.value);
  }

  // Persist a verified receipt to Supabase
  async function persistReceipt(receipt: {
    verdict: 'PASS' | 'FAIL' | 'PENDING';
    milestone: string;
    ocelLifecycle: string[];
    engineSource: string;
    receiptHash: string;
    payload: Record<string, unknown>;
  }) {
    if (!dbSessionId.value) return null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data, error } = await (client as any)
      .from('game_receipts')
      .insert({
        session_id: dbSessionId.value,
        verdict: receipt.verdict,
        milestone: receipt.milestone,
        ocel_event_count: syncedCount.value,
        ocel_lifecycle: receipt.ocelLifecycle,
        engine_source: receipt.engineSource,
        receipt_hash: receipt.receiptHash,
        proven_at: new Date().toISOString(),
        payload: receipt.payload,
      })
      .select('id')
      .single();
    if (error) {
      syncError.value = `Receipt persist failed: ${error.message}`;
      return null;
    }
    await closeSession(receipt.receiptHash);
    return data.id;
  }

  return {
    dbSessionId: readonly(dbSessionId),
    syncedCount: readonly(syncedCount),
    lastHash: readonly(lastHash),
    syncError: readonly(syncError),
    isSyncing: readonly(isSyncing),
    closeSession,
    persistReceipt,
  };
}
