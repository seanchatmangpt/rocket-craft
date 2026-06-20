<script setup lang="ts">
/**
 * UE4Canvas — embeds a real UE4 HTML5 game build inside the Nuxt shell.
 *
 * Why an iframe (not script injection):
 * The UE4 emscripten loader (`Brm.UE4.js`) resolves asset URLs via
 * `Module.locateFile(name)` which returns the bare name (`Brm.wasm`, `Brm.js`,
 * `Brm.data`) — i.e. RELATIVE to the document. Injected into the Nuxt page at
 * `/game`, those fetches resolve to `/game/Brm.wasm` and 404, because the dev
 * proxy only forwards `/manufactured/**` to the UE4 asset server. The loader
 * also assumes a full Brm.html DOM (jQuery, bootstrap, #canvas, progress
 * ribbons) that the Nuxt page does not provide.
 *
 * Embedding the real `/manufactured/Brm.html` in an iframe fixes both: every
 * relative asset resolves under `/manufactured/` (proxied to the asset server),
 * and the loader gets the exact DOM it expects. COOP/COEP headers flow through
 * the proxy so SharedArrayBuffer (wasm-threads) stays enabled.
 *
 * Bridge: the iframe is same-origin (served via the Nuxt proxy on :3000), so we
 * forward UE4's `rocket:ue4` CustomEvents from the iframe window up to the
 * parent window, keeping `useRocketUe4Bridge` working unchanged.
 */

const props = defineProps<{
  // URL of the real UE4 HTML5 page, served through the /manufactured proxy.
  src?: string;
  title?: string;
}>();

const emit = defineEmits<{
  ready: [];
  error: [message: string];
}>();

const gameSrc = computed(() => props.src ?? '/manufactured/Brm.html');

const iframeRef = ref<HTMLIFrameElement>();
const { canvasContainer, isFullscreen, isSupported: fullscreenSupported, toggle: toggleFullscreen } = useRocketFullscreen();
const { isEngineReady } = useRocketUe4Bridge();

let detachBridge: (() => void) | null = null;

function onIframeLoad() {
  const frame = iframeRef.value;
  if (!frame) return;

  // Same-origin (proxied): forward UE4 → bridge events from iframe to parent.
  let frameWin: Window | null = null;
  try {
    frameWin = frame.contentWindow;
  } catch (err) {
    // Cross-origin would land here — should not happen with the proxy, but
    // surface it rather than silently losing the bridge.
    emit('error', `UE4 iframe is cross-origin; bridge unavailable: ${String(err)}`);
    return;
  }
  if (!frameWin) return;

  const forward = (e: Event) => {
    const detail = (e as CustomEvent).detail;
    window.dispatchEvent(new CustomEvent('rocket:ue4', { detail }));
  };
  frameWin.addEventListener('rocket:ue4', forward as EventListener);
  detachBridge = () => frameWin?.removeEventListener('rocket:ue4', forward as EventListener);
}

watch(isEngineReady, (ready) => {
  if (ready) emit('ready');
});

onBeforeUnmount(() => {
  detachBridge?.();
});
</script>

<template>
  <div
    ref="canvasContainer"
    class="ue4-canvas-wrapper"
    :aria-label="title ?? 'UE4 Game Canvas'"
    role="application"
  >
    <iframe
      ref="iframeRef"
      :src="gameSrc"
      :title="title ?? 'Rocket-Craft World'"
      class="ue4-frame"
      allow="autoplay; fullscreen; gamepad; cross-origin-isolated"
      @load="onIframeLoad"
    />

    <div class="canvas-controls" aria-label="Canvas controls">
      <button
        v-if="fullscreenSupported"
        class="fullscreen-btn"
        :aria-label="isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'"
        @click="toggleFullscreen"
      >
        {{ isFullscreen ? '⊡' : '⊞' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.ue4-canvas-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #000;
  overflow: hidden;
}
.ue4-frame {
  width: 100%;
  height: 100%;
  border: 0;
  display: block;
}
.canvas-controls {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  z-index: 10;
}
.fullscreen-btn {
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  border: 1px solid rgba(255,255,255,0.3);
  border-radius: 4px;
  padding: 4px 8px;
  cursor: pointer;
  font-size: 1.1rem;
  line-height: 1;
}
.fullscreen-btn:hover {
  background: rgba(0, 0, 0, 0.85);
}
</style>
