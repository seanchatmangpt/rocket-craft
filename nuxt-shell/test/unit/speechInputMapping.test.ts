// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest';
import { phraseToIntent } from '../../app/composables/useRocketSpeechInput';

/**
 * Voice command → RocketIntent vocabulary (was untested). Completes input coverage:
 * keyboard, gamepad, touch, bridge delivery, and now speech.
 */

describe('phraseToIntent', () => {
  it('maps each voice command to its intent', () => {
    expect(phraseToIntent('start walkthrough')).toMatchObject({ type: 'StartWalkthrough', source: 'speech' });
    expect(phraseToIntent('pause walkthrough')).toMatchObject({ type: 'PauseWalkthrough' });
    expect(phraseToIntent('resume walkthrough')).toMatchObject({ type: 'ResumeWalkthrough' });
    expect(phraseToIntent('open receipt')).toMatchObject({ type: 'OpenReceiptPanel' });
    expect(phraseToIntent('next station')).toMatchObject({ type: 'NextStation' });
    expect(phraseToIntent('previous station')).toMatchObject({ type: 'PreviousStation' });
    expect(phraseToIntent('interact')).toMatchObject({ type: 'Interact' });
    expect(phraseToIntent('inspect')).toMatchObject({ type: 'Interact' }); // synonym
  });

  it('is case- and whitespace-insensitive and matches within a sentence', () => {
    expect(phraseToIntent('  PLEASE Start Walkthrough now ')).toMatchObject({ type: 'StartWalkthrough' });
    expect(phraseToIntent('could you open receipt please')).toMatchObject({ type: 'OpenReceiptPanel' });
  });

  it('returns null for empty / whitespace / unrecognized speech', () => {
    expect(phraseToIntent('')).toBeNull();
    expect(phraseToIntent('   ')).toBeNull();
    expect(phraseToIntent('make me a sandwich')).toBeNull();
  });

  it('first match wins — "start walkthrough" before generic words', () => {
    // contains both "start walkthrough" and "interact"; start walkthrough is checked first
    expect(phraseToIntent('start walkthrough then interact')).toMatchObject({ type: 'StartWalkthrough' });
  });

  it('"next station" is not shadowed by "previous station"', () => {
    expect(phraseToIntent('go to next station')).toMatchObject({ type: 'NextStation' });
    expect(phraseToIntent('go to previous station')).toMatchObject({ type: 'PreviousStation' });
  });
});
