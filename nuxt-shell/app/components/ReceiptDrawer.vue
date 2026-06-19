<script setup lang="ts">
/**
 * ReceiptDrawer — sliding panel showing the admitted-intent audit trail.
 *
 * Law: receipts are immutable evidence. This component renders, never edits.
 * Opened by any component emitting { type: 'OpenReceiptPanel' } into the bus.
 */

interface IntentRecord {
  seq: number;
  type: string;
  source: string;
  timestamp: string;
}

const isOpen = ref(false);
const { lastIntent } = useRocketInputBus();

// Listen for OpenReceiptPanel intents
watch(lastIntent, (intent) => {
  if (intent?.type === 'OpenReceiptPanel') isOpen.value = true;
});

// Collect admitted intents — keep the last 100
let _seq = 0;
const receipts = ref<IntentRecord[]>([]);
watch(lastIntent, (intent) => {
  if (!intent) return;
  const source = 'source' in intent ? (intent as { source: string }).source : 'unknown';
  receipts.value = [
    { seq: ++_seq, type: intent.type, source, timestamp: new Date().toISOString() },
    ...receipts.value,
  ].slice(0, 100);
});

function clearReceipts() {
  receipts.value = [];
}

const totalCount = computed(() => receipts.value.length);
</script>

<template>
  <UDrawer v-model:open="isOpen" side="right">
    <template #header>
      <div class="drawer-header">
        <span class="drawer-title">Admitted Intents</span>
        <span class="drawer-count" aria-label="Receipt count">{{ totalCount }} admitted</span>
      </div>
    </template>

    <div class="receipt-list" role="log" aria-label="Intent audit trail" aria-live="polite">
      <div v-if="receipts.length === 0" class="empty-state" data-testid="receipt-empty">
        No intents admitted yet. Use controls or keyboard to generate events.
      </div>

      <UTimeline v-else :items="receipts.map(r => ({
        label: r.type,
        description: `${r.source} · seq:${r.seq}`,
        date: r.timestamp,
        color: 'success',
        icon: 'i-heroicons-check-circle',
      }))" data-testid="receipt-timeline" />
    </div>

    <template #footer>
      <UButton
        color="neutral"
        variant="outline"
        icon="i-heroicons-trash"
        size="sm"
        data-testid="btn-clear-receipts"
        @click="clearReceipts"
      >
        Clear
      </UButton>
      <UButton
        color="neutral"
        variant="ghost"
        size="sm"
        data-testid="btn-close-drawer"
        @click="isOpen = false"
      >
        Close
      </UButton>
    </template>
  </UDrawer>
</template>

<style scoped>
.drawer-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.drawer-title {
  font-weight: 600;
  font-size: 1rem;
  color: #00f0ff;
  font-family: 'Courier New', monospace;
  letter-spacing: 1px;
}
.drawer-count {
  font-size: 0.7rem;
  color: #666;
  font-family: 'Courier New', monospace;
}
.receipt-list {
  padding: 0.5rem 0;
  overflow-y: auto;
  flex: 1;
}
.empty-state {
  padding: 2rem 1rem;
  text-align: center;
  color: #555;
  font-size: 0.85rem;
}
</style>
