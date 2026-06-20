import { describe, it, expect } from 'vitest'
import { loadFixture } from '../../src/fixtures.js'
import {
  expectGameIntentShape,
  expectNoRawBrowserEventSentToUE4,
} from '../../src/assertions.js'
import type { GameIntent } from '../../src/types.js'

describe('game intent fixtures', () => {
  it('valid intent passes shape check', () => {
    const intent = loadFixture<GameIntent>('game-intent.valid.json')
    expect(() => expectGameIntentShape(intent)).not.toThrow()
  })

  it('valid intent has ADMITTED status', () => {
    const intent = loadFixture<GameIntent>('game-intent.valid.json')
    expect(intent.status).toBe('ADMITTED')
  })

  it('raw-browser-event fixture is refused by assertion', () => {
    const intent = loadFixture<GameIntent>('game-intent.raw-browser-event.json')
    expect(() => expectNoRawBrowserEventSentToUE4([intent])).toThrow()
  })

  it('invalid-type fixture has empty type', () => {
    const intent = loadFixture<GameIntent>('game-intent.invalid-type.json')
    // Empty string type means it should be refused at admission
    expect(intent.type).toBe('')
  })

  it('intent sequence must be numeric', () => {
    const intent = loadFixture<GameIntent>('game-intent.valid.json')
    expect(typeof intent.seq).toBe('number')
    expect(intent.seq).toBeGreaterThan(0)
  })

  it('expectNoRawBrowserEventSentToUE4 passes for valid game intents', () => {
    const intents: GameIntent[] = [
      { seq: 1, type: 'Interact', source: 'dom-button', status: 'ADMITTED' },
      { seq: 2, type: 'StartWalkthrough', source: 'keyboard', status: 'ADMITTED' },
    ]
    expect(() => expectNoRawBrowserEventSentToUE4(intents)).not.toThrow()
  })
})
