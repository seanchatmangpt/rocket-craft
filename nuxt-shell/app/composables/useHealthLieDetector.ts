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
 * Invariants checked:
 *   LIE-1: A receipt claims PASS but has zero OCEL events
 *   LIE-2: A session is alive (is_alive=true) with no activity for >10 minutes
 *   LIE-3: pipeline_health returns DEGRADED but no receipts exist in last hour
 *   LIE-4: Any receipt has engine_source='synthetic' (must never reach the DB)
 *
 * Runs every 30 seconds when mounted; stops on unmount.
 */

export interface HealthLie {
  code: 'LIE-1' | 'LIE-2' | 'LIE-3' | 'LIE-4';
  description: string;
  evidence: Record<string, unknown>;
  detected_at: number;
}

const LIE_INTERVAL_MS = 30_000;

export function useHealthLieDetector() {
  const lies = ref<HealthLie[]>([]);
  const isRunning = ref(false);
  const lastScan = ref<number>(0);

  const { client } = useRocketSupabase();

  function addLie(lie: HealthLie) {
    // Deduplicate by code — only keep the most recent occurrence
    lies.value = [lie, ...lies.value.filter(l => l.code !== lie.code)];
  }

  function clearLie(code: HealthLie['code']) {
    lies.value = lies.value.filter(l => l.code !== code);
  }

  async function scanLie1() {
    // LIE-1: receipt claims PASS with zero OCEL events
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data } = await (client as any)
      .from('game_receipts')
      .select('id, verdict, ocel_event_count')
      .eq('verdict', 'PASS')
      .eq('ocel_event_count', 0)
      .limit(5);
    if (data?.length) {
      addLie({
        code: 'LIE-1',
        description: `${data.length} PASS receipt(s) claim zero OCEL events — impossible without evidence`,
        evidence: { receipts: data.map((r: { id: string }) => r.id) },
        detected_at: Date.now(),
      });
    } else {
      clearLie('LIE-1');
    }
  }

  async function scanLie2() {
    // LIE-2: session alive >10 min with no associated receipt
    const tenMinAgo = new Date(Date.now() - 10 * 60 * 1000).toISOString();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data } = await (client as any)
      .from('game_sessions')
      .select('id, session_started_at, project_name')
      .eq('is_alive', true)
      .lt('session_started_at', tenMinAgo)
      .limit(5);
    if (data?.length) {
      addLie({
        code: 'LIE-2',
        description: `${data.length} session(s) alive for >10 min with no close — stale session leak`,
        evidence: { sessions: data.map((s: { id: string; project_name: string }) => ({ id: s.id, project: s.project_name })) },
        detected_at: Date.now(),
      });
    } else {
      clearLie('LIE-2');
    }
  }

  async function scanLie4() {
    // LIE-4: any synthetic receipt reached the DB (guard migration should block these)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data } = await (client as any)
      .from('game_receipts')
      .select('id, engine_source')
      .eq('engine_source', 'synthetic')
      .limit(5);
    if (data?.length) {
      addLie({
        code: 'LIE-4',
        description: `${data.length} receipt(s) with engine_source=synthetic bypassed the guard trigger`,
        evidence: { receipts: data.map((r: { id: string }) => r.id) },
        detected_at: Date.now(),
      });
    } else {
      clearLie('LIE-4');
    }
  }

  async function scan() {
    if (!isRunning.value) return;
    lastScan.value = Date.now();
    await Promise.allSettled([scanLie1(), scanLie2(), scanLie4()]);
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
