<script setup lang="ts">
/**
 * Pipeline Health Dashboard
 *
 * Queries the pipeline_health view (migration 000004) to show the current
 * state of the cook+session proof pipeline. Refreshes on mount and every 30s.
 *
 * Van der Aalst doctrine: health score is derived from process evidence,
 * not from a flag. A pipeline is HEALTHY only when PASS receipts > 0
 * and the OCEL chain has no breaks.
 */
/**
 * Pipeline Health Dashboard
 *
 * Uses /api/game/pipeline-health (Nitro KV cached, 20s TTL) for fast loads.
 * Pattern: ~/nuxt-layer/server/utils/setDb.ts — Nitro useStorage KV caching.
 *
 * Realtime presence from nuxt-supabase-book chapter-4: tracks live viewers of
 * this page and live game sessions via Supabase Realtime broadcast channel.
 *
 * Van der Aalst doctrine: health score derived from evidence, not from flags.
 */
useHead({ title: 'Rocket-Craft — Pipeline Health' });

interface PipelineHealth {
  total_receipts: number;
  pass_receipts: number;
  fail_receipts: number;
  pending_receipts: number;
  pass_rate_pct: number;
  real_ue4_receipts: number;
  sessions_with_real_ue4: number;
  total_sessions: number;
  active_sessions: number;
  total_players: number;
  last_receipt_at: string | null;
  cached_at?: string;
  cache_hit?: boolean;
}

interface ChainVerifyResult {
  overall: 'PASS' | 'FAIL' | 'UNKNOWN';
  sessions_checked: number;
  breaks: Array<{ session_id: string; message: string; broken_at: number | null }>;
}

interface DailyStatRow {
  day: string;
  sessions: number | null;
  unique_players: number | null;
  receipts: number;
  pass_receipts: number;
  fail_receipts: number;
  real_ue4_receipts: number;
  avg_ocel_events: number;
  pass_rate_pct: number | null;
}

const chainStatus = ref<ChainVerifyResult | null>(null);
const dailyStats = ref<DailyStatRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const liveViewers = ref(0);        // Realtime presence: other dashboard viewers
const liveSessionsRT = ref(0);     // Realtime: game sessions with active OCEL streams

const { client } = useRocketSupabase();

// SSR-aware health: server fetches once, payload carries it to client with no
// round-trip flash. useServerData revalidates client-side after 20 s (matching
// the Nitro KV TTL on pipeline-health.get.ts).
const ssrHealth = await useServerData<PipelineHealth>(
  'pipeline:health',
  () => $fetch<PipelineHealth>('/api/game/pipeline-health'),
  { ttl: 20_000 },
);
const health = ref<PipelineHealth | null>(ssrHealth.value);
watch(ssrHealth, v => { if (v) health.value = v; });

// Load from cached Nitro KV endpoint (falls back to direct Supabase in dev if needed)
async function loadHealth(bust = false) {
  loading.value = true;
  error.value = null;
  try {
    const data = await $fetch<PipelineHealth>(`/api/game/pipeline-health${bust ? '?bust=1' : ''}`);
    health.value = data;
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load pipeline health';
  } finally {
    loading.value = false;
  }
}

async function loadChainStatus() {
  try {
    const result = await $fetch<ChainVerifyResult>('/api/game/chain-verify');
    chainStatus.value = result;
  } catch {
    chainStatus.value = null;
  }
}

const healthScore = computed(() => {
  if (!health.value) return null;
  const h = health.value;
  if (h.total_receipts === 0) return 0;
  if (chainStatus.value?.overall === 'FAIL') return 10;
  if (h.real_ue4_receipts === 0) return 40;
  if (h.pass_rate_pct < 50) return 50;
  if (h.pass_rate_pct < 90) return 75;
  return 100;
});

const healthLabel = computed(() => {
  const s = healthScore.value;
  if (s === null) return '—';
  if (s === 100) return 'HEALTHY';
  if (s >= 75) return 'DEGRADED';
  if (s >= 40) return 'WARNING';
  return 'CRITICAL';
});

const healthColor = computed(() => {
  const s = healthScore.value;
  if (s === null) return '#64748b';
  if (s === 100) return '#22c55e';
  if (s >= 75) return '#f59e0b';
  if (s >= 40) return '#f97316';
  return '#ef4444';
});

// Process conformance (Van der Aalst fitness/precision/generalization/simplicity)
const { conformance, fitnessLabel, fitnessColor, load: loadConformance } = useProcessConformance();

// ── Daily rollup stats from /api/game/dashboard-stats ─────────────────────
async function loadDailyStats() {
  try {
    const result = await $fetch<{ source: string; rows: DailyStatRow[] }>('/api/game/dashboard-stats');
    dailyStats.value = result.rows ?? [];
  } catch { /* non-critical — dashboard still works without rollups */ }
}

// ── Last receipt lifecycle for ChainVisualization flow diagram ───────────────
// Fetches the most recent PASS receipt's ocel_lifecycle to show as the process flow
const lastReceiptLifecycle = ref<string[]>([
  'CookStarted', 'WasmPackaged', 'JsEmitted', 'DataPakStaged', 'PackageVerified'
]);
async function loadLastLifecycle() {
  try {
    // Use the server receipts endpoint — service key stays server-side
    const data = await $fetch<{ rows: Array<{ ocel_lifecycle: string[] }> }>(
      '/api/game/receipts?limit=1&engine_source=rocket_cli&verdict=PASS'
    );
    if (data.rows[0]?.ocel_lifecycle?.length) {
      lastReceiptLifecycle.value = data.rows[0].ocel_lifecycle;
    }
  } catch { /* keep default lifecycle */ }
}

// ── Network status (offline detection beyond navigator.onLine) ────────────────
// OfflineBanner pattern from expo-supabase-ai-template: probes real Supabase
// endpoint so "OS says online" ≠ "Supabase is reachable" distinction is surfaced.
const { status: networkStatus, isOffline, reconnect } = useNetworkStatus();

// ── Health lie detector (LieDetector pattern from expo-supabase-ai-template) ──
// Scans pipeline invariants every 30s and surfaces contradictions (lies):
//   LIE-1: PASS receipt with zero OCEL events
//   LIE-2: alive session >10 min with no close
//   LIE-4: synthetic receipt in the DB (bypass of the guard trigger)
const { lies, lastScan: lieScanAt } = useHealthLieDetector();

// ── Cook log SSE monitor ──────────────────────────────────────────────────────
// Connects to GET /api/game/cook-log (SSE) and streams real-time OCEL events
// emitted by the UAT cook process. Activates when "Connect Cook Monitor" is clicked.
interface CookLogEvt {
  activity: string;
  raw_line: string;
  timestamp_ms: number;
  detail?: string;
  line_no: number;
}
const cookEvents = ref<CookLogEvt[]>([]);
const cookMonitorActive = ref(false);
let cookEventSource: EventSource | null = null;

function startCookMonitor() {
  if (cookEventSource) return;
  cookMonitorActive.value = true;
  cookEvents.value = [];
  cookEventSource = new EventSource('/api/game/cook-log');
  cookEventSource.onmessage = (e) => {
    try {
      const evt = JSON.parse(e.data) as CookLogEvt;
      if (evt.activity !== 'StreamOpened') cookEvents.value.unshift(evt);
      if (cookEvents.value.length > 100) cookEvents.value.splice(100);
    } catch { /* ignore parse errors */ }
  };
  cookEventSource.onerror = () => {
    cookMonitorActive.value = false;
    cookEventSource?.close();
    cookEventSource = null;
  };
}

function stopCookMonitor() {
  cookEventSource?.close();
  cookEventSource = null;
  cookMonitorActive.value = false;
}

const cookActivityColor = (activity: string) => {
  if (activity.includes('Error') || activity.includes('Failed')) return '#ef4444';
  if (activity === 'CookFinished' || activity === 'PackageComplete') return '#22c55e';
  if (activity === 'CookStarted') return '#7dd3fc';
  return '#94a3b8';
};

// Realtime receipt bus — bust KV cache when a new PASS receipt lands so
// the next poll shows fresh numbers (pattern from nuxt-supabase-book chapter-4)
const { receiptBus } = useRocketSessionRealtime();

// Supabase Realtime presence — track viewers of this dashboard page
// Pattern: nuxt-supabase-book chapter-4 "Presence: Track who's online"
let presenceChannel: ReturnType<typeof client.channel> | null = null;

function setupPresence() {
  if (!client) return;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  presenceChannel = (client as any).channel('pipeline-dashboard', {
    config: { presence: { key: `viewer-${Math.random().toString(36).slice(2, 8)}` } },
  });
  presenceChannel!
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    .on('presence', { event: 'sync' }, () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const state = (presenceChannel as any)?.presenceState?.() ?? {};
      liveViewers.value = Object.keys(state).length;
    })
    .subscribe(async (status: string) => {
      if (status === 'SUBSCRIBED') {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        await (presenceChannel as any)?.track({ page: 'pipeline', at: new Date().toISOString() });
      }
    });
}

// Live active session count via ocel_events Realtime — increments when
// a new event arrives for a session (session is alive while events stream in)
let sessionCountChannel: ReturnType<typeof client.channel> | null = null;

function setupSessionCountChannel() {
  if (!client) return;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  sessionCountChannel = (client as any)
    .channel('ocel-live-sessions')
    .on('postgres_changes', { event: 'INSERT', schema: 'public', table: 'ocel_events' }, () => {
      // Each new event increments a debounced live-session estimator
      // Actual count is in pipeline_health.active_sessions; this just signals activity
      loadHealth(true);
    })
    .subscribe();
}

let timer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  loadHealth();
  loadChainStatus();
  loadConformance();
  loadLastLifecycle();
  loadDailyStats();
  setupPresence();
  setupSessionCountChannel();
  timer = setInterval(() => { loadHealth(); loadChainStatus(); loadConformance(); loadLastLifecycle(); loadDailyStats(); }, 30_000);

  // Bust KV cache and reload immediately when a PASS receipt fires
  receiptBus.on((r) => {
    if (r.verdict === 'PASS') {
      loadHealth(true);
      loadConformance();
    }
  });
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  presenceChannel?.unsubscribe();
  sessionCountChannel?.unsubscribe();
  stopCookMonitor();
});
</script>

<template>
  <main class="pipeline-page">
    <!-- Offline / Supabase unreachable banner -->
    <div
      v-if="isOffline"
      class="offline-banner"
      :class="networkStatus === 'reconnecting' ? 'reconnecting' : 'offline'"
    >
      <span v-if="networkStatus === 'offline'">
        ⚠ Supabase unreachable — pipeline metrics may be stale
        <button class="banner-reconnect" @click="reconnect">Reconnect</button>
      </span>
      <span v-else>↻ Reconnecting to Supabase…</span>
    </div>

    <header class="pipeline-header">
      <NuxtLink to="/game">← Mission Control</NuxtLink>
      <h1>Pipeline Health</h1>
      <span v-if="liveViewers > 1" class="presence-badge" :title="`${liveViewers} viewers on this page`">
        ● {{ liveViewers }} live
      </span>
      <span class="refresh-hint">
        auto-refresh 30s
        <span v-if="health?.cache_hit" class="cache-badge" title="Served from Nitro KV cache">· cached</span>
        <button class="bust-btn" title="Force fresh data" @click="loadHealth(true)">↻</button>
      </span>
    </header>

    <div v-if="loading && !health" class="loading">Loading pipeline metrics…</div>
    <div v-else-if="error" class="error-banner">{{ error }}</div>

    <template v-else>
      <!-- Health score card -->
      <section class="health-score-card" :style="{ borderColor: healthColor }">
        <div class="score-number" :style="{ color: healthColor }">
          {{ healthScore ?? '—' }}
        </div>
        <div class="score-label" :style="{ color: healthColor }">{{ healthLabel }}</div>
      </section>

      <!-- Metrics grid -->
      <section v-if="health" class="metrics-grid">
        <div class="metric">
          <span class="metric-value">{{ health.total_receipts }}</span>
          <span class="metric-label">Total Receipts</span>
        </div>
        <div class="metric pass">
          <span class="metric-value">{{ health.pass_receipts }}</span>
          <span class="metric-label">PASS</span>
        </div>
        <div class="metric fail">
          <span class="metric-value">{{ health.fail_receipts }}</span>
          <span class="metric-label">FAIL</span>
        </div>
        <div class="metric">
          <span class="metric-value">{{ health.pass_rate_pct?.toFixed(1) }}%</span>
          <span class="metric-label">Pass Rate</span>
        </div>
        <div class="metric real">
          <span class="metric-value">{{ health.real_ue4_receipts }}</span>
          <span class="metric-label">Real UE4 Receipts</span>
        </div>
        <div class="metric">
          <span class="metric-value">{{ health.total_sessions }}</span>
          <span class="metric-label">Sessions</span>
        </div>
        <div class="metric">
          <span class="metric-value">{{ health.active_sessions }}</span>
          <span class="metric-label">Active</span>
        </div>
        <div class="metric">
          <span class="metric-value">{{ health.total_players }}</span>
          <span class="metric-label">Players</span>
        </div>
      </section>

      <!-- Chain integrity + OCEL flow diagram (ChainVisualization component) -->
      <!-- Adapted from dashboard.bak/app/components/evidence/ChainVerifier.vue +
           dashboard.bak/app/components/ml/ProcessMiningVisualization.vue -->
      <section class="chain-status">
        <ChainVisualization
          :lifecycle="lastReceiptLifecycle"
          :chain-status="chainStatus"
          title="OCEL Cook Pipeline Flow"
        />
        <!-- Download links for broken sessions -->
        <ul v-if="chainStatus?.breaks.length" class="break-dl-list">
          <li v-for="b in chainStatus.breaks" :key="b.session_id">
            <a
              class="ocel-dl-link"
              :href="`/api/game/ocel-export?session_id=${b.session_id}`"
              :download="`ocel2-${b.session_id.slice(0, 8)}.json`"
              title="Download OCEL 2.0 JSON for pm4py conformance check"
            >↓ OCEL 2.0 ({{ b.session_id.slice(0, 8) }}…)</a>
          </li>
        </ul>
      </section>

      <!-- Process conformance (Van der Aalst four quality dimensions) -->
      <section v-if="conformance" class="conformance-section">
        <h2>Process Conformance</h2>
        <p class="conformance-model">Declared model: <code>GameSessionStarted → FrameRendered → InputAdmitted*</code></p>
        <div class="conformance-verdict" :style="{ color: fitnessColor }">
          {{ fitnessLabel }} ({{ conformance.conformant_sessions }}/{{ conformance.total_sessions }} sessions)
        </div>
        <div class="conformance-metrics">
          <div class="cmetric" :title="'Fraction of sessions containing all required activities'">
            <span class="cmetric-val" :style="{ color: fitnessColor }">{{ (conformance.fitness * 100).toFixed(1) }}%</span>
            <span class="cmetric-label">Fitness</span>
          </div>
          <div class="cmetric" :title="'Fraction of sessions with no unexpected activity types'">
            <span class="cmetric-val">{{ (conformance.precision * 100).toFixed(1) }}%</span>
            <span class="cmetric-label">Precision</span>
          </div>
          <div class="cmetric" :title="'Fraction of required activities seen at least once'">
            <span class="cmetric-val">{{ (conformance.generalization * 100).toFixed(1) }}%</span>
            <span class="cmetric-label">Generalization</span>
          </div>
          <div class="cmetric" :title="'Model simplicity (1.0 = single trace, no branches)'">
            <span class="cmetric-val">{{ (conformance.simplicity * 100).toFixed(0) }}%</span>
            <span class="cmetric-label">Simplicity</span>
          </div>
        </div>
        <details v-if="conformance.non_conformant.length" class="non-conformant">
          <summary>{{ conformance.non_conformant.length }} non-conformant session(s)</summary>
          <ul>
            <li v-for="nc in conformance.non_conformant" :key="nc.session_id">
              <span class="nc-sid">{{ nc.session_id.slice(0, 8) }}…</span>
              <span v-if="nc.missing.length" class="nc-missing"> missing: {{ nc.missing.join(', ') }}</span>
              <span v-if="nc.extra.length" class="nc-extra"> extra: {{ nc.extra.join(', ') }}</span>
              <a
                class="ocel-dl-link"
                :href="`/api/game/ocel-export?session_id=${nc.session_id}`"
                :download="`ocel2-${nc.session_id.slice(0, 8)}.json`"
              >↓ OCEL 2.0</a>
            </li>
          </ul>
        </details>
        <div v-if="conformance.all_activities_seen.length" class="activities-seen">
          All activity types seen: {{ conformance.all_activities_seen.join(' · ') }}
        </div>
      </section>

      <!-- Daily rollup stats (from migration 000010 + /api/game/dashboard-stats) -->
      <section v-if="dailyStats.length" class="daily-stats-section">
        <h2>Daily Pipeline Rollup <span class="stat-src">(last 7 days)</span></h2>
        <table class="daily-table">
          <thead>
            <tr>
              <th>Day</th><th>Sessions</th><th>Receipts</th>
              <th>PASS</th><th>FAIL</th><th>Real UE4</th><th>Avg Events</th><th>Pass%</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in dailyStats" :key="row.day" :class="{ 'row-fail': (row.pass_rate_pct ?? 100) < 80 }">
              <td class="day-cell">{{ row.day }}</td>
              <td>{{ row.sessions ?? '—' }}</td>
              <td>{{ row.receipts }}</td>
              <td class="pass-cell">{{ row.pass_receipts }}</td>
              <td class="fail-cell">{{ row.fail_receipts }}</td>
              <td>{{ row.real_ue4_receipts }}</td>
              <td>{{ row.avg_ocel_events ? row.avg_ocel_events.toFixed(1) : '—' }}</td>
              <td :class="{ 'pass-cell': (row.pass_rate_pct ?? 0) >= 90, 'fail-cell': (row.pass_rate_pct ?? 100) < 80 }">
                {{ row.pass_rate_pct != null ? `${row.pass_rate_pct}%` : '—' }}
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- Health Lie Detector violations panel -->
      <section v-if="lies.length" class="lies-panel">
        <h2>
          ⚠ Pipeline Invariant Violations
          <span class="lies-count">{{ lies.length }}</span>
        </h2>
        <div v-for="lie in lies" :key="lie.code" class="lie-row">
          <span class="lie-code">{{ lie.code }}</span>
          <span class="lie-desc">{{ lie.description }}</span>
          <span class="lie-time">{{ new Date(lie.detected_at).toLocaleTimeString() }}</span>
        </div>
        <p class="lies-hint">
          Last invariant scan: {{ lieScanAt ? new Date(lieScanAt).toLocaleTimeString() : 'pending' }}
        </p>
      </section>

      <!-- Cook Log Monitor (SSE real-time stream from ~/ue4-cook-latest.log) -->
      <section class="cook-monitor">
        <h2>
          Cook Log Monitor
          <button v-if="!cookMonitorActive" class="cook-connect-btn" @click="startCookMonitor">
            ▶ Connect
          </button>
          <button v-else class="cook-stop-btn" @click="stopCookMonitor">
            ■ Disconnect
          </button>
          <span v-if="cookMonitorActive" class="cook-live-badge">● LIVE</span>
        </h2>
        <p class="cook-monitor-hint">
          Streams <code>~/ue4-cook-latest.log</code> → OCEL activities in real time.
          Run <code>rocket html5 cook --project Brm</code> to populate.
        </p>
        <div v-if="cookEvents.length" class="cook-event-list">
          <div
            v-for="evt in cookEvents"
            :key="evt.line_no"
            class="cook-evt"
            :style="{ borderLeftColor: cookActivityColor(evt.activity) }"
          >
            <span class="cook-activity" :style="{ color: cookActivityColor(evt.activity) }">
              {{ evt.activity }}
            </span>
            <span v-if="evt.detail" class="cook-detail">{{ evt.detail }}</span>
          </div>
        </div>
        <div v-else-if="cookMonitorActive" class="cook-waiting">
          Waiting for cook activity… (log is empty or no matching events yet)
        </div>
      </section>

      <!-- Last receipt -->
      <section v-if="health?.last_receipt_at" class="last-receipt">
        <span>Last receipt: {{ new Date(health.last_receipt_at).toLocaleString() }}</span>
      </section>
    </template>

    <nav class="pipeline-nav">
      <NuxtLink to="/receipts">Receipts</NuxtLink>
      <NuxtLink to="/leaderboard">Leaderboard</NuxtLink>
    </nav>
  </main>
</template>

<style scoped>
.pipeline-page { max-width: 800px; margin: 0 auto; padding: 2rem 1rem; font-family: monospace; }
.pipeline-header { display: flex; align-items: baseline; gap: 1.5rem; margin-bottom: 2rem; flex-wrap: wrap; }
.pipeline-header h1 { font-size: 1.5rem; margin: 0; }
.refresh-hint { font-size: 0.75rem; color: #64748b; display: flex; align-items: center; gap: 0.4rem; }
.presence-badge { font-size: 0.72rem; color: #22c55e; animation: pulse 2s infinite; }
.cache-badge { color: #475569; }
.bust-btn { background: none; border: none; color: #475569; cursor: pointer; font-size: 0.8rem; padding: 0; }
.bust-btn:hover { color: #7dd3fc; }
@keyframes pulse { 0%,100% { opacity:1 } 50% { opacity:0.5 } }
.loading, .error-banner { padding: 1rem; background: #1e293b; border-radius: 4px; }
.error-banner { color: #ef4444; }
.health-score-card {
  display: flex; flex-direction: column; align-items: center;
  padding: 2rem; border: 2px solid; border-radius: 8px; margin-bottom: 2rem;
}
.score-number { font-size: 4rem; font-weight: 700; line-height: 1; }
.score-label { font-size: 1.25rem; font-weight: 600; margin-top: 0.5rem; }
.metrics-grid {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-bottom: 2rem;
}
.metric {
  background: #1e293b; border-radius: 4px; padding: 1rem; text-align: center;
}
.metric.pass { border-left: 3px solid #22c55e; }
.metric.fail { border-left: 3px solid #ef4444; }
.metric.real { border-left: 3px solid #3b82f6; }
.metric-value { display: block; font-size: 1.75rem; font-weight: 700; }
.metric-label { display: block; font-size: 0.7rem; color: #94a3b8; margin-top: 0.25rem; }
.chain-status { margin-bottom: 1.5rem; }
.break-dl-list { list-style: none; padding: 0.5rem 0 0; margin: 0; display: flex; flex-wrap: wrap; gap: 0.5rem; }
.ocel-dl-link { color: #7dd3fc; text-decoration: none; font-size: 0.75rem; border: 1px solid #334155; padding: 0.1rem 0.4rem; border-radius: 2px; white-space: nowrap; }
.ocel-dl-link:hover { background: #1e293b; }
.daily-stats-section { margin-bottom: 2rem; }
.daily-stats-section h2 { font-size: 0.9rem; color: #94a3b8; margin-bottom: 0.5rem; }
.stat-src { font-size: 0.7rem; color: #475569; font-weight: 400; }
.daily-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; font-family: 'Courier New', monospace; }
.daily-table th { color: #64748b; border-bottom: 1px solid #1e293b; padding: 0.25rem 0.5rem; text-align: left; }
.daily-table td { padding: 0.25rem 0.5rem; border-bottom: 1px solid #0f172a; }
.day-cell { color: #94a3b8; }
.pass-cell { color: #22c55e; }
.fail-cell { color: #ef4444; }
.row-fail { background: rgba(239,68,68,0.05); }
.conformance-section { margin-bottom: 2rem; }
.conformance-section h2 { font-size: 0.9rem; color: #94a3b8; margin-bottom: 0.25rem; }
.conformance-model { font-size: 0.75rem; color: #475569; margin: 0 0 0.75rem; }
.conformance-model code { color: #7dd3fc; }
.conformance-verdict { font-size: 1.1rem; font-weight: 700; margin-bottom: 1rem; }
.conformance-metrics { display: flex; gap: 1rem; margin-bottom: 1rem; }
.cmetric { background: #1e293b; border-radius: 4px; padding: 0.75rem; text-align: center; flex: 1; }
.cmetric-val { display: block; font-size: 1.4rem; font-weight: 700; color: #7dd3fc; }
.cmetric-label { display: block; font-size: 0.65rem; color: #64748b; margin-top: 0.2rem; }
.non-conformant { font-size: 0.8rem; color: #fca5a5; margin-bottom: 0.5rem; }
.non-conformant summary { cursor: pointer; color: #f97316; }
.non-conformant ul { list-style: none; padding: 0.5rem 0 0 1rem; margin: 0; }
.non-conformant li { display: flex; align-items: baseline; gap: 0.5rem; padding: 0.2rem 0; }
.nc-sid { font-weight: 600; color: #94a3b8; }
.nc-missing { color: #f87171; }
.nc-extra { color: #fb923c; }
.activities-seen { font-size: 0.72rem; color: #475569; margin-top: 0.5rem; }
.last-receipt { font-size: 0.8rem; color: #94a3b8; margin-bottom: 1.5rem; }
.pipeline-nav { display: flex; gap: 1.5rem; }
.pipeline-nav a { color: #7dd3fc; text-decoration: none; font-size: 0.875rem; }

/* Cook monitor */
.cook-monitor { background: #0f172a; border: 1px solid #1e293b; border-radius: 6px; padding: 1rem; margin-bottom: 1.5rem; }
.cook-monitor h2 { font-size: 0.9rem; color: #94a3b8; margin: 0 0 0.5rem; display: flex; align-items: center; gap: 0.75rem; }
.cook-connect-btn, .cook-stop-btn { background: #1e293b; border: 1px solid #334155; color: #7dd3fc; font-size: 0.7rem; padding: 0.15rem 0.5rem; border-radius: 3px; cursor: pointer; }
.cook-stop-btn { color: #f87171; border-color: #7f1d1d; }
.cook-live-badge { font-size: 0.7rem; color: #22c55e; animation: pulse 2s infinite; }
.cook-monitor-hint { font-size: 0.72rem; color: #475569; margin: 0 0 0.75rem; }
.cook-monitor-hint code { color: #7dd3fc; }
.cook-event-list { max-height: 240px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
.cook-evt { padding: 0.25rem 0.5rem; border-left: 3px solid #334155; display: flex; align-items: baseline; gap: 0.75rem; }
.cook-activity { font-size: 0.78rem; font-weight: 700; white-space: nowrap; }
.cook-detail { font-size: 0.7rem; color: #64748b; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cook-waiting { font-size: 0.75rem; color: #475569; padding: 0.5rem 0; }

/* Offline banner */
.offline-banner {
  padding: 0.5rem 1rem; font-size: 0.8rem; display: flex; align-items: center;
  gap: 0.75rem; margin-bottom: 1rem; border-radius: 4px;
}
.offline-banner.offline { background: #451a03; color: #fed7aa; border: 1px solid #92400e; }
.offline-banner.reconnecting { background: #0c4a6e; color: #bae6fd; border: 1px solid #0369a1; }
.banner-reconnect {
  background: #92400e; border: none; color: #fff; font-size: 0.75rem;
  padding: 0.15rem 0.5rem; border-radius: 3px; cursor: pointer;
}
.banner-reconnect:hover { background: #b45309; }

/* Health lies panel */
.lies-panel {
  background: #1c0a0a; border: 1px solid #7f1d1d; border-radius: 6px;
  padding: 1rem; margin-bottom: 1.5rem;
}
.lies-panel h2 { font-size: 0.9rem; color: #f87171; margin: 0 0 0.75rem; display: flex; align-items: center; gap: 0.5rem; }
.lies-count {
  background: #7f1d1d; color: #fca5a5; font-size: 0.7rem; padding: 0.1rem 0.4rem;
  border-radius: 999px; font-weight: 700;
}
.lie-row {
  display: flex; align-items: baseline; gap: 0.75rem; font-size: 0.78rem;
  border-left: 3px solid #ef4444; padding: 0.25rem 0.5rem; margin-bottom: 0.35rem;
}
.lie-code { font-weight: 700; color: #ef4444; white-space: nowrap; }
.lie-desc { flex: 1; color: #fca5a5; }
.lie-time { font-size: 0.7rem; color: #64748b; white-space: nowrap; }
.lies-hint { font-size: 0.7rem; color: #475569; margin: 0.5rem 0 0; }
</style>
