/**
 * useRocketFullscreen — browser owns fullscreen consent for the UE4 canvas container.
 *
 * Law: UE4 must not call requestFullscreen directly. The browser shell requests
 * and manages fullscreen. UE4 only knows to expand its rendering surface.
 */
export function useRocketFullscreen() {
  const canvasContainer = ref<HTMLElement | null>(null);
  const { emit } = useRocketInputBus();

  const fullscreen = useFullscreen(canvasContainer);

  // Notify UE4 via the intent bus when fullscreen mode changes
  watch(fullscreen.isFullscreen, (entering) => {
    if (entering) {
      emit({ type: 'EnterImmersiveMode', source: 'fullscreen-api' });
    } else {
      emit({ type: 'ExitImmersiveMode', source: 'fullscreen-api' });
    }
  });

  return {
    canvasContainer,
    isFullscreen: fullscreen.isFullscreen,
    isSupported: fullscreen.isSupported,
    enter: fullscreen.enter,
    exit: fullscreen.exit,
    toggle: fullscreen.toggle,
  };
}
