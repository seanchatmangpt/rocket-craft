// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest';
import { buildRocketIntentJs } from '../../app/composables/useRocketUe4Bridge';
import type { RocketIntent } from '../../app/composables/useRocketInputBus';

/**
 * UE4 bridge delivery contract.
 *
 * The bridge forwards admitted intents to the engine via
 * Module.UE4_ExecuteJavascript(buildRocketIntentJs(...)). The stock Brm build has
 * no `rocketIntentReceiver`, so it's a guarded no-op there — but an INSTRUMENTED
 * build defines it, and this test proves that when the receiver exists, the
 * generated JS actually delivers the intent (not just that a string was built).
 *
 * Without this, a payload-format regression would silently drop ALL input on a
 * real instrumented build ("a car I cannot control") with no test failing.
 */

// Run the generated JS the way UE4_ExecuteJavascript would (function scope eval),
// with a provided rocketIntentReceiver in scope.
function runDelivery(js: string, receiver: ((arg: unknown) => void) | undefined): void {
  // eslint-disable-next-line no-new-func
  const fn = new Function('rocketIntentReceiver', js);
  fn(receiver);
}

describe('buildRocketIntentJs delivery contract', () => {
  const intent: RocketIntent = { type: 'MoveForward', value: 0.8, source: 'keyboard:w' };

  it('invokes rocketIntentReceiver with the seq + intent payload when present', () => {
    let received: { seq: number; intent: RocketIntent } | null = null;
    runDelivery(buildRocketIntentJs(intent, 42), (arg) => { received = arg as typeof received; });
    expect(received).not.toBeNull();
    expect(received!.seq).toBe(42);
    expect(received!.intent.type).toBe('MoveForward');
    expect((received!.intent as { value?: number }).value).toBe(0.8);
    expect(received!.intent.source).toBe('keyboard:w');
  });

  it('is a SAFE no-op when rocketIntentReceiver is undefined (stock build)', () => {
    // Must NOT throw — the stock Brm build has no receiver.
    expect(() => runDelivery(buildRocketIntentJs(intent, 1), undefined)).not.toThrow();
  });

  it('produces valid JS for every intent type without injection', () => {
    const types: RocketIntent[] = [
      { type: 'Interact', source: 's' },
      { type: 'NextStation', source: 's' },
      { type: 'OpenReceiptPanel', source: 's' },
      // An adversarial source string must not break out of the JSON literal.
      { type: 'MoveForward', value: 1, source: `evil");globalThis.__pwned=1;//` },
    ];
    for (const t of types) {
      let got: { intent: RocketIntent } | null = null;
      runDelivery(buildRocketIntentJs(t, 1), (a) => { got = a as typeof got; });
      expect(got!.intent.type).toBe(t.type);
    }
    // The injection attempt must NOT have executed.
    expect((globalThis as Record<string, unknown>).__pwned).toBeUndefined();
  });
});
