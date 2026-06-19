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

const { client } = useRocketSupabase();

async function loadHealth() {
  loading.value = true;
  error.value = null;
  try {
    // Query the pipeline_health view
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data, error: err } = await (client as any)
      .from('pipeline_health')
      .select('*')
      .single();
    if (err) throw new Error(err.message);
    health.value = data as PipelineHealth;
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
    // Chain verify unavailable (e.g., Supabase not configured) — not fatal
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

let timer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  loadHealth();
  loadChainStatus();
  timer = setInterval(() => { loadHealth(); loadChainStatus(); }, 30_000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<template>
  <main class="pipeline-page">
    <header class="pipeline-header">
      <NuxtLink to="/game">← Mission Control</NuxtLink>
      <h1>Pipeline Health</h1>
      <span class="refresh-hint">auto-refresh 30s</span>
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
.pipeline-header { display: flex; align-items: baseline; gap: 1.5rem; margin-bottom: 2rem; }
.pipeline-header h1 { font-size: 1.5rem; margin: 0; }
.refresh-hint { font-size: 0.75rem; color: #64748b; }
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
.last-receipt { font-size: 0.8rem; color: #94a3b8; margin-bottom: 1.5rem; }
.pipeline-nav { display: flex; gap: 1.5rem; }
.pipeline-nav a { color: #7dd3fc; text-decoration: none; font-size: 0.875rem; }
</style>
