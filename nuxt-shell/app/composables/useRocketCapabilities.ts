/**
 * useRocketCapabilities — detect and report browser capability status.
 *
 * Reports supported/unsupported/residual state for:
 *   speech recognition, speech synthesis, gamepad, fullscreen, PWA install.
 *
 * Used by the diagnostic panel and by Playwright to assert capability states.
 * All detection is client-only.
 */

export interface BrowserCapabilities {
  speechRecognition: boolean;
  speechSynthesis: boolean;
  gamepad: boolean;
  fullscreen: boolean;
  pwaInstallable: boolean;
  serviceWorker: boolean;
  sharedArrayBuffer: boolean;
  webgl2: boolean;
}

export function useRocketCapabilities() {
  const capabilities = ref<BrowserCapabilities>({
    speechRecognition: false,
    speechSynthesis: false,
    gamepad: false,
    fullscreen: false,
    pwaInstallable: false,
    serviceWorker: false,
    sharedArrayBuffer: false,
    webgl2: false,
  });

  onMounted(() => {
    const w = window as unknown as Record<string, unknown>;
    capabilities.value = {
      speechRecognition: !!(w['SpeechRecognition'] || w['webkitSpeechRecognition']),
      speechSynthesis: 'speechSynthesis' in window,
      gamepad: 'getGamepads' in navigator,
      fullscreen: 'requestFullscreen' in document.documentElement,
      pwaInstallable: false, // set to true when beforeinstallprompt fires
      serviceWorker: 'serviceWorker' in navigator,
      sharedArrayBuffer: typeof SharedArrayBuffer !== 'undefined',
      webgl2: (() => {
        try {
          const canvas = document.createElement('canvas');
          return !!canvas.getContext('webgl2');
        } catch { return false; }
      })(),
    };

    // Track PWA installability
    window.addEventListener('beforeinstallprompt', () => {
      capabilities.value = { ...capabilities.value, pwaInstallable: true };
    });
  });

  const unsupported = computed(() =>
    (Object.entries(capabilities.value) as [keyof BrowserCapabilities, boolean][])
      .filter(([, v]) => !v)
      .map(([k]) => k)
  );

  const allCriticalSupported = computed(() =>
    capabilities.value.webgl2 &&
    capabilities.value.serviceWorker &&
    capabilities.value.sharedArrayBuffer
  );

  return { capabilities: readonly(capabilities), unsupported, allCriticalSupported };
}
