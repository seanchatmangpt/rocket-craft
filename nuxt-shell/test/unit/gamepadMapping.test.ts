// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest';
import { mapGamepadToIntents, GAMEPAD_DEADZONE } from '../../app/composables/useRocketGamepad';

/**
 * Gamepad → RocketIntent mapping contract (was entirely untested).
 * A refactor of the deadzone or axis/button bindings would silently break
 * controller gameplay with no CI signal.
 */

const noButtons = [{ pressed: false }, { pressed: false }, { pressed: false }, { pressed: false }];

describe('mapGamepadToIntents', () => {
  it('left-stick up → MoveForward with magnitude', () => {
    const out = mapGamepadToIntents([0, -0.8], noButtons);
    expect(out).toHaveLength(1);
    expect(out[0]).toMatchObject({ type: 'MoveForward', value: 0.8, source: 'gamepad:left-stick' });
  });

  it('left-stick down → MoveBackward', () => {
    expect(mapGamepadToIntents([0, 0.6], noButtons)[0]).toMatchObject({ type: 'MoveBackward', value: 0.6 });
  });

  it('left-stick left/right → TurnLeft / TurnRight', () => {
    expect(mapGamepadToIntents([-0.9, 0], noButtons)[0]).toMatchObject({ type: 'TurnLeft', value: 0.9 });
    expect(mapGamepadToIntents([0.5, 0], noButtons)[0]).toMatchObject({ type: 'TurnRight', value: 0.5 });
  });

  it('deadzone: |axis| <= 0.25 emits nothing (drift suppression)', () => {
    expect(mapGamepadToIntents([0.25, -0.25], noButtons)).toEqual([]);
    expect(mapGamepadToIntents([0.1, -0.2], noButtons)).toEqual([]);
    expect(GAMEPAD_DEADZONE).toBe(0.25);
  });

  it('just past deadzone emits', () => {
    expect(mapGamepadToIntents([0.26, 0], noButtons)).toHaveLength(1);
  });

  it('diagonal stick emits both axes', () => {
    const out = mapGamepadToIntents([-0.5, -0.5], noButtons);
    expect(out.map((i) => i.type).sort()).toEqual(['MoveForward', 'TurnLeft']);
  });

  it('buttons A/B/Y → Interact / ExitImmersiveMode / OpenReceiptPanel', () => {
    const press = (idx: number) => noButtons.map((_, i) => ({ pressed: i === idx }));
    expect(mapGamepadToIntents([0, 0], press(0))[0]).toMatchObject({ type: 'Interact', source: 'gamepad:A' });
    expect(mapGamepadToIntents([0, 0], press(1))[0]).toMatchObject({ type: 'ExitImmersiveMode', source: 'gamepad:B' });
    expect(mapGamepadToIntents([0, 0], press(3))[0]).toMatchObject({ type: 'OpenReceiptPanel', source: 'gamepad:Y' });
  });

  it('button index 2 (X) is intentionally unmapped', () => {
    const press2 = noButtons.map((_, i) => ({ pressed: i === 2 }));
    expect(mapGamepadToIntents([0, 0], press2)).toEqual([]);
  });

  it('stick + button combine in one poll', () => {
    const out = mapGamepadToIntents([0, -0.9], [{ pressed: true }, { pressed: false }, { pressed: false }, { pressed: false }]);
    expect(out.map((i) => i.type).sort()).toEqual(['Interact', 'MoveForward']);
  });

  it('empty axes/buttons → no intents (no crash on missing data)', () => {
    expect(mapGamepadToIntents([], [])).toEqual([]);
  });
});
