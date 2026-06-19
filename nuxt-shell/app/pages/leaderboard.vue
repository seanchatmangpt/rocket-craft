<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Leaderboard' });

// Live updates: re-fetch when a new PASS receipt fires (trigger already ran by then)
const { receiptBus } = useRocketSessionRealtime();

interface LeaderboardRow {
  rank: number;
  player_id: string;
  display_name: string | null;
  total_receipts: number;
  pass_receipts: number;
  pass_rate_pct: number | null;
  last_pass_at: string | null;
  best_ocel_events: number | null;
}

const rows = ref<LeaderboardRow[]>([]);
const total = ref(0);
const loading = ref(true);
const error = ref<string | null>(null);
const lastUpdate = ref<string | null>(null);

async function loadLeaderboard() {
  loading.value = true;
  error.value = null;
  try {
    const data = await $fetch<{ rows: LeaderboardRow[]; total: number }>('/api/game/leaderboard?limit=100');
    rows.value = data.rows;
    total.value = data.total;
    lastUpdate.value = new Date().toLocaleTimeString();
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to load leaderboard';
  } finally {
    loading.value = false;
  }
}

receiptBus.on((receipt) => {
  if (receipt.verdict === 'PASS') loadLeaderboard();
});

onMounted(loadLeaderboard);

const medal = (rank: number) => rank === 1 ? '🥇' : rank === 2 ? '🥈' : rank === 3 ? '🥉' : `#${rank}`;
const passRate = (r: LeaderboardRow) => r.pass_rate_pct != null ? `${r.pass_rate_pct.toFixed(0)}%` : '—';
</script>

<template>
  <main class="lb-shell">
    <header class="lb-header">
      <NuxtLink to="/game" class="back">← Mission Control</NuxtLink>
      <h1>Leaderboard</h1>
      <span class="live-badge">● LIVE</span>
      <span v-if="lastUpdate" class="update-ts">updated {{ lastUpdate }}</span>
    </header>

    <div v-if="loading" class="status">Loading…</div>
    <div v-else-if="error" class="status error">{{ error }}</div>
    <div v-else-if="rows.length === 0" class="status">No proven sessions yet.</div>

    <ol v-else class="lb-list">
      <li v-for="r in rows" :key="r.player_id" class="lb-row">
        <span class="rank">{{ medal(r.rank) }}</span>
        <span class="name">{{ r.display_name ?? 'Anonymous Pilot' }}</span>
        <span class="score">{{ (r.best_ocel_events ?? 0).toLocaleString() }} <small>events</small></span>
        <span class="pass-rate" :class="{ high: (r.pass_rate_pct ?? 0) >= 80 }">{{ passRate(r) }} pass</span>
        <span class="receipts">{{ r.pass_receipts }}/{{ r.total_receipts }}</span>
        <span class="ts">{{ r.last_pass_at ? new Date(r.last_pass_at).toLocaleDateString() : '—' }}</span>
      </li>
    </ol>
    <p v-if="total > rows.length" class="more">Showing top {{ rows.length }} of {{ total }} pilots</p>
  </main>
</template>

<style scoped>
.lb-shell {
  min-height: 100dvh;
  background: #0b0f19;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  padding: 1rem;
}
.lb-header {
  display: flex; align-items: center; gap: 1rem; flex-wrap: wrap;
  border-bottom: 1px solid #1e3a5f; padding-bottom: 0.75rem; margin-bottom: 1.5rem;
}
.lb-header h1 { font-size: 1rem; color: #00f0ff; margin: 0; flex: 1; }
.back { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.live-badge { font-size: 0.7rem; color: #00c853; animation: pulse 2s infinite; }
.update-ts { font-size: 0.65rem; color: #444; }
@keyframes pulse { 0%,100% { opacity: 1 } 50% { opacity: 0.4 } }
.status { color: #666; text-align: center; padding: 2rem; font-size: 0.85rem; }
.status.error { color: #ff4444; }
.lb-list { list-style: none; margin: 0; padding: 0; }
.lb-row {
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.6rem 0; border-bottom: 1px solid #0d1117; font-size: 0.8rem;
}
.lb-row:first-child { border-top: none; }
.rank { width: 3rem; text-align: center; font-size: 1rem; }
.name { flex: 1; color: #e0e0e0; }
.score { color: #00f0ff; font-weight: bold; min-width: 7rem; text-align: right; }
.score small { color: #666; font-weight: normal; font-size: 0.65rem; }
.pass-rate { color: #666; min-width: 4.5rem; text-align: right; font-size: 0.7rem; }
.pass-rate.high { color: #00c853; }
.receipts { color: #555; font-size: 0.7rem; min-width: 3rem; text-align: right; }
.ts { color: #555; font-size: 0.7rem; min-width: 6rem; text-align: right; }
.more { color: #444; font-size: 0.7rem; text-align: center; margin-top: 1rem; }
</style>
