/**
 * useRocketSpeechInput — voice commands → RocketIntents via VueUse useSpeechRecognition.
 *
 * Must be inside <ClientOnly>. Includes capability detection — speech recognition
 * is only available in Chrome/Edge; reports 'unsupported' gracefully in Firefox/Safari.
 */
import type { RocketIntent } from './useRocketInputBus';

/**
 * Pure phrase→intent mapping for voice commands. Extracted from the recognition
 * watcher so the vocabulary is unit-tested without the Web Speech API. Case- and
 * whitespace-insensitive; first match wins; returns null for unrecognized speech.
 */
export function phraseToIntent(text: string): RocketIntent | null {
  const phrase = text.toLowerCase().trim();
  if (!phrase) return null;
  if (phrase.includes('start walkthrough')) return { type: 'StartWalkthrough', source: 'speech' };
  if (phrase.includes('pause walkthrough')) return { type: 'PauseWalkthrough', source: 'speech' };
  if (phrase.includes('resume walkthrough')) return { type: 'ResumeWalkthrough', source: 'speech' };
  if (phrase.includes('open receipt')) return { type: 'OpenReceiptPanel', source: 'speech' };
  if (phrase.includes('next station')) return { type: 'NextStation', source: 'speech' };
  if (phrase.includes('previous station')) return { type: 'PreviousStation', source: 'speech' };
  if (phrase.includes('interact') || phrase.includes('inspect')) return { type: 'Interact', source: 'speech' };
  return null;
}

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
    const intent = phraseToIntent(text);
    if (intent) emit(intent);
  });

  return { isSupported, isListening: speech.isListening, start: speech.start, stop: speech.stop };
}
