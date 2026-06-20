/**
 * rocketBridge — admitted event contract between the browser shell and UE4.
 *
 * Law: UE4 does not receive raw browser reality. UE4 receives admitted
 * projection commands. The browser shell owns access, consent, and focus.
 * UE4 owns visual embodiment only.
 *
 * Flow:
 *   DOM/Nuxt → InputMapper → rocketBridge.send(InputIntent)
 *                                 ↓
 *                           UE4 bridge recv
 *                                 ↓
 *                    window.dispatchEvent("rocket:ue4", ProjectionEvent)
 *                                 ↓
 *                     receipt chain closed
 */

// ─── Input side (Browser → UE4) ───────────────────────────────────────────

export type InputIntent =
  | { type: 'MoveForward'; value: number }
  | { type: 'MoveBackward'; value: number }
  | { type: 'TurnLeft'; value: number }
  | { type: 'TurnRight'; value: number }
  | { type: 'Interact' }
  | { type: 'StartWalkthrough'; station?: string }
  | { type: 'PauseWalkthrough' }
  | { type: 'ResumeWalkthrough' }
  | { type: 'NextStation' }
  | { type: 'PreviousStation' }
  | { type: 'FocusStation'; stationId: string }
  | { type: 'OpenReceiptPanel' }
  | { type: 'ToggleDiagnosticOverlay' }
  | { type: 'EnterImmersiveMode' }
  | { type: 'ExitImmersiveMode' };

export interface AdmittedIntent {
  seq: number;
  intent: InputIntent;
  source: string; // e.g. "keyboard:W", "dom-button:start-walkthrough", "gamepad:axis0"
  timestamp: string; // ISO 8601
  receipt?: string; // wasm4pm receipt hash when available
}

// ─── Output side (UE4 → Browser) ──────────────────────────────────────────

export type ProjectionEvent =
  | { type: 'EngineReady' }
  | { type: 'WalkthroughStateChanged'; station: string; receipt?: string }
  | { type: 'StationFocused'; stationId: string }
  | { type: 'ReceiptEmitted'; receipt: string; payload: unknown }
  | { type: 'DiagnosticUpdate'; diagnostics: Record<string, unknown> }
  | { type: 'EngineError'; message: string };

// ─── Bridge implementation ─────────────────────────────────────────────────

let _seq = 0;

export interface RocketBridge {
  /** Send an admitted input intent to UE4. */
  send(intent: InputIntent, source?: string): AdmittedIntent;
  /** Listen for projection events from UE4. Returns an unsubscribe fn. */
  on(handler: (event: ProjectionEvent) => void): () => void;
  /** Called by UE4's JavaScript bridge to dispatch a projection event. */
  emit(event: ProjectionEvent): void;
  /** True once UE4 has signaled EngineReady. */
  readonly isReady: boolean;
}

class BridgeImpl implements RocketBridge {
  private _handlers: Set<(event: ProjectionEvent) => void> = new Set();
  private _ready = false;

  get isReady(): boolean {
    return this._ready;
  }

  send(intent: InputIntent, source = 'unknown'): AdmittedIntent {
    const admitted: AdmittedIntent = {
      seq: ++_seq,
      intent,
      source,
      timestamp: new Date().toISOString(),
    };

    // Forward to UE4 via Module['UE4_ExecuteJavascript'] when available
    const ue4Exec = (window as unknown as Record<string, unknown>)['Module'] as
      | { UE4_ExecuteJavascript?: (js: string) => void }
      | undefined;

    if (ue4Exec?.UE4_ExecuteJavascript) {
      const payload = JSON.stringify(admitted);
      try {
        ue4Exec.UE4_ExecuteJavascript(
          `if(typeof rocketIntentReceiver==='function'){rocketIntentReceiver(${payload});}`
        );
      } catch {
        // UE4 not ready — queue or drop; not a bridge failure
      }
    }

    // Dispatch as a DOM event too so Playwright can observe it
    window.dispatchEvent(
      new CustomEvent<AdmittedIntent>('rocket:intent', { detail: admitted })
    );

    return admitted;
  }

  on(handler: (event: ProjectionEvent) => void): () => void {
    this._handlers.add(handler);
    return () => this._handlers.delete(handler);
  }

  emit(event: ProjectionEvent): void {
    if (event.type === 'EngineReady') {
      this._ready = true;
    }
    this._handlers.forEach((h) => {
      try { h(event); } catch { /* handler errors must not break the bridge */ }
    });
    window.dispatchEvent(
      new CustomEvent<ProjectionEvent>('rocket:ue4', { detail: event })
    );
  }
}

/** Singleton bridge installed on `window.rocketBridge`. */
export const rocketBridge: RocketBridge = new BridgeImpl();

// Install globally so UE4's JavaScript execution context can call it
if (typeof window !== 'undefined') {
  (window as unknown as Record<string, unknown>)['rocketBridge'] = rocketBridge;
}

// ─── DOM Input Mapper ──────────────────────────────────────────────────────

const KEY_TO_INTENT: Record<string, InputIntent> = {
  w: { type: 'MoveForward', value: 1.0 },
  ArrowUp: { type: 'MoveForward', value: 1.0 },
  s: { type: 'MoveBackward', value: 1.0 },
  ArrowDown: { type: 'MoveBackward', value: 1.0 },
  a: { type: 'TurnLeft', value: 1.0 },
  ArrowLeft: { type: 'TurnLeft', value: 1.0 },
  d: { type: 'TurnRight', value: 1.0 },
  ArrowRight: { type: 'TurnRight', value: 1.0 },
  e: { type: 'Interact' },
  r: { type: 'OpenReceiptPanel' },
  ' ': { type: 'NextStation' },
};

/**
 * Wire DOM keyboard events to the bridge input mapper.
 * Call once from the app shell — not from inside UE4.
 */
export function initInputMapper(): () => void {
  const onKeydown = (e: KeyboardEvent) => {
    // Don't intercept while user is typing in a real input/textarea
    const tag = (e.target as HTMLElement).tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

    const intent = KEY_TO_INTENT[e.key];
    if (intent) {
      e.preventDefault();
      rocketBridge.send(intent, `keyboard:${e.key}`);
    }

    if (e.key === 'Escape') {
      rocketBridge.send({ type: 'ExitImmersiveMode' }, 'keyboard:Escape');
    }
  };

  document.addEventListener('keydown', onKeydown);
  return () => document.removeEventListener('keydown', onKeydown);
}

/**
 * Convenience helper: wire a DOM button to emit a specific intent.
 * Returns cleanup fn.
 */
export function bindButton(
  el: HTMLElement,
  intent: InputIntent,
  source?: string
): () => void {
  const onClick = () => rocketBridge.send(intent, source ?? `dom-button:${el.id || el.textContent}`);
  el.addEventListener('click', onClick);
  return () => el.removeEventListener('click', onClick);
}
