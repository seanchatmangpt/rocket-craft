<script setup lang="ts">
/**
 * GameControlPanel — DOM game controls using Nuxt UI components.
 *
 * Law: Game inputs are Playwright-testable DOM buttons. UE4 does not own this surface.
 * All clicks emit RocketIntents via the admitted bus — UE4 receives them as projections.
 */

const { emit } = useRocketInputBus();
const { lastIntent } = useRocketInputBus();

const toast = useToast();

function startWalkthrough() {
  emit({ type: 'StartWalkthrough', source: 'dom-button:start-walkthrough' });
  toast.add({ title: 'Walkthrough started', color: 'success', duration: 2000 });
}

function pauseWalkthrough() {
  emit({ type: 'PauseWalkthrough', source: 'dom-button:pause' });
}

function nextStation() {
  emit({ type: 'NextStation', source: 'dom-button:next-station' });
}

function openReceipts() {
  emit({ type: 'OpenReceiptPanel', source: 'dom-button:receipts' });
}

function interact() {
  emit({ type: 'Interact', source: 'dom-button:interact' });
}

const lastIntentLabel = computed(() =>
  lastIntent.value ? `${lastIntent.value.type} ← ${lastIntent.value.source}` : '—'
);
</script>

<template>
  <div class="game-control-panel" role="toolbar" aria-label="Game controls">
    <div class="controls-row">
      <!-- Primary walkthrough controls -->
      <UButton
        id="btn-start-walkthrough"
        color="primary"
        icon="i-heroicons-play"
        aria-label="Start walkthrough"
        data-testid="btn-start-walkthrough"
        @click="startWalkthrough"
      >
        Start Walkthrough
      </UButton>

      <UButton
        id="btn-pause"
        color="neutral"
        variant="outline"
        icon="i-heroicons-pause"
        aria-label="Pause walkthrough"
        data-testid="btn-pause"
        @click="pauseWalkthrough"
      >
        Pause
      </UButton>

      <UButton
        id="btn-next-station"
        color="neutral"
        variant="soft"
        icon="i-heroicons-forward"
        aria-label="Next station"
        data-testid="btn-next-station"
        @click="nextStation"
      >
        Next Station
      </UButton>

      <UButton
        id="btn-interact"
        color="secondary"
        icon="i-heroicons-cursor-arrow-rays"
        aria-label="Interact"
        data-testid="btn-interact"
        @click="interact"
      >
        Interact <UKbd>E</UKbd>
      </UButton>

      <UButton
        id="btn-receipts"
        color="warning"
        variant="soft"
        icon="i-heroicons-document-check"
        aria-label="Open receipt panel"
        data-testid="btn-receipts"
        @click="openReceipts"
      >
        Receipts <UKbd>R</UKbd>
      </UButton>
    </div>

    <!-- Intent debug display — visible in dev, useful for Playwright assertions -->
    <div
      class="intent-debug"
      aria-live="polite"
      aria-label="Last admitted intent"
      data-testid="last-intent"
    >
      {{ lastIntentLabel }}
    </div>
  </div>
</template>

<style scoped>
.game-control-panel {
  padding: 0.5rem 1rem;
  background: #0d1117;
  border-top: 1px solid #1e3a5f;
}
.controls-row {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  align-items: center;
}
.intent-debug {
  margin-top: 0.25rem;
  font-size: 0.65rem;
  color: #444;
  font-family: monospace;
}
</style>
