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
  | { type: 'WalkthroughStateChanged'; station: string; receipt?: string }
  | { type: 'StationFocused'; stationId: string }
  | { type: 'ReceiptEmitted'; receipt: string; payload: unknown }
  | { type: 'DiagnosticUpdate'; diagnostics: Record<string, unknown> }
  | { type: 'EngineError'; message: string };

export function useRocketUe4Bridge() {
  const isEngineReady = ref(false);
  const lastProjectionEvent = ref<ProjectionEvent | null>(null);
  const { emit: emitIntent } = useRocketInputBus();

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
    const payload = JSON.stringify({ seq: Date.now(), intent });
    try {
      ue4Module.UE4_ExecuteJavascript(
        `if(typeof rocketIntentReceiver==='function'){rocketIntentReceiver(${payload});}`
      );
    } catch {
      // Engine may not be ready yet; intent is dropped gracefully
    }
  }

  return { isEngineReady: readonly(isEngineReady), lastProjectionEvent: readonly(lastProjectionEvent) };
}
