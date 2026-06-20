/**
 * useNetworkStatus
 *
 * Tracks Supabase reachability beyond navigator.onLine.
 * "Online (OS)" ≠ "Supabase is reachable" — this composable probes the real
 * Supabase REST endpoint to distinguish network-down from Supabase-down.
 *
 * State machine:
 *   idle → offline → reconnecting → connected → idle
 *
 * Usage:
 *   const { status, isOffline, reconnect } = useNetworkStatus()
 */

export type NetworkStatus = 'idle' | 'offline' | 'reconnecting' | 'connected';

const PROBE_INTERVAL_MS = 15_000;
const RECONNECT_DELAY_MS = 1_200;

export function useNetworkStatus() {
  const status = ref<NetworkStatus>('idle');
  const lastChecked = ref<number>(0);

  const { client } = useRocketSupabase();

  async function probe(): Promise<boolean> {
    try {
      // Ping the Supabase health endpoint via a lightweight query
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const { error } = await (client as any).from('game_sessions').select('id').limit(1);
      return !error;
    } catch {
      return false;
    }
  }

  async function reconnect() {
    if (status.value === 'reconnecting') return;
    status.value = 'reconnecting';
    await new Promise(r => setTimeout(r, RECONNECT_DELAY_MS));
    const ok = await probe();
    if (ok) {
      status.value = 'connected';
      setTimeout(() => { if (status.value === 'connected') status.value = 'idle'; }, 1_500);
    } else {
      status.value = 'offline';
    }
  }

  async function checkOnce() {
    if (status.value === 'reconnecting') return;
    lastChecked.value = Date.now();
    const ok = await probe();
    if (!ok && status.value === 'idle') {
      status.value = 'offline';
    } else if (ok && status.value === 'offline') {
      await reconnect();
    }
  }

  let interval: ReturnType<typeof setInterval> | null = null;

  onMounted(() => {
    // Browser online/offline events for fast detection
    window.addEventListener('offline', () => { status.value = 'offline'; });
    window.addEventListener('online', () => reconnect());

    // Periodic probe to catch Supabase-specific outages
    interval = setInterval(checkOnce, PROBE_INTERVAL_MS);

    // Initial probe after a short delay so the page load doesn't block
    setTimeout(checkOnce, 2_000);
  });

  onUnmounted(() => {
    if (interval) clearInterval(interval);
    window.removeEventListener('offline', () => { status.value = 'offline'; });
    window.removeEventListener('online', () => reconnect());
  });

  return {
    status: readonly(status),
    isOffline: computed(() => status.value === 'offline' || status.value === 'reconnecting'),
    lastChecked: readonly(lastChecked),
    reconnect,
  };
}
