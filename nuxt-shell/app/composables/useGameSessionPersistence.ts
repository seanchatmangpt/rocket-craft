/**
 * useGameSessionPersistence — persists OCEL events to Supabase with hash chaining.
 *
 * All writes go through server-side API routes (service role key), never via the
 * anon client from the browser. Attack surface summary:
 *   game_sessions  → POST /api/game/session, PATCH /api/game/session/[id]
 *   ocel_events    → POST /api/game/ocel-ingest (batched, server-side chain)
 *   game_receipts  → POST /api/game/cook-receipt (4 proof gates)
 *
 * Hash formula: BLAKE3 of canonical JSON {id, timestamp, type, data, prev_hash}
 * — same formula as exportHashedOcelLog, so chain_tip == last stored event_hash.
 * seq is 0-indexed: first event is seq=0.
 */

export function useGameSessionPersistence() {
  const { events, sessionId, isPlaying } = useGameSessionOcel();

  const { computeEventHash } = useHashChain();
  const dbSessionId = ref<string | null>(null);
  const lastHash = ref<string | null>(null);
  const syncedCount = ref(0);
  const syncError = ref<string | null>(null);
  const isSyncing = ref(false);

  // Open a DB session row via server API when the game session starts
  watch(sessionId, async (sid) => {
    if (!sid) return;
    try {
      const result = await $fetch<{ session_id: string; started_at: string }>('/api/game/session', {
        method: 'POST',
        body: { browser_session_id: sid, engine_source: 'browser' },
      });
      dbSessionId.value = result.session_id;
      lastHash.value = null;
      syncedCount.value = 0;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      syncError.value = `Failed to create DB session: ${msg}`;
    }
  });

  // Sync new OCEL events to the server as they arrive (batched via ocel-ingest)
  watch(events, async (all) => {
    if (!dbSessionId.value || isSyncing.value) return;
    const unsync = all.slice(syncedCount.value);
    if (unsync.length === 0) return;

    isSyncing.value = true;
    try {
      // Compute BLAKE3 hashes client-side to maintain chain_tip locally
      const batch: Array<{
        session_id: string;
        activity: string;
        timestamp_ms: number;
        object_refs: string[];
        attributes: Record<string, unknown>;
        event_hash: string;
        prev_hash: string | null;
        seq: number;
      }> = [];

      for (const evt of unsync) {
        const seq = syncedCount.value;
        const hash = await computeEventHash({
          id: evt.id,
          timestamp: new Date(evt.timestamp_ms).toISOString(),
          type: evt.activity,
          data: {
            object_refs: evt.object_refs as unknown as Record<string, unknown>,
            attributes: evt.attributes as Record<string, unknown>,
          },
          prev_hash: lastHash.value,
        });
        batch.push({
          session_id: dbSessionId.value!,
          activity: evt.activity,
          timestamp_ms: evt.timestamp_ms,
          object_refs: evt.object_refs.map(o => o.object_id),
          attributes: evt.attributes as Record<string, unknown>,
          event_hash: hash,
          prev_hash: lastHash.value,
          seq,
        });
        lastHash.value = hash;
        syncedCount.value = seq + 1;
      }

      // Ingest via server route (handles ocel_events insert + continuing chain)
      await $fetch('/api/game/ocel-ingest', {
        method: 'POST',
        body: { session_id: dbSessionId.value, events: batch },
      }).catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        syncError.value = `ocel-ingest failed: ${msg}`;
      });

      // Update session alive status and count via PATCH
      await $fetch(`/api/game/session/${dbSessionId.value}`, {
        method: 'PATCH',
        body: { is_alive: isPlaying.value, ocel_event_count: syncedCount.value },
      }).catch(() => { /* non-fatal — count corrected on close */ });

    } finally {
      isSyncing.value = false;
    }
  }, { deep: true });

  // On session end, mark the DB row closed via PATCH
  async function closeSession(receiptHash?: string) {
    if (!dbSessionId.value) return;
    await $fetch(`/api/game/session/${dbSessionId.value}`, {
      method: 'PATCH',
      body: {
        session_ended_at: new Date().toISOString(),
        is_alive: false,
        ocel_event_count: syncedCount.value,
        receipt_hash: receiptHash ?? null,
      },
    }).catch(() => { /* non-fatal — session row still valid without end timestamp */ });
  }

  // Persist a verified receipt — routes through the server proof gate (cook-receipt.post.ts).
  // The proof gate is the only write path for game_receipts: it enforces engine_source check,
  // lifecycle completeness, receipt_hash format, and optional Ed25519 signature.
  async function persistReceipt(receipt: {
    verdict: 'PASS' | 'FAIL' | 'PENDING';
    milestone: string;
    ocelLifecycle: string[];
    engineSource: string;
    receiptHash: string;
    payload: Record<string, unknown>;
  }) {
    if (!dbSessionId.value) return null;

    let receiptId: string | null = null;
    try {
      const result = await $fetch<{ receipt_id: string; verdict: string; chain_verified: boolean }>('/api/game/cook-receipt', {
        method: 'POST',
        body: {
          session_id: dbSessionId.value,
          verdict: receipt.verdict,
          milestone: receipt.milestone,
          ocel_event_count: syncedCount.value,
          ocel_lifecycle: receipt.ocelLifecycle,
          engine_source: receipt.engineSource,
          receipt_hash: receipt.receiptHash,
          proven_at: new Date().toISOString(),
          payload: receipt.payload,
        },
      });
      receiptId = result.receipt_id;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      syncError.value = `Receipt proof gate rejected: ${msg}`;
      return null;
    }

    await closeSession(receipt.receiptHash);
    return receiptId;
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
