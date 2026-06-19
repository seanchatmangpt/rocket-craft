<script setup lang="ts">
/**
 * UE4Canvas — mounts a UE4 HTML5 game build inside the Nuxt shell.
 *
 * Law: UE4 owns visual projection only. This component:
 * - mounts inside <ClientOnly> (no SSR, no hydration mismatch)
 * - suppresses requestPointerLock so UMG widgets receive first-click
 * - injects the UE4 loader script dynamically into the container div
 * - emits 'ready' when UE4 signals EngineReady via window.rocket:ue4
 */

const props = defineProps<{
  scriptSrc: string;  // e.g. '/manufactured/Brm.UE4.js'
  title?: string;
}>();

const emit = defineEmits<{
  ready: [];
  error: [message: string];
}>();

const container = ref<HTMLDivElement>();
const { canvasContainer, isFullscreen, isSupported: fullscreenSupported, toggle: toggleFullscreen } = useRocketFullscreen();
const { isEngineReady } = useRocketUe4Bridge();

onMounted(() => {
  if (!container.value) return;

  // Suppress pointer lock globally — browser shell owns access, not UE4
  suppressPointerLock();

  // Create canvas element that UE4's loader expects
  const canvas = document.createElement('canvas');
  canvas.id = 'canvas';
  canvas.className = 'emscripten';
  canvas.tabIndex = 0;
  canvas.setAttribute('oncontextmenu', 'event.preventDefault()');
  canvas.style.cssText = 'width:100%;height:100%;display:block;';
  container.value.appendChild(canvas);

  // Inject UE4 loader script
  const script = document.createElement('script');
  script.src = props.scriptSrc;
  script.async = true;
  script.onerror = () => emit('error', `Failed to load ${props.scriptSrc}`);
  container.value.appendChild(script);
});

// Watch bridge engine-ready state and emit to parent
watch(isEngineReady, (ready) => {
  if (ready) emit('ready');
});

function suppressPointerLock() {
  if (!import.meta.client) return;
  const noop = () => Promise.resolve();
  try {
    Object.defineProperty(HTMLElement.prototype, 'requestPointerLock', {
      value: noop, writable: false, configurable: false,
    });
    Object.defineProperty(Document.prototype, 'exitPointerLock', {
      value: noop, writable: false, configurable: false,
    });
  } catch {
    HTMLElement.prototype.requestPointerLock = noop;
  }
  // Auto-focus canvas on visibility so no click is wasted on focus acquisition
  const poll = setInterval(() => {
    const c = document.getElementById('canvas') as HTMLCanvasElement | null;
    if (c && c.style.display !== 'none') {
      c.focus();
      clearInterval(poll);
    }
  }, 500);
  document.addEventListener('click', () => {
    const c = document.getElementById('canvas');
    if (c) c.focus();
  });
}
</script>

<template>
  <div
    ref="canvasContainer"
    class="ue4-canvas-wrapper"
    :aria-label="title ?? 'UE4 Game Canvas'"
    role="application"
  >
    <div ref="container" class="ue4-canvas-container" />

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
.ue4-canvas-container {
  width: 100%;
  height: 100%;
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
