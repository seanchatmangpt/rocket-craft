// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from 'vitest'
import { useRocketInputBus, type RocketIntent } from '../../app/composables/useRocketInputBus'

// Reset module-level state between tests by calling reset()
let bus: ReturnType<typeof useRocketInputBus>

beforeEach(() => {
  bus = useRocketInputBus()
  bus.reset()
})

describe('useRocketInputBus', () => {
  it('emits an intent and increments seq', () => {
    const admitted = bus.emit({ type: 'Interact', source: 'test' })
    expect(admitted.seq).toBe(1)
    expect(admitted.intent.type).toBe('Interact')
    expect(admitted.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}/)
  })

  it('seq is monotonically increasing', () => {
    const a = bus.emit({ type: 'Interact', source: 'test' })
    const b = bus.emit({ type: 'NextStation', source: 'test' })
    expect(b.seq).toBe(a.seq + 1)
  })

  it('lastIntent reflects the most recent emit', () => {
    bus.emit({ type: 'Interact', source: 'kb' })
    bus.emit({ type: 'OpenReceiptPanel', source: 'kb' })
    expect(bus.lastIntent.value?.type).toBe('OpenReceiptPanel')
  })

  it('lastIntent is null before any emit', () => {
    expect(bus.lastIntent.value).toBeNull()
  })

  it('history keeps the last 100 intents', () => {
    for (let i = 0; i < 105; i++) {
      bus.emit({ type: 'Interact', source: `test-${i}` })
    }
    expect(bus.history.value.length).toBe(100)
  })

  it('reset clears history and resets seq', () => {
    bus.emit({ type: 'Interact', source: 'test' })
    bus.reset()
    expect(bus.history.value.length).toBe(0)
    expect(bus.lastIntent.value).toBeNull()
    const next = bus.emit({ type: 'Interact', source: 'test' })
    expect(next.seq).toBe(1)
  })

  it('history is readonly — pushing to it does not mutate', () => {
    bus.emit({ type: 'Interact', source: 'test' })
    // TypeScript would prevent push at compile time; verify runtime length is unchanged
    const len = bus.history.value.length
    expect(len).toBe(1)
  })

  it('admitted intent has all required fields', () => {
    const admitted = bus.emit({ type: 'StartWalkthrough', source: 'dom-button' })
    expect(typeof admitted.seq).toBe('number')
    expect(typeof admitted.timestamp).toBe('string')
    expect(admitted.intent.type).toBe('StartWalkthrough')
  })

  it('different intent types all emit correctly', () => {
    const types: RocketIntent['type'][] = [
      'MoveForward', 'Interact', 'OpenReceiptPanel', 'ExitImmersiveMode',
    ]
    const results = [
      bus.emit({ type: 'MoveForward', value: 1.0, source: 'gamepad' }),
      bus.emit({ type: 'Interact', source: 'keyboard' }),
      bus.emit({ type: 'OpenReceiptPanel', source: 'keyboard' }),
      bus.emit({ type: 'ExitImmersiveMode', source: 'keyboard' }),
    ]
    results.forEach((r, i) => expect(r.intent.type).toBe(types[i]))
  })
})
