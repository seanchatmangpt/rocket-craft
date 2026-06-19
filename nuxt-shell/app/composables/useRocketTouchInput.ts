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
      // Velocity: pixels per second, normalized to 0–1 against 1000px/s ceiling
      const distPx = Math.sqrt(lengthX.value ** 2 + lengthY.value ** 2);
      const velocityNorm = Math.min(1, (distPx / (durationMs / 1000)) / 1000);

      switch (direction) {
        case 'right':
          emit({ type: 'MoveForward', value: velocityNorm, source: 'touch' });
          haptic(10);
          break;
        case 'left':
          emit({ type: 'MoveBackward', value: velocityNorm, source: 'touch' });
          haptic(10);
          break;
        case 'up':
          emit({ type: 'Interact', source: 'touch' });
          haptic([10, 5, 10]);
          break;
        case 'down':
          emit({ type: 'ExitImmersiveMode', source: 'touch' });
          haptic(20);
          break;
      }
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
