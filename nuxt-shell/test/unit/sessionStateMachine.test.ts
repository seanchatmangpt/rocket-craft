import { describe, it, expect } from 'vitest'
import {
  deriveSessionState,
  validateSessionTransition,
  requireSessionState,
  type GameSessionSnapshot,
} from '../../server/utils/sessionStateMachine'

function makeSession(overrides: Partial<GameSessionSnapshot> = {}): GameSessionSnapshot {
  return {
    id: 'sess-abc',
    is_alive: false,
    session_ended_at: null,
    receipt_hash: null,
    ...overrides,
  }
}

describe('deriveSessionState', () => {
  it('Created: is_alive=false, no end, no receipt', () => {
    expect(deriveSessionState(makeSession())).toBe('Created')
  })

  it('Active: is_alive=true', () => {
    expect(deriveSessionState(makeSession({ is_alive: true }))).toBe('Active')
  })

  it('Closed: is_alive=false + session_ended_at set', () => {
    expect(deriveSessionState(makeSession({ session_ended_at: '2026-01-01' }))).toBe('Closed')
  })

  it('Proven: session_ended_at + receipt_hash both set', () => {
    expect(deriveSessionState(makeSession({
      session_ended_at: '2026-01-01',
      receipt_hash: 'a'.repeat(64),
    }))).toBe('Proven')
  })
})

describe('validateSessionTransition', () => {
  it('Active → Closed is valid', () => {
    const sess = makeSession({ is_alive: true })
    const r = validateSessionTransition(sess, 'Closed')
    expect(r.valid).toBe(true)
    expect(r.from).toBe('Active')
    expect(r.to).toBe('Closed')
  })

  it('Proven → Active is invalid', () => {
    const sess = makeSession({ session_ended_at: '2026-01-01', receipt_hash: 'a'.repeat(64) })
    const r = validateSessionTransition(sess, 'Active')
    expect(r.valid).toBe(false)
    expect(r.reason).toContain('Proven')
  })

  it('Created → Active is allowed', () => {
    const r = validateSessionTransition(makeSession(), 'Active')
    expect(r.valid).toBe(true)
  })

  it('Created → Closed is allowed (seeded sessions skip Active)', () => {
    const r = validateSessionTransition(makeSession(), 'Closed')
    expect(r.valid).toBe(true)
  })

  it('Closed → Proven is allowed', () => {
    const sess = makeSession({ session_ended_at: '2026-01-01' })
    const r = validateSessionTransition(sess, 'Proven')
    expect(r.valid).toBe(true)
  })

  it('Proven → anything is rejected (terminal state)', () => {
    const sess = makeSession({ session_ended_at: '2026-01-01', receipt_hash: 'a'.repeat(64) })
    for (const target of ['Created', 'Active', 'Closed', 'Proven'] as const) {
      const r = validateSessionTransition(sess, target)
      expect(r.valid).toBe(false)
    }
  })

  it('Active → Proven skips Closed — rejected', () => {
    const sess = makeSession({ is_alive: true })
    const r = validateSessionTransition(sess, 'Proven')
    expect(r.valid).toBe(false)
  })
})

describe('requireSessionState', () => {
  it('returns null when session is in expected state', () => {
    const sess = makeSession({ is_alive: true })
    expect(requireSessionState(sess, 'Active')).toBeNull()
  })

  it('returns error string when session is not in expected state', () => {
    const sess = makeSession()
    const err = requireSessionState(sess, 'Active', 'Closed')
    expect(typeof err).toBe('string')
    expect(err).toContain('Created')
  })
})
