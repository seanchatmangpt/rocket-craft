/**
 * useRocketSpeechInput — voice commands → RocketIntents via VueUse useSpeechRecognition.
 *
 * Must be inside <ClientOnly>. Includes capability detection — speech recognition
 * is only available in Chrome/Edge; reports 'unsupported' gracefully in Firefox/Safari.
 */
export function useRocketSpeechInput() {
  const { emit } = useRocketInputBus();

  const w = typeof window !== 'undefined' ? window as unknown as Record<string, unknown> : null;
  const isSupported = useSpeechRecognition !== undefined && w
    ? ref(!!(w['SpeechRecognition'] || w['webkitSpeechRecognition']))
    : ref(false);

  if (!isSupported.value) {
    return { isSupported, isListening: ref(false), start: () => {}, stop: () => {} };
  }

  const speech = useSpeechRecognition({ lang: 'en-US', continuous: true, interimResults: false });

  watch(speech.result, (text) => {
    const phrase = text.toLowerCase().trim();
    if (!phrase) return;

    if (phrase.includes('start walkthrough'))
      emit({ type: 'StartWalkthrough', source: 'speech' });
    else if (phrase.includes('pause walkthrough'))
      emit({ type: 'PauseWalkthrough', source: 'speech' });
    else if (phrase.includes('resume walkthrough'))
      emit({ type: 'ResumeWalkthrough', source: 'speech' });
    else if (phrase.includes('open receipt'))
      emit({ type: 'OpenReceiptPanel', source: 'speech' });
    else if (phrase.includes('next station'))
      emit({ type: 'NextStation', source: 'speech' });
    else if (phrase.includes('previous station'))
      emit({ type: 'PreviousStation', source: 'speech' });
    else if (phrase.includes('interact') || phrase.includes('inspect'))
      emit({ type: 'Interact', source: 'speech' });
  });

  return { isSupported, isListening: speech.isListening, start: speech.start, stop: speech.stop };
}
