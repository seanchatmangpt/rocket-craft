/**
 * useHealthLieDetector
 *
 * Active runtime assertion engine for pipeline health invariants.
 * Inspired by the LieDetector pattern from the expo-supabase-ai-template.
 *
 * A "lie" is any condition where the declared pipeline state contradicts
 * observable evidence. Rather than silently accepting contradictions, this
 * composable surfaces them as violations for the dashboard to display.
 *
 * Invariants checked (server-side via /api/game/health-lies):
 *   LIE-1: A receipt claims PASS but has zero OCEL events
 *   LIE-2: A session is alive (is_alive=true) with no activity for >10 minutes
 *   LIE-4: Any receipt has engine_source='synthetic' (must never reach the DB)
 *
 * Runs every 30 seconds when mounted; stops on unmount.
 * Uses the server endpoint so 3 raw Supabase queries per scan
 * become one $fetch call — service role key stays server-side.
 */

export interface HealthLie {
  code: 'LIE-1' | 'LIE-2' | 'LIE-3' | 'LIE-4';
  description: string;
  evidence: Record<string, unknown>;
  detected_at: number;
}

interface HealthLiesResponse {
  lies: Array<Omit<HealthLie, 'detected_at'>>;
  scanned_at: string;
  all_clear: boolean;
}

const LIE_INTERVAL_MS = 30_000;

export function useHealthLieDetector() {
  const lies = ref<HealthLie[]>([]);
  const isRunning = ref(false);
  const lastScan = ref<number>(0);

  function mergeLies(incoming: Array<Omit<HealthLie, 'detected_at'>>) {
    const now = Date.now();
    const incomingCodes = new Set(incoming.map(l => l.code));

    // Clear resolved lies; keep LIE-3 which is computed locally from health endpoint
    const survived = lies.value.filter(l => !incomingCodes.has(l.code) && l.code === 'LIE-3');

    lies.value = [
      ...incoming.map(l => ({ ...l, detected_at: now })),
      ...survived,
    ];
  }

  async function scan() {
    if (!isRunning.value) return;
    lastScan.value = Date.now();
    try {
      const result = await $fetch<HealthLiesResponse>('/api/game/health-lies');
      mergeLies(result.lies);
    } catch {
      // Server unreachable — keep existing lies, don't clear them
    }
  }

  let interval: ReturnType<typeof setInterval> | null = null;

  function start() {
    if (isRunning.value) return;
    isRunning.value = true;
    scan();
    interval = setInterval(scan, LIE_INTERVAL_MS);
  }

  function stop() {
    isRunning.value = false;
    if (interval) { clearInterval(interval); interval = null; }
  }

  onMounted(start);
  onUnmounted(stop);

  return {
    lies: readonly(lies),
    isRunning: readonly(isRunning),
    lastScan: readonly(lastScan),
    scan,
    start,
    stop,
  };
}
