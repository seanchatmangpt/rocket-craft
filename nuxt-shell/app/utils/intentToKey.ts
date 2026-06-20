/**
 * RocketIntent → synthetic keyboard key mapping.
 *
 * The stock UE4 HTML5 build has no `rocketIntentReceiver`, so control-plane
 * intents (panel buttons, gamepad, voice, touch) were a no-op against it — the
 * "car I cannot control" gap. But UE4's emscripten runtime DOES process native
 * keyboard events on its canvas. Translating intents into synthetic KeyboardEvents
 * dispatched to the iframe canvas makes every admitted intent actually drive the
 * stock car, with no rebuild — using the standard WASD + E/Space/R/Esc bindings.
 *
 * Pure + tested so the bindings can't silently break. Returns null for intents
 * that have no movement/action key (e.g. walkthrough meta-intents).
 */
export interface SyntheticKey {
  /** KeyboardEvent.code (layout-independent; what emscripten/UE4 reads). */
  code: string;
  /** KeyboardEvent.key (character value). */
  key: string;
  /** keyCode/which (legacy; some emscripten input paths still read it). */
  keyCode: number;
}

const INTENT_KEY_MAP: Record<string, SyntheticKey> = {
  MoveForward: { code: 'KeyW', key: 'w', keyCode: 87 },
  MoveBackward: { code: 'KeyS', key: 's', keyCode: 83 },
  TurnLeft: { code: 'KeyA', key: 'a', keyCode: 65 },
  TurnRight: { code: 'KeyD', key: 'd', keyCode: 68 },
  Interact: { code: 'KeyE', key: 'e', keyCode: 69 },
  NextStation: { code: 'Space', key: ' ', keyCode: 32 },
  OpenReceiptPanel: { code: 'KeyR', key: 'r', keyCode: 82 },
  ExitImmersiveMode: { code: 'Escape', key: 'Escape', keyCode: 27 },
};

export function intentToKey(intentType: string): SyntheticKey | null {
  return INTENT_KEY_MAP[intentType] ?? null;
}

/** Minimal shape of the iframe window we need — keeps this unit-testable. */
export interface KeyDispatchWindow {
  document: Document;
  KeyboardEvent: typeof KeyboardEvent;
}

/**
 * Translate an intent to a synthetic key and dispatch it (keydown+keyup) to the
 * UE4 canvas (and document) inside the given window. Returns true if a key was
 * dispatched, false for unmapped intents. This is what actually drives the stock
 * car from control-plane intents.
 */
export function dispatchIntentKeyToCanvas(frameWin: KeyDispatchWindow, intentType: string): boolean {
  const k = intentToKey(intentType);
  if (!k) return false;
  const doc = frameWin.document;
  const canvas = doc.getElementById('canvas') ?? doc.body;
  if (!canvas) return false;
  for (const phase of ['keydown', 'keyup'] as const) {
    const ev = new frameWin.KeyboardEvent(phase, {
      code: k.code, key: k.key, keyCode: k.keyCode, which: k.keyCode,
      bubbles: true, cancelable: true,
    });
    canvas.dispatchEvent(ev);
    doc.dispatchEvent(ev);
  }
  return true;
}
