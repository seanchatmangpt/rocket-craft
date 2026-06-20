import type { RocketIntent } from './useRocketInputBus';

/** Left-stick deadzone — drift below this magnitude is not admitted as an intent. */
export const GAMEPAD_DEADZONE = 0.25;

/**
 * Pure mapping: gamepad axes + buttons → RocketIntents. Extracted so the deadzone
 * and axis/button bindings are unit-tested without the browser Gamepad API.
 *
 *   axes[0] = left-stick X (− left / + right), axes[1] = Y (− up / + down)
 *   buttons[0]=A→Interact, [1]=B→ExitImmersiveMode, [3]=Y→OpenReceiptPanel
 */
export function mapGamepadToIntents(
  axes: readonly number[],
  buttons: readonly { pressed: boolean }[],
): RocketIntent[] {
  const intents: RocketIntent[] = [];
  const x = axes[0] ?? 0;
  const y = axes[1] ?? 0;

  if (y < -GAMEPAD_DEADZONE) intents.push({ type: 'MoveForward', value: Math.abs(y), source: 'gamepad:left-stick' });
  if (y > GAMEPAD_DEADZONE) intents.push({ type: 'MoveBackward', value: y, source: 'gamepad:left-stick' });
  if (x < -GAMEPAD_DEADZONE) intents.push({ type: 'TurnLeft', value: Math.abs(x), source: 'gamepad:left-stick' });
  if (x > GAMEPAD_DEADZONE) intents.push({ type: 'TurnRight', value: x, source: 'gamepad:left-stick' });

  if (buttons[0]?.pressed) intents.push({ type: 'Interact', source: 'gamepad:A' });
  if (buttons[1]?.pressed) intents.push({ type: 'ExitImmersiveMode', source: 'gamepad:B' });
  if (buttons[3]?.pressed) intents.push({ type: 'OpenReceiptPanel', source: 'gamepad:Y' });

  return intents;
}

/**
 * useRocketGamepad — maps browser Gamepad API to RocketIntents via VueUse.
 *
 * VueUse useGamepad polls via requestAnimationFrame internally.
 * Must be used in a <ClientOnly> context (gamepad API is browser-only).
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
    for (const intent of mapGamepadToIntents(pad.axes, pad.buttons)) emit(intent);
  });

  return { isSupported, standardPad };
}
