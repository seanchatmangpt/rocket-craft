// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest';
import { swipeToIntent, normalizeSwipeVelocity, type SwipeDirection } from '../../app/composables/useRocketTouchInput';

/**
 * Touch gesture → RocketIntent mapping + velocity normalization (was untested).
 * Completes input-device coverage (keyboard, gamepad, bridge already guarded).
 */

describe('swipeToIntent', () => {
  it('maps the four swipe directions per the GDD combat grammar', () => {
    expect(swipeToIntent('right', 0.7)).toMatchObject({ type: 'MoveForward', value: 0.7, source: 'touch' });
    expect(swipeToIntent('left', 0.4)).toMatchObject({ type: 'MoveBackward', value: 0.4, source: 'touch' });
    expect(swipeToIntent('up', 0)).toMatchObject({ type: 'Interact', source: 'touch' });
    expect(swipeToIntent('down', 0)).toMatchObject({ type: 'ExitImmersiveMode', source: 'touch' });
  });

  it('returns null for no recognized direction', () => {
    expect(swipeToIntent('none', 0.5)).toBeNull();
    expect(swipeToIntent('' as SwipeDirection, 0.5)).toBeNull();
  });

  it('carries velocity only on the movement verbs', () => {
    expect(swipeToIntent('right', 0.9)).toHaveProperty('value', 0.9);
    // Interact/Exit have no value field (discrete verbs)
    expect(swipeToIntent('up', 0.9)).not.toHaveProperty('value');
  });
});

describe('normalizeSwipeVelocity', () => {
  it('normalizes px/s to 0–1 against a 1000 px/s ceiling', () => {
    // 100px in 200ms = 500 px/s → 0.5
    expect(normalizeSwipeVelocity(100, 0, 200)).toBeCloseTo(0.5, 5);
    // diagonal: 300px in 300ms ≈ 1000 px/s → clamps to 1
    expect(normalizeSwipeVelocity(180, 240, 300)).toBe(1); // sqrt(180²+240²)=300
  });

  it('clamps fast swipes to 1', () => {
    expect(normalizeSwipeVelocity(2000, 0, 100)).toBe(1); // 20000 px/s
  });

  it('guards divide-by-zero: non-positive duration → 0', () => {
    expect(normalizeSwipeVelocity(100, 100, 0)).toBe(0);
    expect(normalizeSwipeVelocity(100, 100, -5)).toBe(0);
  });

  it('zero distance → 0 velocity', () => {
    expect(normalizeSwipeVelocity(0, 0, 200)).toBe(0);
  });
});
