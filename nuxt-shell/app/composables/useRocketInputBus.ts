/**
 * useRocketInputBus — canonical normalized intent bus.
 *
 * Law: All browser inputs (keyboard, gamepad, speech, DOM) are normalized into
 * RocketIntent objects here. UE4 receives only admitted intents, never raw events.
 */

export type RocketIntent =
  | { type: 'MoveForward'; value: number; source: string }
  | { type: 'MoveBackward'; value: number; source: string }
  | { type: 'TurnLeft'; value: number; source: string }
  | { type: 'TurnRight'; value: number; source: string }
  | { type: 'Interact'; source: string }
  | { type: 'StartWalkthrough'; source: string; station?: string }
  | { type: 'PauseWalkthrough'; source: string }
  | { type: 'ResumeWalkthrough'; source: string }
  | { type: 'NextStation'; source: string }
  | { type: 'PreviousStation'; source: string }
  | { type: 'FocusStation'; stationId: string; source: string }
  | { type: 'InspectSocket'; socketId: string; source: string }
  | { type: 'OpenReceiptPanel'; source: string }
  | { type: 'ToggleDiagnosticOverlay'; source: string }
  | { type: 'EnterImmersiveMode'; source: string }
  | { type: 'ExitImmersiveMode'; source: string };

export interface AdmittedIntent {
  seq: number;
  intent: RocketIntent;
  timestamp: string;
}

const _bus = shallowRef<RocketIntent[]>([]);
let _seq = 0;

export function useRocketInputBus() {
  const lastIntent = computed(() => _bus.value[_bus.value.length - 1] ?? null);

  function emit(intent: RocketIntent): AdmittedIntent {
    const admitted: AdmittedIntent = {
      seq: ++_seq,
      intent,
      timestamp: new Date().toISOString(),
    };

    _bus.value = [..._bus.value.slice(-99), intent]; // keep last 100

    // Dispatch as DOM event so Playwright can observe without WebGL access
    if (import.meta.client) {
      window.dispatchEvent(
        new CustomEvent<AdmittedIntent>('rocket:intent', { detail: admitted })
      );
    }

    return admitted;
  }

  function reset() {
    _bus.value = [];
    _seq = 0;
  }

  return { lastIntent, emit, reset, history: readonly(_bus) };
}
