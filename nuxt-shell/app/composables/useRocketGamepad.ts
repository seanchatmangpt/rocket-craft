/**
 * useRocketGamepad — maps browser Gamepad API to RocketIntents via VueUse.
 *
 * VueUse useGamepad polls via requestAnimationFrame internally.
 * Must be used in a <ClientOnly> context (gamepad API is browser-only).
 * Deadzone of 0.25 prevents drift noise from being admitted as intents.
 */
export function useRocketGamepad() {
  const { emit } = useRocketInputBus();
  const { isSupported, gamepads } = useGamepad();

  const standardPad = computed(() =>
    gamepads.value.find((g) => g.mapping === 'standard') ?? null
  );

  watchEffect(() => {
    const pad = standardPad.value;
    if (!isSupported.value || !pad) return;

    const x = pad.axes[0] ?? 0;
    const y = pad.axes[1] ?? 0;

    if (y < -0.25) emit({ type: 'MoveForward',  value: Math.abs(y), source: 'gamepad:left-stick' });
    if (y >  0.25) emit({ type: 'MoveBackward', value: y,           source: 'gamepad:left-stick' });
    if (x < -0.25) emit({ type: 'TurnLeft',     value: Math.abs(x), source: 'gamepad:left-stick' });
    if (x >  0.25) emit({ type: 'TurnRight',    value: x,           source: 'gamepad:left-stick' });

    // Button 0 = A/Cross → Interact
    if (pad.buttons[0]?.pressed) emit({ type: 'Interact', source: 'gamepad:A' });
    // Button 1 = B/Circle → ExitImmersiveMode
    if (pad.buttons[1]?.pressed) emit({ type: 'ExitImmersiveMode', source: 'gamepad:B' });
    // Button 3 = Y/Triangle → OpenReceiptPanel
    if (pad.buttons[3]?.pressed) emit({ type: 'OpenReceiptPanel', source: 'gamepad:Y' });
  });

  return { isSupported, standardPad };
}
