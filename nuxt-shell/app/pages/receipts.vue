<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Session Receipts' });

const { client } = useRocketSupabase();

// Live updates when receipts arrive from browser sessions OR the Rust CLI cook pipeline
const { receiptBus, latestReceipt } = useRocketSessionRealtime();

interface ReceiptRow {
  id: string;
  session_id: string;
  verdict: 'PASS' | 'FAIL' | 'PENDING';
  milestone: string;
  ocel_event_count: number;
  ocel_lifecycle: string[];
  engine_source: string;
  receipt_hash: string;
  proven_at: string;
}

const receipts = ref<ReceiptRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

async function loadReceipts() {
  loading.value = true;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { data, error: err } = await (client as any)
    .from('game_receipts')
    .select('id, session_id, verdict, milestone, ocel_event_count, ocel_lifecycle, engine_source, receipt_hash, proven_at')
    .order('proven_at', { ascending: false })
    .limit(50);
  loading.value = false;
  if (err) { error.value = err.message; return; }
  receipts.value = data ?? [];
}

onMounted(loadReceipts);

// Prepend new receipt immediately when Realtime fires (avoids full re-fetch latency)
receiptBus.on((r) => {
  receipts.value = [r as ReceiptRow, ...receipts.value].slice(0, 50);
});

const verdictColor = (v: string) => v === 'PASS' ? '#00c853' : v === 'FAIL' ? '#ff4444' : '#888';
const shortHash = (h: string) => h.slice(0, 12) + '…';
const shortId = (id: string) => id.slice(0, 8);
</script>

<template>
  <main class="receipts-shell">
    <header class="receipts-header">
      <NuxtLink to="/game" class="back">← Mission Control</NuxtLink>
      <h1>Session Receipts</h1>
      <button class="refresh-btn" :disabled="loading" @click="loadReceipts">↻ Refresh</button>
    </header>

    <div v-if="loading" class="status-line">Loading receipts…</div>
    <div v-else-if="error" class="status-line error">{{ error }}</div>
    <div v-else-if="receipts.length === 0" class="status-line">No receipts yet — complete a game session to create one.</div>

    <table v-else class="receipts-table">
      <thead>
        <tr>
          <th>Verdict</th>
          <th>Milestone</th>
          <th>Events</th>
          <th>Engine</th>
          <th>Lifecycle</th>
          <th>Session</th>
          <th>Hash</th>
          <th>Proven</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in receipts" :key="r.id">
          <td :style="{ color: verdictColor(r.verdict), fontWeight: 'bold' }">{{ r.verdict }}</td>
          <td class="mono">{{ r.milestone }}</td>
          <td class="num">{{ r.ocel_event_count }}</td>
          <td class="mono">{{ r.engine_source }}</td>
          <td class="lifecycle">{{ r.ocel_lifecycle.join(' → ') }}</td>
          <td class="mono dimmed">{{ shortId(r.session_id) }}</td>
          <td class="mono dimmed" :title="r.receipt_hash">{{ shortHash(r.receipt_hash) }}</td>
          <td class="dimmed">{{ new Date(r.proven_at).toLocaleString() }}</td>
        </tr>
      </tbody>
    </table>
  </main>
</template>

<style scoped>
.receipts-shell {
  min-height: 100dvh;
  background: #0b0f19;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
  padding: 1rem;
}
.receipts-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1.5rem;
  border-bottom: 1px solid #1e3a5f;
  padding-bottom: 0.75rem;
}
.receipts-header h1 { font-size: 1rem; color: #00f0ff; margin: 0; flex: 1; }
.back { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.refresh-btn {
  background: none; border: 1px solid #1e3a5f; color: #666;
  padding: 0.2rem 0.6rem; cursor: pointer; font-family: inherit; font-size: 0.75rem;
}
.refresh-btn:hover:not(:disabled) { border-color: #00f0ff; color: #00f0ff; }
.status-line { color: #666; font-size: 0.85rem; padding: 2rem; text-align: center; }
.status-line.error { color: #ff4444; }
.receipts-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; }
.receipts-table th {
  text-align: left; color: #888; border-bottom: 1px solid #1e3a5f;
  padding: 0.4rem 0.75rem; font-weight: normal; letter-spacing: 0.05em;
}
.receipts-table td { padding: 0.4rem 0.75rem; border-bottom: 1px solid #0d1117; }
.receipts-table tr:hover td { background: #0d1117; }
.mono { font-family: 'Courier New', monospace; }
.num { text-align: right; color: #00f0ff; }
.dimmed { color: #555; }
.lifecycle { color: #888; font-size: 0.7rem; }
</style>
