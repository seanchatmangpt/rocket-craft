/**
 * server/utils/sessionStateMachine.ts
 *
 * Typestate enforcer for game_sessions lifecycle.
 * Ported from truex/src/invariants/invariant-context.js state-transition pattern.
 *
 * Legal state transitions:
 *   Created → Active → Closed → Proven  (terminal)
 *
 * Every endpoint that writes game_sessions MUST call validateSessionTransition()
 * before mutating is_alive, session_ended_at, or receipt_hash.
 */

export type SessionState = 'Created' | 'Active' | 'Closed' | 'Proven'

export interface GameSessionSnapshot {
  id: string
  is_alive: boolean
  session_ended_at: string | null
  receipt_hash: string | null
}

export interface TransitionResult {
  valid: boolean
  reason?: string
  from: SessionState
  to: SessionState
}

/** Derive current state from a session row — pure function, no DB needed */
export function deriveSessionState(session: GameSessionSnapshot): SessionState {
  if (session.receipt_hash && session.session_ended_at) return 'Proven'
  if (session.session_ended_at && !session.is_alive) return 'Closed'
  if (session.is_alive) return 'Active'
  return 'Created'
}

const ALLOWED_TRANSITIONS: Record<SessionState, SessionState[]> = {
  Created: ['Active', 'Closed'],  // Created can be directly closed (seeded sessions)
  Active:  ['Closed'],
  Closed:  ['Proven'],
  Proven:  [],                    // terminal — no further transitions
}

/**
 * Validate whether a transition from the current state to the target is legal.
 * Does NOT mutate — call this before every game_sessions write.
 */
export function validateSessionTransition(
  session: GameSessionSnapshot,
  targetState: SessionState,
): TransitionResult {
  const from = deriveSessionState(session)
  const allowed = ALLOWED_TRANSITIONS[from] ?? []

  if (allowed.includes(targetState)) {
    return { valid: true, from, to: targetState }
  }

  return {
    valid: false,
    reason: `Illegal session transition: ${from} → ${targetState}. Session ${session.id} is ${from}. Allowed: [${allowed.join(', ') || 'none'}]`,
    from,
    to: targetState,
  }
}

/**
 * Assert that a session is in one of the expected states.
 * Returns null if valid, or an error string if not.
 */
export function requireSessionState(
  session: GameSessionSnapshot,
  ...expected: SessionState[]
): string | null {
  const actual = deriveSessionState(session)
  if (expected.includes(actual)) return null
  return `Session ${session.id} is ${actual}, expected one of [${expected.join(', ')}]`
}
