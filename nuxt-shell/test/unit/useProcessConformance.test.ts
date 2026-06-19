// @vitest-environment happy-dom
/**
 * useProcessConformance.test.ts
 *
 * Tests for the Van der Aalst conformance metrics composable.
 * conformanceOf() is the pure core — testable without Supabase.
 *
 * Declared process model: GameSessionStarted → FrameRendered (→ InputAdmitted optional)
 *   fitness      = fraction of sessions with all required activities
 *   precision    = fraction of sessions with no unexpected activities
 *   simplicity   = 1.0 (single-trace model)
 *   generalization = fraction of required activities seen across all sessions
 */

import { describe, it, expect } from 'vitest'
import {
  conformanceOf,
  type SessionLifecycleSummary,
  type ConformanceResult,
} from '../../app/composables/useProcessConformance'

const computeConformance = conformanceOf

function makeSessions(overrides: Partial<SessionLifecycleSummary>[]): SessionLifecycleSummary[] {
  return overrides.map((o, i) => ({
    session_id: `session-${i}`,
    event_count: 3,
    distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
    duration_ms: 1000,
    latest_verdict: 'PASS',
    ...o,
  }))
}

describe('computeConformance — fitness', () => {
  it('empty sessions → fitness=0', () => {
    const result: ConformanceResult = computeConformance([])
    expect(result.fitness).toBe(0)
    expect(result.total_sessions).toBe(0)
    expect(result.conformant_sessions).toBe(0)
  })

  it('all sessions conformant → fitness=1.0', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] },
      { distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] },
    ])
    const result = computeConformance(sessions)
    expect(result.fitness).toBe(1.0)
    expect(result.conformant_sessions).toBe(2)
    expect(result.non_conformant).toHaveLength(0)
  })

  it('no sessions conformant → fitness=0', () => {
    const sessions = makeSessions([
      { distinct_activities: ['FrameRendered'] }, // missing GameSessionStarted
      { distinct_activities: ['GameSessionStarted'] }, // missing FrameRendered
    ])
    const result = computeConformance(sessions)
    expect(result.fitness).toBe(0)
    expect(result.conformant_sessions).toBe(0)
    expect(result.non_conformant).toHaveLength(2)
  })

  it('half sessions conformant → fitness=0.5', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] },
      { distinct_activities: ['FrameRendered'] }, // missing GameSessionStarted
    ])
    const result = computeConformance(sessions)
    expect(result.fitness).toBe(0.5)
  })
})

describe('computeConformance — non_conformant breakdown', () => {
  it('reports missing activities correctly', () => {
    const sessions = makeSessions([
      { session_id: 'bad-1', distinct_activities: ['FrameRendered', 'InputAdmitted'] },
    ])
    const result = computeConformance(sessions)
    expect(result.non_conformant).toHaveLength(1)
    expect(result.non_conformant[0]!.missing).toContain('GameSessionStarted')
    expect(result.non_conformant[0]!.session_id).toBe('bad-1')
  })

  it('reports extra (unexpected) activities', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted', 'HackAttempt'] },
    ])
    const result = computeConformance(sessions)
    // HackAttempt is not in EXPECTED_ACTIVITIES → extra
    expect(result.non_conformant[0]!.extra).toContain('HackAttempt')
  })

  it('conformant session has no non_conformant entry', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'] },
    ])
    const result = computeConformance(sessions)
    expect(result.non_conformant).toHaveLength(0)
  })
})

describe('computeConformance — simplicity and generalization', () => {
  it('simplicity is always 1.0 (single-trace declared model)', () => {
    for (const sessions of [[], makeSessions([{}]), makeSessions([{}, {}])]) {
      const result = computeConformance(sessions)
      expect(result.simplicity).toBe(1.0)
    }
  })

  it('generalization=1.0 when all required activities appear across sessions', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted'] },
      { distinct_activities: ['FrameRendered'] },
    ])
    const result = computeConformance(sessions)
    expect(result.generalization).toBe(1.0)
  })

  it('generalization=0.5 when only one required activity appears', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted'] },
    ])
    const result = computeConformance(sessions)
    // Only GameSessionStarted seen, FrameRendered missing → 1/2
    expect(result.generalization).toBe(0.5)
  })

  it('all_activities_seen contains union of all distinct_activities', () => {
    const sessions = makeSessions([
      { distinct_activities: ['GameSessionStarted', 'FrameRendered'] },
      { distinct_activities: ['InputAdmitted', 'CookStarted'] },
    ])
    const result = computeConformance(sessions)
    expect(result.all_activities_seen).toContain('GameSessionStarted')
    expect(result.all_activities_seen).toContain('FrameRendered')
    expect(result.all_activities_seen).toContain('InputAdmitted')
    expect(result.all_activities_seen).toContain('CookStarted')
  })
})
