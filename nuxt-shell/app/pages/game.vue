<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Mission Control' });

// Wire keyboard intents — auto-cleaned on unmount by VueUse useEventListener
useRocketKeyboard();

const { isEngineReady } = useRocketUe4Bridge();
const engineStatus = computed(() =>
  isEngineReady.value ? 'Engine ready' : 'Loading engine…'
);

function onEngineReady() {
  console.log('[rocket-craft] UE4 engine ready');
}

function onEngineError(msg: string) {
  console.error('[rocket-craft] UE4 engine error:', msg);
}
</script>

<template>
  <main class="game-shell">
    <!-- DOM HUD — lives in Nuxt, not UE4 -->
    <header class="shell-header" role="banner" aria-label="Mission control header">
      <span class="brand">ROCKET-CRAFT</span>
      <span class="engine-status" :class="{ ready: isEngineReady }">{{ engineStatus }}</span>
      <nav class="shell-nav" aria-label="Shell navigation">
        <NuxtLink to="/receipt">Receipts</NuxtLink>
        <NuxtLink to="/profile">Profile</NuxtLink>
        <NuxtLink to="/login">Auth</NuxtLink>
      </nav>
    </header>

    <!-- UE4 canvas — browser-only, no SSR -->
    <ClientOnly>
      <UE4Canvas
        script-src="/manufactured/Brm.UE4.js"
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
.shell-nav { margin-left: auto; display: flex; gap: 1rem; }
.shell-nav a { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.canvas-loading {
  display: flex; align-items: center; justify-content: center;
  aspect-ratio: 16/9; background: #111; color: #666;
}
.game-controls { flex-shrink: 0; }
</style>
