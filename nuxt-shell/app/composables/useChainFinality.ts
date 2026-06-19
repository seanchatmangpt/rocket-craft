/**
 * useChainFinality — per-receipt OCEL chain finality proof composable.
 *
 * Calls /api/game/receipt-finalize for a given (session_id, receipt_hash) pair
 * and exposes the PROVEN/CHAIN_BROKEN/HASH_MISMATCH/NO_EVENTS verdict reactively.
 *
 * Used by receipts.vue (inline per-row proof) and ReceiptDrawer.vue (full-page).
 *
 * Van der Aalst doctrine: a PASS verdict on game_receipts is necessary but not
 * sufficient — chain finality proves the OCEL event log is causally consistent
 * and was not modified after emit.
 */

export type ChainVerdict = 'PROVEN' | 'CHAIN_BROKEN' | 'HASH_MISMATCH' | 'NO_EVENTS' | 'PENDING' | 'ERROR';

export interface ChainFinalityResult {
  verdict: ChainVerdict;
  chain_verified: boolean;
  chain_tip_matches_hash: boolean;
  broken_at: number | null;
}

/**
 * Map of receipt_id → finality result (lazy: only loaded when `prove(id, ...)` is called).
 * Shared across all component instances that import this composable.
 */
const _finality = ref<Record<string, ChainFinalityResult>>({});
const _loading = ref<Record<string, boolean>>({});

export function useChainFinality() {
  const finality = readonly(_finality);
  const loading = readonly(_loading);

  /**
   * Request chain-finality proof for a receipt.
   * Idempotent — subsequent calls with the same id return the cached verdict.
   */
  async function prove(
    receiptId: string,
    sessionId: string | null,
    receiptHash: string,
  ): Promise<ChainFinalityResult | null> {
    if (_finality.value[receiptId]) return _finality.value[receiptId];
    if (!sessionId) {
      _finality.value[receiptId] = {
        verdict: 'NO_EVENTS',
        chain_verified: false,
        chain_tip_matches_hash: false,
        broken_at: null,
      };
      return _finality.value[receiptId];
    }

    _loading.value[receiptId] = true;
    try {
      const result = await $fetch<ChainFinalityResult>('/api/game/receipt-finalize', {
        method: 'POST',
        body: { session_id: sessionId, receipt_hash: receiptHash },
      });
      _finality.value[receiptId] = result;
      return result;
    } catch {
      _finality.value[receiptId] = {
        verdict: 'ERROR',
        chain_verified: false,
        chain_tip_matches_hash: false,
        broken_at: null,
      };
      return _finality.value[receiptId];
    } finally {
      _loading.value[receiptId] = false;
    }
  }

  /** Prove all receipts in a batch (parallel, max 5 concurrent). */
  async function proveAll(
    receipts: Array<{ id: string; session_id: string | null; receipt_hash: string }>,
  ) {
    // Chunk to 5-at-a-time to avoid flooding the server
    const CHUNK = 5;
    for (let i = 0; i < receipts.length; i += CHUNK) {
      await Promise.all(
        receipts.slice(i, i + CHUNK).map(r => prove(r.id, r.session_id, r.receipt_hash)),
      );
    }
  }

  return { finality, loading, prove, proveAll };
}
