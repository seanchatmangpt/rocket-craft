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

const health = ref<PipelineHealth | null>(null);
const chainStatus = ref<ChainVerifyResult | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const liveViewers = ref(0);        // Realtime presence: other dashboard viewers
const liveSessionsRT = ref(0);     // Realtime: game sessions with active OCEL streams

const { client } = useRocketSupabase();

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
  presenceChannel
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
  setupPresence();
  setupSessionCountChannel();
  timer = setInterval(() => { loadHealth(); loadChainStatus(); loadConformance(); }, 30_000);

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
});
</script>

<template>
  <main class="pipeline-page">
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

      <!-- Chain integrity -->
      <section class="chain-status">
        <h2>OCEL Chain Integrity</h2>
        <template v-if="chainStatus">
          <div :class="['chain-badge', chainStatus.overall.toLowerCase()]">
            {{ chainStatus.overall }} — {{ chainStatus.sessions_checked }} session(s) checked
          </div>
          <ul v-if="chainStatus.breaks.length" class="break-list">
            <li v-for="b in chainStatus.breaks" :key="b.session_id">
              <span class="break-session">{{ b.session_id.slice(0, 8) }}…</span>
              <span class="break-msg"> — {{ b.message }}</span>
              <span v-if="b.broken_at !== null" class="break-seq"> (seq {{ b.broken_at }})</span>
              <a
                class="ocel-dl-link"
                :href="`/api/game/ocel-export?session_id=${b.session_id}`"
                :download="`ocel2-${b.session_id.slice(0, 8)}.json`"
                title="Download OCEL 2.0 JSON for pm4py conformance check"
              >↓ OCEL 2.0</a>
            </li>
          </ul>
        </template>
        <div v-else class="chain-badge unknown">
          Chain verify unavailable — run `rocket supabase migrate`
        </div>
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
.chain-status h2 { font-size: 0.9rem; color: #94a3b8; margin-bottom: 0.5rem; }
.chain-badge { display: inline-block; padding: 0.4rem 1rem; border-radius: 4px; font-size: 0.875rem; }
.chain-badge.pass { background: #14532d; color: #86efac; }
.chain-badge.fail { background: #450a0a; color: #fca5a5; }
.chain-badge.unknown { background: #1e293b; color: #94a3b8; }
.break-list { margin: 0.75rem 0 0 1rem; color: #fca5a5; font-size: 0.8rem; list-style: none; padding: 0; }
.break-list li { display: flex; align-items: baseline; gap: 0.4rem; padding: 0.2rem 0; }
.break-session { font-weight: 600; }
.break-msg { flex: 1; }
.break-seq { color: #f97316; }
.ocel-dl-link { color: #7dd3fc; text-decoration: none; font-size: 0.75rem; border: 1px solid #334155; padding: 0.1rem 0.4rem; border-radius: 2px; white-space: nowrap; }
.ocel-dl-link:hover { background: #1e293b; }
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
</style>
