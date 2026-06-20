/**
 * useRocketKeyboard — maps DOM keyboard events to RocketIntents via VueUse.
 *
 * useEventListener auto-registers on mount and removes on unmount — no manual cleanup.
 * Never intercepts keystrokes while a form element has focus.
 */
import type { RocketIntent } from './useRocketInputBus';

const KEY_MAP: Record<string, RocketIntent> = {
  w:          { type: 'MoveForward',  value: 1, source: 'keyboard:w' },
  arrowup:    { type: 'MoveForward',  value: 1, source: 'keyboard:ArrowUp' },
  s:          { type: 'MoveBackward', value: 1, source: 'keyboard:s' },
  arrowdown:  { type: 'MoveBackward', value: 1, source: 'keyboard:ArrowDown' },
  a:          { type: 'TurnLeft',     value: 1, source: 'keyboard:a' },
  arrowleft:  { type: 'TurnLeft',     value: 1, source: 'keyboard:ArrowLeft' },
  d:          { type: 'TurnRight',    value: 1, source: 'keyboard:d' },
  arrowright: { type: 'TurnRight',    value: 1, source: 'keyboard:ArrowRight' },
  e:          { type: 'Interact',     source: 'keyboard:e' },
  r:          { type: 'OpenReceiptPanel', source: 'keyboard:r' },
  ' ':        { type: 'NextStation',  source: 'keyboard:Space' },
};

export function useRocketKeyboard() {
  const { emit } = useRocketInputBus();

  // useEventListener (VueUse) — auto-cleanup on component unmount
  useEventListener(window, 'keydown', (e: KeyboardEvent) => {
    if (e.repeat) return;

    // Never intercept while user is typing in a form element
    const tag = (e.target as HTMLElement)?.tagName?.toUpperCase();
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

    const intent = KEY_MAP[e.key.toLowerCase()];
    if (intent) {
      e.preventDefault();
      emit(intent);
      return;
    }

    if (e.key === 'Escape') {
      e.preventDefault();
      emit({ type: 'ExitImmersiveMode', source: 'keyboard:Escape' });
    }
  });
}
