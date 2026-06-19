/**
 * useRocketSpeechOutput — text-to-speech for diagnostics, receipts, and NPC narration.
 *
 * Must be inside <ClientOnly>. Reports capability status for the diagnostic panel.
 */
export function useRocketSpeechOutput() {
  const text = ref('');
  const isSupported = ref(
    import.meta.client && ('speechSynthesis' in window)
  );

  if (!isSupported.value) {
    return {
      isSupported,
      speak: (_msg: string) => {},
      stop: () => {},
      isSpeaking: ref(false),
    };
  }

  const speech = useSpeechSynthesis(text, { lang: 'en-US', rate: 1, pitch: 1, volume: 1 });

  function speak(message: string) {
    text.value = message;
    speech.speak();
  }

  return {
    isSupported,
    speak,
    stop: speech.stop,
    isSpeaking: speech.isPlaying,
  };
}
