// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from 'vitest';
import { intentToKey, dispatchIntentKeyToCanvas } from '../../app/utils/intentToKey';

// Fresh DOM per test so leftover #canvas elements don't shadow getElementById.
beforeEach(() => { document.body.innerHTML = ''; });

/**
 * Intent → synthetic-key mapping. This is what makes control-plane intents
 * (panel/gamepad/voice/touch) actually drive the STOCK UE4 car via native canvas
 * keyboard input (no rocketIntentReceiver / no rebuild). A binding change here
 * would silently break game control.
 */

describe('intentToKey', () => {
  it('maps movement intents to WASD', () => {
    expect(intentToKey('MoveForward')).toMatchObject({ code: 'KeyW', key: 'w', keyCode: 87 });
    expect(intentToKey('MoveBackward')).toMatchObject({ code: 'KeyS', key: 's', keyCode: 83 });
    expect(intentToKey('TurnLeft')).toMatchObject({ code: 'KeyA', key: 'a', keyCode: 65 });
    expect(intentToKey('TurnRight')).toMatchObject({ code: 'KeyD', key: 'd', keyCode: 68 });
  });

  it('maps action intents to their UE4 keys', () => {
    expect(intentToKey('Interact')).toMatchObject({ code: 'KeyE', keyCode: 69 });
    expect(intentToKey('NextStation')).toMatchObject({ code: 'Space', key: ' ', keyCode: 32 });
    expect(intentToKey('OpenReceiptPanel')).toMatchObject({ code: 'KeyR', keyCode: 82 });
    expect(intentToKey('ExitImmersiveMode')).toMatchObject({ code: 'Escape', key: 'Escape', keyCode: 27 });
  });

  it('returns null for meta intents with no game key', () => {
    expect(intentToKey('StartWalkthrough')).toBeNull();
    expect(intentToKey('OpenReceiptPanelXYZ')).toBeNull();
    expect(intentToKey('')).toBeNull();
  });

  it('every mapped key has code, key, and a positive keyCode (emscripten reads all three)', () => {
    for (const t of ['MoveForward', 'MoveBackward', 'TurnLeft', 'TurnRight', 'Interact', 'NextStation', 'OpenReceiptPanel', 'ExitImmersiveMode']) {
      const k = intentToKey(t)!;
      expect(k.code).toMatch(/^(Key[A-Z]|Space|Escape)$/);
      expect(typeof k.key).toBe('string');
      expect(k.keyCode).toBeGreaterThan(0);
    }
  });
});

describe('dispatchIntentKeyToCanvas (intent → real key events on the canvas)', () => {
  function mockFrame() {
    // happy-dom provides document + KeyboardEvent; emulate the iframe's canvas.
    const canvas = document.createElement('div');
    canvas.id = 'canvas';
    document.body.appendChild(canvas);
    const events: Array<{ phase: string; code: string }> = [];
    for (const phase of ['keydown', 'keyup']) {
      canvas.addEventListener(phase, (e) => events.push({ phase, code: (e as KeyboardEvent).code }));
    }
    return { frameWin: { document, KeyboardEvent } as any, canvas, events };
  }

  it('dispatches keydown+keyup with the mapped code for a movement intent', () => {
    const { frameWin, events } = mockFrame();
    const ok = dispatchIntentKeyToCanvas(frameWin, 'MoveForward');
    expect(ok).toBe(true);
    expect(events).toEqual([{ phase: 'keydown', code: 'KeyW' }, { phase: 'keyup', code: 'KeyW' }]);
  });

  it('returns false and dispatches nothing for an unmapped intent', () => {
    const { frameWin, events } = mockFrame();
    expect(dispatchIntentKeyToCanvas(frameWin, 'StartWalkthrough')).toBe(false);
    expect(events).toEqual([]);
  });

  it('drives distinct keys for distinct intents (control plane → game input)', () => {
    const { frameWin, events } = mockFrame();
    dispatchIntentKeyToCanvas(frameWin, 'TurnLeft');
    dispatchIntentKeyToCanvas(frameWin, 'Interact');
    expect(events.map((e) => e.code)).toEqual(['KeyA', 'KeyA', 'KeyE', 'KeyE']);
  });
});
