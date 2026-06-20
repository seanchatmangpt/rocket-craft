<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Mission Control' });

// Wire keyboard intents — auto-cleaned on unmount by VueUse useEventListener
useRocketKeyboard();

// Wire touch/swipe intents for mobile — tap, swipe, long-press → RocketIntent
const canvasRef = ref<HTMLElement | null>(null);
useRocketTouchInput(canvasRef);

const { isEngineReady } = useRocketUe4Bridge();

// OCEL process-mining proof: isPlaying is derived from the event log, not a flag
const { isPlaying, events: ocelEvents, exportOcelLog, exportHashedOcelLog } = useGameSessionOcel();

// Lifecycle is the ordered list of unique activity names — used by the server to verify lawful OCEL process
const lifecycle = computed(() => [...new Set(ocelEvents.value.map(e => e.activity))]);

// Persist OCEL events to Supabase with hash chaining for pm4py conformance replay
const { syncedCount, syncError, dbSessionId, dbReceiptHash, lastHash } = useGameSessionPersistence();

// Minimum lawful OCEL lifecycle for auto-commit (Van der Aalst: process must be provable)
const PROVEN_LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] as const;

// Guard: only commit once per session, even if the watcher fires multiple times
const hasAutoCommitted = ref(false);
const lastCommitVerdict = ref<string | null>(null);

async function commitReceipt() {
  if (!dbSessionId.value || !dbReceiptHash.value) return;

  // Use receipt-finalize (chain-verified) instead of the bare receipt POST.
  // dbReceiptHash is the hash the server computed in session.post — finalize
  // verifies the chain tip matches before stamping PROVEN.
  const result = await $fetch('/api/game/receipt-finalize', {
    method: 'POST',
    body: {
      session_id: dbSessionId.value,
      receipt_hash: dbReceiptHash.value,
      update_receipt: true,
    },
  }).catch(() => null);

  if (result) {
    lastCommitVerdict.value = (result as { verdict?: string }).verdict ?? null;
    console.info(`[rocket-craft] Receipt-finalize ${lastCommitVerdict.value} — auto-committed`);
  }
  return result;
}

// Auto-commit receipt when OCEL lifecycle reaches the proven minimum.
// Fires once per session; subsequent events after proven state are ignored.
// This closes Gap 3: the Playwright receipt-persistence poll now has a real trigger.
watch(
  () => lifecycle.value,
  async (activities) => {
    if (hasAutoCommitted.value || !dbSessionId.value) return;
    const actSet = new Set(activities);
    const proven = PROVEN_LIFECYCLE.every(a => actSet.has(a));
    if (!proven) return;
    hasAutoCommitted.value = true;
    await commitReceipt();
  },
  { deep: false },
);

const engineStatus = computed(() => {
  if (isPlaying.value) return `LIVE — ${ocelEvents.value.length} events`;
  if (isEngineReady.value) return 'Engine ready — waiting for frames';
  return 'Loading engine…';
});

// Signal to Playwright that OCEL composables are mounted and listening.
// game-loop.spec.ts polls `window.__rocketOcelReady` before injecting EngineReady.
onMounted(() => {
  if (import.meta.client) {
    (window as unknown as Record<string, unknown>).__rocketOcelReady = true;
  }
});

function onEngineReady() {
  console.log('[rocket-craft] UE4 engine ready');
}

function onEngineError(msg: string) {
  console.error('[rocket-craft] UE4 engine error:', msg);
}

function downloadOcelLog() {
  const log = exportOcelLog();
  const blob = new Blob([JSON.stringify(log, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  // Embed event count in filename for forensic traceability
  a.download = `game-session-ocel-${ocelEvents.value.length}evts.json`;
  a.click();
}

async function downloadHashedOcelLog() {
  const hashedLog = await exportHashedOcelLog(Date.now());
  const blob = new Blob([JSON.stringify(hashedLog, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `game-session-ocel-hashed-${hashedLog.hashed_events.length}evts.json`;
  a.click();
}
</script>

<template>
  <main class="game-shell" :data-session-id="dbSessionId || undefined">
    <!-- DOM HUD — lives in Nuxt, not UE4 -->
    <header class="shell-header" role="banner" aria-label="Mission control header">
      <span class="brand">ROCKET-CRAFT</span>
      <span
        class="engine-status"
        :class="{ ready: isEngineReady, live: isPlaying }"
        :title="isPlaying ? 'OCEL log proves session is live' : ''"
        :data-ocel-events="ocelEvents.length"
        :data-is-playing="isPlaying ? 'true' : 'false'"
        data-testid="engine-status"
      >{{ engineStatus }}</span>
      <span
        v-if="syncedCount > 0 || syncError"
        class="sync-status"
        :class="{ error: !!syncError }"
        :title="syncError ?? `${syncedCount} events persisted to Supabase`"
      >{{ syncError ? '⚠ sync err' : `↑ ${syncedCount}` }}</span>
      <nav class="shell-nav" aria-label="Shell navigation">
        <button
          v-if="isPlaying"
          class="ocel-export"
          title="Export OCEL log (plain)"
          data-testid="ocel-export-btn"
          @click="downloadOcelLog"
        >↓ OCEL</button>
        <button
          v-if="isPlaying"
          class="ocel-export"
          title="Export OCEL log with BLAKE3 hash chain + Merkle root"
          data-testid="ocel-export-hashed-btn"
          @click="downloadHashedOcelLog"
        >↓ OCEL+BLAKE3</button>
        <button
          v-if="isPlaying && dbSessionId"
          class="receipt-commit"
          title="Commit session receipt — server verifies OCEL lifecycle"
          data-testid="receipt-commit-btn"
          @click="commitReceipt"
        >✓ Commit</button>
        <NuxtLink to="/receipts">Receipts</NuxtLink>
        <NuxtLink to="/leaderboard">Leaderboard</NuxtLink>
        <NuxtLink to="/pipeline">Pipeline</NuxtLink>
        <NuxtLink to="/profile">Profile</NuxtLink>
        <NuxtLink to="/login">Auth</NuxtLink>
      </nav>
    </header>

    <!-- UE4 canvas — touch target + browser-only rendering -->
    <div ref="canvasRef" class="canvas-wrapper">
    <ClientOnly>
      <UE4Canvas
        src="/manufactured/Brm.html"
        title="Rocket-Craft World"
        @ready="onEngineReady"
        @error="onEngineError"
      />
      <template #fallback>
        <div class="canvas-loading" role="status" aria-label="Loading game engine">
          Loading Rocket-Craft…
        </div>
      </template>
    </ClientOnly>
    </div>

    <!-- DOM game controls — accessible, Playwright-testable -->
    <section class="game-controls" aria-label="Game controls">
      <ClientOnly>
        <GameControlPanel />
      </ClientOnly>
    </section>

    <!-- Receipt audit drawer — opened by OpenReceiptPanel intent -->
    <ClientOnly>
      <ReceiptDrawer />
    </ClientOnly>
  </main>
</template>

<style scoped>
.game-shell {
  display: flex;
  flex-direction: column;
  height: 100dvh;
  background: #0b0f19;
  color: #e0e0e0;
  font-family: 'Courier New', monospace;
}
.shell-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
  background: #0d1117;
  border-bottom: 1px solid #1e3a5f;
  flex-shrink: 0;
}
.brand { font-weight: bold; color: #00f0ff; letter-spacing: 2px; }
.engine-status { font-size: 0.75rem; color: #666; }
.engine-status.ready { color: #00c853; }
.engine-status.live { color: #00f0ff; font-weight: bold; }
.sync-status { font-size: 0.7rem; color: #00c853; opacity: 0.7; }
.sync-status.error { color: #ff4444; }
.ocel-export {
  background: none; border: 1px solid #00f0ff; color: #00f0ff;
  font-size: 0.75rem; padding: 0.15rem 0.5rem; cursor: pointer; border-radius: 2px;
}
.shell-nav { margin-left: auto; display: flex; gap: 1rem; }
.shell-nav a { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.canvas-loading {
  display: flex; align-items: center; justify-content: center;
  aspect-ratio: 16/9; background: #111; color: #666;
}
.game-controls { flex-shrink: 0; }
</style>
