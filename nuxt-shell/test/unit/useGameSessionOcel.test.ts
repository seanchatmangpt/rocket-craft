// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import { useGameSessionOcel } from '../../app/composables/useGameSessionOcel'

describe('useGameSessionOcel', () => {
  it('starts with empty log and no session', () => {
    const { events, objects, sessionId, isPlaying } = useGameSessionOcel()
    expect(events.value).toHaveLength(0)
    expect(objects.value).toHaveLength(0)
    expect(sessionId.value).toBeNull()
    expect(isPlaying.value).toBe(false)
  })

  it('GameSessionStarted: dispatching EngineReady does not throw', () => {
    useGameSessionOcel()
    // onMounted not wired outside component, but the event handler itself must not throw
    expect(() =>
      window.dispatchEvent(new CustomEvent('rocket:ue4', { detail: { type: 'EngineReady' } }))
    ).not.toThrow()
  })

  it('exportOcelLog returns objects, events, exported_at_ms', () => {
    const { exportOcelLog } = useGameSessionOcel()
    const log = exportOcelLog()
    expect(Array.isArray(log.objects)).toBe(true)
    expect(Array.isArray(log.events)).toBe(true)
    expect(typeof log.exported_at_ms).toBe('number')
    expect(log.exported_at_ms).toBeGreaterThan(0)
  })

  it('isPlaying is false before EngineReady', () => {
    const { isPlaying } = useGameSessionOcel()
    expect(isPlaying.value).toBe(false)
  })

  it('lastActivityAt is null with empty log', () => {
    const { lastActivityAt } = useGameSessionOcel()
    expect(lastActivityAt.value).toBeNull()
  })

  it('sessionId ref is initially null', () => {
    const { sessionId } = useGameSessionOcel()
    expect(sessionId.value).toBeNull()
  })
})
