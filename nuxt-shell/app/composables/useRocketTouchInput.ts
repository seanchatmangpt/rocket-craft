/**
 * useRocketTouchInput — maps touch swipes and taps → RocketIntent.
 *
 * Ported from ~/seth/vue-storybook-test/src/composables/useMobileInteractions.ts
 * Adapted to the rocket-craft intent vocabulary (GDD: swipe = combat verb).
 *
 * Gesture → Intent mapping (per GDD combat grammar):
 *   Swipe right  → MoveForward (value = normalized velocity 0–1)
 *   Swipe left   → MoveBackward
 *   Swipe up     → Interact (upward lunge/block)
 *   Swipe down   → ExitImmersiveMode (pause/retreat)
 *   Tap          → Interact (tap = quick attack / confirm)
 *   Long press   → OpenReceiptPanel
 *
 * Haptics: 10ms vibration on each admitted intent (VueUse useVibrate).
 */

import { useSwipe, useVibrate, useMediaQuery, usePointerSwipe } from '@vueuse/core';
import type { RocketIntent } from './useRocketInputBus';

export type SwipeDirection = 'left' | 'right' | 'up' | 'down' | 'none';

/**
 * Normalize swipe velocity to 0–1 against a 1000 px/s ceiling. Pure + tested so
 * the combat-verb intensity can't silently change. durationMs<=0 → 0 (guards /0).
 */
export function normalizeSwipeVelocity(lengthX: number, lengthY: number, durationMs: number): number {
  if (durationMs <= 0) return 0;
  const distPx = Math.sqrt(lengthX ** 2 + lengthY ** 2);
  return Math.min(1, (distPx / (durationMs / 1000)) / 1000);
}

/**
 * Pure gesture→intent mapping (GDD combat grammar). Extracted from useSwipe's
 * callback so the bindings are unit-tested without a touch device.
 *   right→MoveForward, left→MoveBackward, up→Interact, down→ExitImmersiveMode
 */
export function swipeToIntent(direction: SwipeDirection, velocityNorm: number): RocketIntent | null {
  switch (direction) {
    case 'right': return { type: 'MoveForward', value: velocityNorm, source: 'touch' };
    case 'left': return { type: 'MoveBackward', value: velocityNorm, source: 'touch' };
    case 'up': return { type: 'Interact', source: 'touch' };
    case 'down': return { type: 'ExitImmersiveMode', source: 'touch' };
    default: return null;
  }
}

export interface TouchInputOptions {
  /** Minimum swipe distance in px to register (default 40) */
  minSwipeDistance?: number;
  /** Enable haptic feedback via Navigator.vibrate (default true) */
  enableHaptics?: boolean;
  /** Long-press threshold in ms (default 600) */
  longPressMs?: number;
}

export function useRocketTouchInput(
  target: Ref<HTMLElement | null | undefined>,
  options: TouchInputOptions = {},
) {
  const { minSwipeDistance = 40, enableHaptics = true, longPressMs = 600 } = options;

  const { emit } = useRocketInputBus();
  const isTouch = useMediaQuery('(hover: none) and (pointer: coarse)');
  const { vibrate } = useVibrate();

  function haptic(pattern: number | number[] = 10) {
    if (enableHaptics && isTouch.value) vibrate(pattern);
  }

  // ── Swipe detection ────────────────────────────────────────────────────────

  let swipeStartTime = 0;

  const { direction: swipeDir, lengthX, lengthY, isSwiping } = useSwipe(target, {
    threshold: minSwipeDistance,
    onSwipeStart() {
      swipeStartTime = Date.now();
    },
    onSwipeEnd(_e, direction) {
      const durationMs = Date.now() - swipeStartTime;
      const velocityNorm = normalizeSwipeVelocity(lengthX.value, lengthY.value, durationMs);
      const intent = swipeToIntent(direction as SwipeDirection, velocityNorm);
      if (!intent) return;
      emit(intent);
      // Haptic pattern varies by gesture (kept inline — device-only side effect).
      const patterns: Record<string, number | number[]> = {
        MoveForward: 10, MoveBackward: 10, Interact: [10, 5, 10], ExitImmersiveMode: 20,
      };
      haptic(patterns[intent.type] ?? 10);
    },
  });

  // ── Tap and long-press detection ──────────────────────────────────────────

  let tapTimer: ReturnType<typeof setTimeout> | null = null;
  let pressTimer: ReturnType<typeof setTimeout> | null = null;
  let touchMoved = false;

  function onTouchStart() {
    touchMoved = false;
    pressTimer = setTimeout(() => {
      if (!touchMoved) {
        emit({ type: 'OpenReceiptPanel', source: 'touch-longpress' });
        haptic([15, 5, 15]);
      }
    }, longPressMs);
  }

  function onTouchMove() {
    touchMoved = true;
    if (pressTimer) { clearTimeout(pressTimer); pressTimer = null; }
  }

  function onTouchEnd() {
    if (pressTimer) { clearTimeout(pressTimer); pressTimer = null; }
    if (!touchMoved && !isSwiping.value) {
      // Debounce double-tap (300ms window)
      if (tapTimer) {
        clearTimeout(tapTimer);
        tapTimer = null;
        emit({ type: 'NextStation', source: 'touch-doubletap' });
        haptic([8, 4, 8]);
      } else {
        tapTimer = setTimeout(() => {
          tapTimer = null;
          emit({ type: 'Interact', source: 'touch-tap' });
          haptic(8);
        }, 300);
      }
    }
  }

  // Attach raw touch listeners for tap/long-press (swipe handled by useSwipe)
  useEventListener(target, 'touchstart', onTouchStart, { passive: true });
  useEventListener(target, 'touchmove', onTouchMove, { passive: true });
  useEventListener(target, 'touchend', onTouchEnd, { passive: true });

  // Pointer events for non-touch (mouse) fallback
  const { direction: pointerDir } = usePointerSwipe(target, {
    threshold: minSwipeDistance,
    onSwipeEnd(_e, direction) {
      // Only fire on non-touch devices (touch already handled by useSwipe)
      if (isTouch.value) return;
      if (direction === 'right') emit({ type: 'MoveForward', value: 0.5, source: 'pointer' });
      if (direction === 'left') emit({ type: 'MoveBackward', value: 0.5, source: 'pointer' });
      if (direction === 'up') emit({ type: 'Interact', source: 'pointer' });
    },
  });

  return {
    isTouch,
    isSwiping,
    swipeDir,
    pointerDir,
  };
}
