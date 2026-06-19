/**
 * useRocketRealtimeLeaderboard — presence-aware realtime leaderboard composable.
 *
 * Adapted from the useRealtimeData pattern (src/chapter-4/realtime.md).
 *
 * Problem: useRocketSessionRealtime.subscribe() is called via onMounted, so if
 * the component mounts before the Supabase client finishes initialising the
 * channel silently never receives events. This composable fixes that by:
 *
 *   1. Performing an initial fetch immediately (no waiting for a realtime event).
 *   2. Subscribing to postgres_changes on INSERT/UPDATE for the `leaderboard`
 *      view so rows update without polling.
 *   3. Exposing a `status` ref ('connecting'|'live'|'error') driven by the
 *      Supabase channel subscription callback.
 *   4. Auto-retrying on CHANNEL_ERROR via useTimeoutFn (VueUse, auto-imported
 *      by @vueuse/nuxt) with exponential back-off capped at 30 s.
 *
 * Usage (replaces the manual loadLeaderboard + receiptBus pattern):
 *
 *   const { rows, loading, error, status, lastUpdate } = useRocketRealtimeLeaderboard();
 */

import { useEventBus } from '@vueuse/core';
import type { LiveReceipt } from './useRocketSessionRealtime';

// ── Types ────────────────────────────────────────────────────────────────────

export interface LeaderboardRow {
  rank: number | null;
  score: number;
  updated_at: string;
  players: {
    id: string;
    username: string | null;
    high_score: number;
  } | null;
}

export type LeaderboardStatus = 'connecting' | 'live' | 'error';

// ── Constants ────────────────────────────────────────────────────────────────

const RECEIPT_BUS_KEY = 'rocket:receipt:new';
const CHANNEL_NAME = 'rocket:leaderboard-realtime';
const INITIAL_RETRY_MS = 2_000;
const MAX_RETRY_MS = 30_000;

// ── Composable ───────────────────────────────────────────────────────────────

export function useRocketRealtimeLeaderboard() {
  const { client } = useRocketSupabase();

  const rows = ref<LeaderboardRow[]>([]);
  const loading = ref(true);
  const error = ref<string | null>(null);
  const status = ref<LeaderboardStatus>('connecting');
  const lastUpdate = ref<string | null>(null);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let channelRef: ReturnType<typeof client.channel> | null = null;
  let retryDelayMs = INITIAL_RETRY_MS;
  let retryHandle: ReturnType<typeof useTimeoutFn> | null = null;

  // ── Initial fetch ─────────────────────────────────────────────────────────

  async function fetchRows() {
    loading.value = true;
    error.value = null;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data, error: fetchErr } = await (client as any)
      .from('leaderboard')
      .select('rank, score, updated_at, players(id, username, high_score)')
      .order('score', { ascending: false })
      .limit(100);

    loading.value = false;

    if (fetchErr) {
      error.value = (fetchErr as { message: string }).message;
      return;
    }

    rows.value = (data as LeaderboardRow[]) ?? [];
    lastUpdate.value = new Date().toLocaleTimeString();
  }

  // ── Channel subscription ──────────────────────────────────────────────────

  function unsubscribe() {
    if (channelRef) {
      client.removeChannel(channelRef);
      channelRef = null;
    }
    retryHandle?.stop();
    retryHandle = null;
  }

  function subscribe() {
    // Guard: don't double-subscribe
    if (channelRef) return;

    status.value = 'connecting';

    const channel = client
      .channel(CHANNEL_NAME)
      // React to INSERT on the leaderboard view (Postgres trigger already ran)
      .on(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        'postgres_changes' as any,
        { event: 'INSERT', schema: 'public', table: 'leaderboard' },
        (_payload: { new: Record<string, unknown> }) => {
          // Re-fetch the full ranked list — the DB trigger may have re-ranked
          // multiple rows, so a merge of a single new row would be stale.
          void fetchRows();
        },
      )
      .on(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        'postgres_changes' as any,
        { event: 'UPDATE', schema: 'public', table: 'leaderboard' },
        (_payload: { new: Record<string, unknown> }) => {
          void fetchRows();
        },
      )
      .subscribe((subscribeStatus: string) => {
        if (subscribeStatus === 'SUBSCRIBED') {
          status.value = 'live';
          retryDelayMs = INITIAL_RETRY_MS; // reset back-off on success
        } else if (subscribeStatus === 'CHANNEL_ERROR' || subscribeStatus === 'TIMED_OUT') {
          status.value = 'error';
          // Remove the broken channel before retrying
          if (channelRef) {
            client.removeChannel(channelRef);
            channelRef = null;
          }
          // Exponential back-off retry
          retryHandle = useTimeoutFn(() => {
            retryDelayMs = Math.min(retryDelayMs * 2, MAX_RETRY_MS);
            subscribe();
          }, retryDelayMs);
        }
      });

    channelRef = channel;
  }

  // ── Receipt bus bridge ────────────────────────────────────────────────────
  // Re-fetch when a PASS receipt fires (leaderboard trigger has already run by
  // the time the event arrives, so the DB reflects the new rank).

  const receiptBus = useEventBus<LiveReceipt>(RECEIPT_BUS_KEY);
  let receiptUnsub: (() => void) | null = null;

  // ── Lifecycle ─────────────────────────────────────────────────────────────

  onMounted(() => {
    if (!import.meta.client) return;

    // Kick off the initial data fetch immediately — don't wait for a realtime event.
    void fetchRows();

    // Subscribe to realtime postgres_changes
    subscribe();

    // Also re-fetch on any PASS receipt (belt-and-suspenders: covers the case
    // where the leaderboard view doesn't have a direct realtime trigger).
    receiptUnsub = receiptBus.on((receipt) => {
      if (receipt.verdict === 'PASS') void fetchRows();
    });
  });

  onUnmounted(() => {
    if (!import.meta.client) return;
    unsubscribe();
    receiptUnsub?.();
    receiptUnsub = null;
  });

  // ── Public API ─────────────────────────────────────────────────────────────

  return {
    /** Current ranked leaderboard rows (pre-populated by initial fetch). */
    rows: readonly(rows),
    /** True while the initial fetch or a triggered re-fetch is in flight. */
    loading: readonly(loading),
    /** Error message from the most recent failed fetch, or null. */
    error: readonly(error),
    /**
     * Channel subscription status.
     * 'connecting' — waiting for SUBSCRIBED ack from Supabase.
     * 'live'       — channel is SUBSCRIBED and pushing events.
     * 'error'      — CHANNEL_ERROR / TIMED_OUT; retry is scheduled.
     */
    status: readonly(status),
    /** Locale time string of the last successful fetch, or null. */
    lastUpdate: readonly(lastUpdate),
    /** Force a manual re-fetch (useful for pull-to-refresh). */
    refresh: fetchRows,
  };
}
