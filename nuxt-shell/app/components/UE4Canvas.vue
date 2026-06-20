<script setup lang="ts">
/**
 * UE4Canvas — embeds a real UE4 HTML5 game build inside the Nuxt shell AND
 * bridges its real lifecycle into the OCEL/OTEL telemetry pipeline.
 *
 * Why an iframe (not script injection):
 * The UE4 emscripten loader resolves asset URLs via Module.locateFile(name)
 * which returns the bare name (Brm.wasm/Brm.js/Brm.data) — relative to the
 * document. Injected into /game those 404 (the proxy only forwards
 * /manufactured/**). Embedding the real /manufactured/Brm.html makes every
 * relative asset resolve under /manufactured/ (proxied), and the loader gets
 * the exact DOM (jQuery, bootstrap, #canvas, progress ribbons) it expects.
 *
 * The telemetry problem this fixes:
 * The stock UE4 build does NOT dispatch the `rocket:ue4` EngineReady event the
 * bridge/OCEL composables wait for, so sessionId stayed null forever and ZERO
 * OCEL/OTEL was produced. The iframe is same-origin (served via the proxy on
 * :3000), so we reach into its emscripten Module and derive REAL signals:
 *   - EngineReady: Module.postRun fires after the UE4 engine has launched.
 *   - FrameRendered: the iframe's requestAnimationFrame is UE4's render loop;
 *     each call is genuine frame evidence (not a fabricated setInterval).
 *   - Module bridge: expose the iframe's Module on the parent window so the
 *     bridge's forwardToUe4() (UE4_ExecuteJavascript) reaches the engine.
 */

const props = defineProps<{
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

let pollHandle: number | null = null;
let frameThrottleTs = 0;
let rafHooked = false;
let engineReadyDispatched = false;

// Throttle FrameRendered to ~4/sec — real frames fire at 30-60Hz, but the OCEL
// log only needs proof of liveness, not every frame.
const FRAME_DISPATCH_INTERVAL_MS = 250;

function dispatchUe4(detail: Record<string, unknown>) {
  window.dispatchEvent(new CustomEvent('rocket:ue4', { detail }));
}

function hookRenderLoop(frameWin: Window & { requestAnimationFrame: typeof requestAnimationFrame }) {
  if (rafHooked) return;
  rafHooked = true;
  const originalRaf = frameWin.requestAnimationFrame.bind(frameWin);
  frameWin.requestAnimationFrame = function (cb: FrameRequestCallback): number {
    return originalRaf((t: number) => {
      const now = Date.now();
      if (now - frameThrottleTs >= FRAME_DISPATCH_INTERVAL_MS) {
        frameThrottleTs = now;
        // Real frame evidence — UE4's render loop is alive.
        dispatchUe4({ type: 'FrameRendered', source: 'ue4_raf', frame_ts_ms: now });
      }
      cb(t);
    });
  };
}

function onEngineReady(frameWin: Window) {
  if (engineReadyDispatched) return;
  engineReadyDispatched = true;

  // Expose the iframe's Module on the parent so the bridge can forward input.
  const mod = (frameWin as unknown as Record<string, unknown>)['Module'];
  if (mod) (window as unknown as Record<string, unknown>)['Module'] = mod;

  // Real readiness — wasm runtime launched. Drives session creation in OCEL.
  dispatchUe4({ type: 'EngineReady' });

  // Hook the render loop for genuine per-frame evidence.
  hookRenderLoop(frameWin as Window & { requestAnimationFrame: typeof requestAnimationFrame });
}

function onIframeLoad() {
  const frame = iframeRef.value;
  if (!frame) return;

  let frameWin: (Window & { Module?: { postRun?: unknown[]; calledRun?: boolean; canvas?: HTMLCanvasElement } }) | null = null;
  try {
    frameWin = frame.contentWindow as typeof frameWin;
  } catch (err) {
    emit('error', `UE4 iframe is cross-origin; telemetry bridge unavailable: ${String(err)}`);
    return;
  }
  if (!frameWin) return;

  // Poll for the emscripten Module, then register a real readiness callback.
  // wasm compile + shader warmup can take tens of seconds, so poll generously.
  let waited = 0;
  const POLL_MS = 200;
  const MAX_WAIT_MS = 120_000;
  pollHandle = window.setInterval(() => {
    waited += POLL_MS;
    const mod = frameWin?.Module;
    if (mod) {
      // Module exists. If the engine already finished launching, fire now.
      if (mod.calledRun || (mod.canvas && mod.canvas.style.display === 'block')) {
        onEngineReady(frameWin!);
        if (pollHandle) { clearInterval(pollHandle); pollHandle = null; }
        return;
      }
      // Otherwise register a postRun callback (runs when the engine launches).
      if (Array.isArray(mod.postRun)) {
        mod.postRun.push(() => onEngineReady(frameWin!));
        if (pollHandle) { clearInterval(pollHandle); pollHandle = null; }
        return;
      }
    }
    if (waited >= MAX_WAIT_MS) {
      if (pollHandle) { clearInterval(pollHandle); pollHandle = null; }
      emit('error', `UE4 engine did not initialize within ${MAX_WAIT_MS / 1000}s`);
    }
  }, POLL_MS) as unknown as number;
}

watch(isEngineReady, (ready) => {
  if (ready) emit('ready');
});

onBeforeUnmount(() => {
  if (pollHandle) { clearInterval(pollHandle); pollHandle = null; }
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
