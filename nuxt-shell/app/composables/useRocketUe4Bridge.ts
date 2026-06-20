/**
 * useRocketUe4Bridge — sends admitted RocketIntents to the UE4 canvas.
 *
 * Law: UE4 receives only admitted projection commands from the intent bus.
 * Raw browser events never reach UE4 directly.
 *
 * UE4 → Browser: window.dispatchEvent(new CustomEvent('rocket:ue4', { detail: ProjectionEvent }))
 * Browser → UE4: Module.UE4_ExecuteJavascript('rocketIntentReceiver(json)')
 */

export type ProjectionEvent =
  | { type: 'EngineReady' }
  | { type: 'FrameRendered'; frame_ts_ms?: number; source?: string }
  | { type: 'WalkthroughStateChanged'; station: string; receipt?: string }
  | { type: 'StationFocused'; stationId: string }
  | { type: 'ReceiptEmitted'; receipt: string; payload: unknown }
  | { type: 'DiagnosticUpdate'; diagnostics: Record<string, unknown> }
  | { type: 'EngineError'; message: string };

/**
 * Build the JS string sent to UE4 via Module.UE4_ExecuteJavascript. An instrumented
 * UE4 build defines `rocketIntentReceiver`; the guard makes it a SAFE no-op against
 * the stock build (which has none). Exported so the delivery contract is tested by
 * eval-ing it against a mock receiver — guards that admitted input actually reaches
 * the engine when a receiver exists.
 */
export function buildRocketIntentJs(
  intent: import('./useRocketInputBus').RocketIntent,
  seq: number,
): string {
  const payload = JSON.stringify({ seq, intent });
  return `if(typeof rocketIntentReceiver==='function'){rocketIntentReceiver(${payload});}`;
}

export function useRocketUe4Bridge() {
  const isEngineReady = ref(false);
  const lastProjectionEvent = ref<ProjectionEvent | null>(null);

  // Listen for projection events FROM UE4 via DOM CustomEvent
  useEventListener(window, 'rocket:ue4', (e: Event) => {
    const event = (e as CustomEvent<ProjectionEvent>).detail;
    lastProjectionEvent.value = event;
    if (event.type === 'EngineReady') {
      isEngineReady.value = true;
    }
  });

  // Watch intent bus and forward to UE4 when engine is ready
  const { lastIntent } = useRocketInputBus();
  watch(lastIntent, (intent) => {
    if (!intent || !isEngineReady.value) return;
    forwardToUe4(intent);
  });

  function forwardToUe4(intent: import('./useRocketInputBus').RocketIntent) {
    if (!import.meta.client) return;
    const ue4Module = (window as unknown as Record<string, unknown>)['Module'] as
      | { UE4_ExecuteJavascript?: (js: string) => void }
      | undefined;
    if (!ue4Module?.UE4_ExecuteJavascript) return;
    try {
      ue4Module.UE4_ExecuteJavascript(buildRocketIntentJs(intent, Date.now()));
    } catch {
      // Engine may not be ready yet; intent is dropped gracefully
    }
  }

  return { isEngineReady: readonly(isEngineReady), lastProjectionEvent: readonly(lastProjectionEvent) };
}
