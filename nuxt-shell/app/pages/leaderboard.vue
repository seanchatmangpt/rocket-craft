<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Leaderboard' });

const { client } = useRocketSupabase();

// Live updates: re-fetch when a new receipt (from browser OR Rust CLI) lands
const { receiptBus } = useRocketSessionRealtime();

interface LeaderboardRow {
  rank: number | null;
  score: number;
  updated_at: string;
  players: { id: string; username: string | null; high_score: number } | null;
}

const rows = ref<LeaderboardRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const lastUpdate = ref<string | null>(null);

async function loadLeaderboard() {
  loading.value = true;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { data, error: err } = await (client as any)
    .from('leaderboard')
    .select('rank, score, updated_at, players(id, username, high_score)')
    .order('score', { ascending: false })
    .limit(100);
  loading.value = false;
  if (err) { error.value = err.message; return; }
  rows.value = data ?? [];
  lastUpdate.value = new Date().toLocaleTimeString();
}

// Re-fetch when any PASS receipt fires (leaderboard trigger already ran by then)
receiptBus.on((receipt) => {
  if (receipt.verdict === 'PASS') loadLeaderboard();
});

onMounted(loadLeaderboard);

const medal = (rank: number | null) => rank === 1 ? '🥇' : rank === 2 ? '🥈' : rank === 3 ? '🥉' : `#${rank ?? '?'}`;
</script>

<template>
  <main class="lb-shell">
    <header class="lb-header">
      <NuxtLink to="/game" class="back">← Mission Control</NuxtLink>
      <h1>Leaderboard</h1>
      <span class="live-badge">● LIVE</span>
    </header>

    <div v-if="loading" class="status">Loading…</div>
    <div v-else-if="error" class="status error">{{ error }}</div>
    <div v-else-if="rows.length === 0" class="status">No sessions proven yet.</div>

    <ol v-else class="lb-list">
      <li v-for="r in rows" :key="r.players?.id ?? r.rank ?? 0" class="lb-row">
        <span class="rank">{{ medal(r.rank) }}</span>
        <span class="name">{{ r.players?.username ?? 'Anonymous Pilot' }}</span>
        <span class="score">{{ r.score.toLocaleString() }} <small>events</small></span>
        <span class="ts">{{ new Date(r.updated_at).toLocaleDateString() }}</span>
      </li>
    </ol>
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
  display: flex; align-items: center; gap: 1rem;
  border-bottom: 1px solid #1e3a5f; padding-bottom: 0.75rem; margin-bottom: 1.5rem;
}
.lb-header h1 { font-size: 1rem; color: #00f0ff; margin: 0; flex: 1; }
.back { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.live-badge { font-size: 0.7rem; color: #00c853; animation: pulse 2s infinite; }
@keyframes pulse { 0%,100% { opacity: 1 } 50% { opacity: 0.4 } }
.status { color: #666; text-align: center; padding: 2rem; font-size: 0.85rem; }
.status.error { color: #ff4444; }
.lb-list { list-style: none; margin: 0; padding: 0; }
.lb-row {
  display: flex; align-items: center; gap: 1rem;
  padding: 0.6rem 0; border-bottom: 1px solid #0d1117;
}
.lb-row:first-child { border-top: none; }
.rank { width: 3rem; text-align: center; font-size: 1rem; }
.name { flex: 1; color: #e0e0e0; }
.score { color: #00f0ff; font-weight: bold; min-width: 8rem; text-align: right; }
.score small { color: #666; font-weight: normal; font-size: 0.7rem; }
.ts { color: #555; font-size: 0.7rem; min-width: 6rem; text-align: right; }
</style>
