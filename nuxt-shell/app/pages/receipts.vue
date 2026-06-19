<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Session Receipts' });

const { client } = useRocketSupabase();

// Live updates when receipts arrive from browser sessions OR the Rust CLI cook pipeline
const { receiptBus } = useRocketSessionRealtime();

interface ReceiptRow {
  id: string;
  session_id: string | null;
  verdict: 'PASS' | 'FAIL' | 'PENDING';
  milestone: string;
  ocel_event_count: number;
  ocel_lifecycle: string[];
  engine_source: string;
  receipt_hash: string;
  proven_at: string;
  ed25519_sig: string | null;
  payload: Record<string, unknown> | null;
}

const receipts = ref<ReceiptRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
// sig verification state: receipt id → 'verifying' | 'ok' | 'fail' | 'unsigned' | 'no-key'
const sigStatus = ref<Record<string, 'verifying' | 'ok' | 'fail' | 'unsigned' | 'no-key'>>({});

async function loadReceipts() {
  loading.value = true;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { data, error: err } = await (client as any)
    .from('game_receipts')
    .select('id, session_id, verdict, milestone, ocel_event_count, ocel_lifecycle, engine_source, receipt_hash, proven_at, ed25519_sig, payload')
    .order('proven_at', { ascending: false })
    .limit(50);
  loading.value = false;
  if (err) { error.value = err.message; return; }
  receipts.value = data ?? [];
}

onMounted(loadReceipts);

// Prepend new receipt immediately when Realtime fires
receiptBus.on((r) => {
  receipts.value = [r as ReceiptRow, ...receipts.value].slice(0, 50);
});

async function verifySig(r: ReceiptRow) {
  if (!r.ed25519_sig) { sigStatus.value[r.id] = 'unsigned'; return; }
  sigStatus.value[r.id] = 'verifying';
  try {
    const result = await $fetch<{ verified: boolean; error?: string }>('/api/game/verify-signature', {
      method: 'POST',
      body: {
        verdict: r.verdict,
        receipt_hash: r.receipt_hash,
        session_id: r.session_id,
        proven_at: r.proven_at,
        ed25519_sig: r.ed25519_sig,
      },
    });
    if (result.error?.includes('not configured')) {
      sigStatus.value[r.id] = 'no-key';
    } else {
      sigStatus.value[r.id] = result.verified ? 'ok' : 'fail';
    }
  } catch {
    sigStatus.value[r.id] = 'fail';
  }
}

const verdictColor = (v: string) => v === 'PASS' ? '#00c853' : v === 'FAIL' ? '#ff4444' : '#888';
const shortHash = (h: string) => h ? h.slice(0, 14) + '…' : '—';
const shortId = (id: string | null) => id ? id.slice(0, 8) : '—';

const sigLabel: Record<string, string> = {
  verifying: '…',
  ok: '✓ Ed25519',
  fail: '✗ invalid',
  unsigned: '— unsigned',
  'no-key': '— no pubkey',
};
const sigColor: Record<string, string> = {
  verifying: '#888',
  ok: '#00c853',
  fail: '#ff4444',
  unsigned: '#555',
  'no-key': '#555',
};
</script>

<template>
  <main class="receipts-shell">
    <header class="receipts-header">
      <NuxtLink to="/game" class="back">← Mission Control</NuxtLink>
      <h1>Session Receipts</h1>
      <NuxtLink to="/pipeline" class="pipeline-link">Pipeline ↗</NuxtLink>
      <button class="refresh-btn" :disabled="loading" @click="loadReceipts">↻ Refresh</button>
    </header>

    <div v-if="loading" class="status-line">Loading receipts…</div>
    <div v-else-if="error" class="status-line error">{{ error }}</div>
    <div v-else-if="receipts.length === 0" class="status-line">
      No receipts yet — complete a game session or run <code>rocket html5 verify</code>.
    </div>

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
          <th>Signature</th>
          <th>Proven</th>
          <th>Proof</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in receipts" :key="r.id" :class="{ 'row-pass': r.verdict === 'PASS', 'row-fail': r.verdict === 'FAIL' }">
          <td :style="{ color: verdictColor(r.verdict), fontWeight: 'bold' }">{{ r.verdict }}</td>
          <td class="mono small">{{ r.milestone }}</td>
          <td class="num">{{ r.ocel_event_count }}</td>
          <td class="mono small" :class="{ 'real-ue4': r.engine_source === 'real_ue4' }">{{ r.engine_source }}</td>
          <td class="lifecycle">{{ r.ocel_lifecycle?.join(' → ') ?? '—' }}</td>
          <td class="mono dimmed">{{ shortId(r.session_id) }}</td>
          <td class="mono dimmed" :title="r.receipt_hash">{{ shortHash(r.receipt_hash) }}</td>
          <td>
            <button
              v-if="!sigStatus[r.id]"
              class="verify-btn"
              :disabled="!r.ed25519_sig"
              :title="r.ed25519_sig ? 'Verify Ed25519 signature' : 'No signature — browser receipt or key not configured'"
              @click="verifySig(r)"
            >
              {{ r.ed25519_sig ? '? Verify' : '— unsigned' }}
            </button>
            <span
              v-else
              :style="{ color: sigColor[sigStatus[r.id] ?? 'unsigned'] }"
              class="sig-result"
            >{{ sigLabel[sigStatus[r.id] ?? 'unsigned'] }}</span>
          </td>
          <td class="dimmed small">{{ new Date(r.proven_at).toLocaleString() }}</td>
          <td class="proof-links">
            <a
              v-if="r.session_id"
              class="proof-link"
              :href="`/api/game/ocel-export?session_id=${r.session_id}`"
              :download="`ocel2-${r.session_id.slice(0, 8)}.json`"
              title="Download OCEL 2.0 JSON for pm4py"
            >↓ OCEL</a>
          </td>
        </tr>
      </tbody>
    </table>

    <footer class="receipts-footer">
      <span>Receipts prove OCEL lifecycle · Hash chain verifiable via <code>rocket supabase chain-verify</code></span>
    </footer>
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
.back, .pipeline-link { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.refresh-btn {
  background: none; border: 1px solid #1e3a5f; color: #666;
  padding: 0.2rem 0.6rem; cursor: pointer; font-family: inherit; font-size: 0.75rem;
}
.refresh-btn:hover:not(:disabled) { border-color: #00f0ff; color: #00f0ff; }
.status-line { color: #666; font-size: 0.85rem; padding: 2rem; text-align: center; }
.status-line.error { color: #ff4444; }
.status-line code { color: #00f0ff; }
.receipts-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; }
.receipts-table th {
  text-align: left; color: #888; border-bottom: 1px solid #1e3a5f;
  padding: 0.4rem 0.75rem; font-weight: normal; letter-spacing: 0.05em;
}
.receipts-table td { padding: 0.35rem 0.75rem; border-bottom: 1px solid #0d1117; vertical-align: middle; }
.receipts-table tr:hover td { background: #0d1117; }
.row-pass td:first-child { border-left: 2px solid #00c853; }
.row-fail td:first-child { border-left: 2px solid #ff4444; }
.mono { font-family: 'Courier New', monospace; }
.small { font-size: 0.72rem; }
.num { text-align: right; color: #00f0ff; }
.dimmed { color: #555; }
.lifecycle { color: #888; font-size: 0.7rem; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.real-ue4 { color: #00c853; }
.verify-btn {
  background: none; border: 1px solid #334155; color: #64748b;
  font-size: 0.7rem; padding: 0.1rem 0.4rem; cursor: pointer; font-family: inherit;
  border-radius: 2px;
}
.verify-btn:hover:not(:disabled) { border-color: #7dd3fc; color: #7dd3fc; }
.verify-btn:disabled { opacity: 0.4; cursor: default; }
.sig-result { font-size: 0.75rem; }
.proof-links { display: flex; gap: 0.4rem; }
.proof-link {
  color: #7dd3fc; text-decoration: none; font-size: 0.7rem;
  border: 1px solid #1e3a5f; padding: 0.1rem 0.35rem; border-radius: 2px;
}
.proof-link:hover { background: #1e293b; }
.receipts-footer { margin-top: 1.5rem; font-size: 0.7rem; color: #334155; }
.receipts-footer code { color: #555; }
</style>
