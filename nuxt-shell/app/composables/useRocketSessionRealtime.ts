/**
 * useRocketSessionRealtime — Supabase Realtime subscription for game receipts and sessions.
 *
 * Ported from ~/seth/neako-web/backup/composables/useRealtimeCollaborations.ts
 * Pattern: postgres_changes subscription with hydration and VueUse event bus.
 *
 * Subscribes to:
 *   - INSERT on `game_receipts` → fires 'receipt:new' event bus
 *   - UPDATE on `game_sessions` → fires 'session:updated' event bus
 *
 * Used by: leaderboard.vue (live rank updates), receipts.vue (auto-refresh).
 *
 * The leaderboard Postgres trigger fires automatically on PASS receipts,
 * so subscribing to game_receipts is the minimal hook to drive all live UI.
 */

import { useEventBus } from '@vueuse/core';

export interface LiveReceipt {
  id: string;
  session_id: string | null;
  verdict: 'PASS' | 'FAIL' | 'PENDING';
  milestone: string;
  ocel_event_count: number;
  engine_source: string;
  proven_at: string;
}

export interface LiveSession {
  id: string;
  player_id: string | null;
  is_alive: boolean;
  ocel_event_count: number;
  engine_source: string;
}

const RECEIPT_BUS_KEY = 'rocket:receipt:new';
const SESSION_BUS_KEY = 'rocket:session:updated';

export function useRocketSessionRealtime() {
  const { client } = useRocketSupabase();
  const receiptBus = useEventBus<LiveReceipt>(RECEIPT_BUS_KEY);
  const sessionBus = useEventBus<LiveSession>(SESSION_BUS_KEY);

  // Latest receipts and live sessions — populated by the subscription
  const latestReceipt = ref<LiveReceipt | null>(null);
  const liveSessions = ref<LiveSession[]>([]);
  const isSubscribed = ref(false);
  const channelRef = shallowRef<ReturnType<typeof client.channel> | null>(null);

  function hydrateReceipt(row: Record<string, unknown>): LiveReceipt | null {
    if (!row?.id || !row?.verdict) return null;
    return {
      id: String(row.id),
      session_id: row.session_id ? String(row.session_id) : null,
      verdict: row.verdict as 'PASS' | 'FAIL' | 'PENDING',
      milestone: String(row.milestone ?? ''),
      ocel_event_count: Number(row.ocel_event_count ?? 0),
      engine_source: String(row.engine_source ?? 'unknown'),
      proven_at: String(row.proven_at ?? new Date().toISOString()),
    };
  }

  function hydrateSession(row: Record<string, unknown>): LiveSession | null {
    if (!row?.id) return null;
    return {
      id: String(row.id),
      player_id: row.player_id ? String(row.player_id) : null,
      is_alive: Boolean(row.is_alive),
      ocel_event_count: Number(row.ocel_event_count ?? 0),
      engine_source: String(row.engine_source ?? 'unknown'),
    };
  }

  function subscribe() {
    if (isSubscribed.value) return;

    const channel = client
      .channel('rocket:game-events')
      .on(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        'postgres_changes' as any,
        { event: 'INSERT', schema: 'public', table: 'game_receipts' },
        (payload: { new: Record<string, unknown> }) => {
          const receipt = hydrateReceipt(payload.new);
          if (!receipt) return;
          latestReceipt.value = receipt;
          receiptBus.emit(receipt);
        },
      )
      .on(
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        'postgres_changes' as any,
        { event: 'UPDATE', schema: 'public', table: 'game_sessions' },
        (payload: { new: Record<string, unknown> }) => {
          const session = hydrateSession(payload.new);
          if (!session) return;
          // Update or insert into liveSessions
          const idx = liveSessions.value.findIndex(s => s.id === session.id);
          if (idx >= 0) {
            liveSessions.value = liveSessions.value.map((s, i) => i === idx ? session : s);
          } else {
            liveSessions.value = [session, ...liveSessions.value].slice(0, 20);
          }
          sessionBus.emit(session);
        },
      )
      .subscribe((status: string) => {
        isSubscribed.value = status === 'SUBSCRIBED';
      });

    channelRef.value = channel;
    isSubscribed.value = true;
  }

  function unsubscribe() {
    if (channelRef.value) {
      client.removeChannel(channelRef.value);
      channelRef.value = null;
      isSubscribed.value = false;
    }
  }

  // Auto-subscribe on mount, unsubscribe on unmount
  onMounted(subscribe);
  onUnmounted(unsubscribe);

  return {
    latestReceipt: readonly(latestReceipt),
    liveSessions: readonly(liveSessions),
    isSubscribed: readonly(isSubscribed),
    receiptBus,
    sessionBus,
    subscribe,
    unsubscribe,
  };
}
