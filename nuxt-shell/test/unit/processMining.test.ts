import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  discoverDfg,
  computeFitness,
  computePrecision,
  computeSimplicity,
  computeGeneralization,
  checkConformance,
  type MiningEvent,
} from '../../server/utils/processMining'

const LAWFUL = ['GameSessionStarted', 'InputAdmitted', 'FrameRendered', 'GameSessionClosed']

function events(...activities: string[]): MiningEvent[] {
  return activities.map((activity, i) => ({ activity, timestamp_ms: 1_000_000 + i * 200, seq: i }))
}

// ── discoverDfg ───────────────────────────────────────────────────────────────

describe('discoverDfg', () => {
  it('single event: no edges, one start/end activity', () => {
    const dfg = discoverDfg(events('A'))
    expect(dfg.edges.size).toBe(0)
    expect(dfg.startActivities.get('A')).toBe(1)
    expect(dfg.endActivities.get('A')).toBe(1)
    expect(dfg.activityCounts.get('A')).toBe(1)
  })

  it('two events: one directed edge', () => {
    const dfg = discoverDfg(events('A', 'B'))
    expect(dfg.edges.get('A')?.get('B')).toBe(1)
    expect(dfg.startActivities.get('A')).toBe(1)
    expect(dfg.endActivities.get('B')).toBe(1)
  })

  it('lawful 4-step lifecycle: 3 edges', () => {
    const dfg = discoverDfg(events(...LAWFUL))
    expect(dfg.edges.get('GameSessionStarted')?.get('InputAdmitted')).toBe(1)
    expect(dfg.edges.get('InputAdmitted')?.get('FrameRendered')).toBe(1)
    expect(dfg.edges.get('FrameRendered')?.get('GameSessionClosed')).toBe(1)
    expect(dfg.edges.size).toBe(3)
  })

  it('counts repeated activity', () => {
    const dfg = discoverDfg(events('A', 'B', 'A'))
    expect(dfg.activityCounts.get('A')).toBe(2)
    expect(dfg.activityCounts.get('B')).toBe(1)
  })
})

// ── computeFitness ───────────────────────────────────────────────────────────

describe('computeFitness', () => {
  it('perfect lawful trace → fitness=1.0', () => {
    const { fitness, deviations } = computeFitness(events(...LAWFUL), LAWFUL)
    expect(fitness).toBe(1.0)
    expect(deviations.filter(d => d.reason !== 'event after declared lifecycle exhausted')).toHaveLength(0)
  })

  it('empty events → fitness=0', () => {
    const { fitness } = computeFitness([], LAWFUL)
    expect(fitness).toBe(0)
  })

  it('all activities present but out of order → fitness < 1.0', () => {
    // Reverse order — every activity present but wrong sequence
    const reversed = [...LAWFUL].reverse()
    const { fitness, deviations } = computeFitness(events(...reversed), LAWFUL)
    expect(fitness).toBeLessThan(1.0)
    expect(deviations.length).toBeGreaterThan(0)
  })

  it('missing last activity → fitness < 1.0, deviation shows missing', () => {
    const { fitness, deviations } = computeFitness(events(...LAWFUL.slice(0, 3)), LAWFUL)
    expect(fitness).toBeLessThan(1.0)
    const missing = deviations.find(d => d.reason.includes('never observed'))
    expect(missing).toBeDefined()
  })

  it('extra activities after lifecycle → deviation recorded', () => {
    const extra = [...LAWFUL, 'ExtraActivity']
    const { deviations } = computeFitness(events(...extra), LAWFUL)
    const extraDeviation = deviations.find(d => d.actual_activity === 'ExtraActivity')
    expect(extraDeviation).toBeDefined()
  })

  it('single activity matching declared lifecycle of 1 → fitness=1.0', () => {
    const { fitness } = computeFitness(events('A'), ['A'])
    expect(fitness).toBe(1.0)
  })

  // Property: fitness ∈ [0,1] always
  it('∀ event sequences → fitness ∈ [0,1]', () => {
    fc.assert(
      fc.property(
        fc.array(fc.constantFrom('A', 'B', 'C', 'D'), { minLength: 0, maxLength: 10 }),
        (acts) => {
          const { fitness } = computeFitness(events(...acts), ['A', 'B', 'C'])
          expect(fitness).toBeGreaterThanOrEqual(0)
          expect(fitness).toBeLessThanOrEqual(1)
        },
      ),
    )
  })
})

// ── computePrecision ──────────────────────────────────────────────────────────

describe('computePrecision', () => {
  it('lawful trace → precision=1.0 (all DFG edges are allowed)', () => {
    const dfg = discoverDfg(events(...LAWFUL))
    const p = computePrecision(dfg, LAWFUL)
    expect(p).toBe(1.0)
  })

  it('rogue edge (A→C skipping B) → precision < 1.0', () => {
    const dfg = discoverDfg(events('GameSessionStarted', 'FrameRendered', 'GameSessionClosed'))
    const p = computePrecision(dfg, LAWFUL)
    expect(p).toBeLessThan(1.0)
  })

  it('empty DFG (single event) → precision=1.0', () => {
    const dfg = discoverDfg(events('GameSessionStarted'))
    expect(computePrecision(dfg, LAWFUL)).toBe(1.0)
  })

  // Property: precision ∈ [0,1] always
  it('∀ event sequences → precision ∈ [0,1]', () => {
    fc.assert(
      fc.property(
        fc.array(fc.constantFrom('A', 'B', 'C', 'D'), { minLength: 1, maxLength: 8 }),
        (acts) => {
          const dfg = discoverDfg(events(...acts))
          const p = computePrecision(dfg, ['A', 'B', 'C'])
          expect(p).toBeGreaterThanOrEqual(0)
          expect(p).toBeLessThanOrEqual(1)
        },
      ),
    )
  })
})

// ── computeSimplicity ─────────────────────────────────────────────────────────

describe('computeSimplicity', () => {
  it('lawful linear trace → simplicity=1.0 (no loops)', () => {
    const evts = events(...LAWFUL)
    const dfg = discoverDfg(evts)
    expect(computeSimplicity(evts, dfg)).toBe(1.0)
  })

  it('self-loop (A→A) → simplicity < 1.0', () => {
    const evts = events('A', 'A', 'B')
    const dfg = discoverDfg(evts)
    expect(computeSimplicity(evts, dfg)).toBeLessThan(1.0)
  })

  it('back-edge (B→A→B) → simplicity < 1.0', () => {
    const evts = events('A', 'B', 'A', 'C')
    const dfg = discoverDfg(evts)
    expect(computeSimplicity(evts, dfg)).toBeLessThan(1.0)
  })

  // Property: simplicity ∈ [0,1] always
  it('∀ event sequences → simplicity ∈ [0,1]', () => {
    fc.assert(
      fc.property(
        fc.array(fc.constantFrom('A', 'B', 'C'), { minLength: 1, maxLength: 10 }),
        (acts) => {
          const evts = events(...acts)
          const dfg = discoverDfg(evts)
          const s = computeSimplicity(evts, dfg)
          expect(s).toBeGreaterThanOrEqual(0)
          expect(s).toBeLessThanOrEqual(1)
        },
      ),
    )
  })
})

// ── checkConformance (integrated) ─────────────────────────────────────────────

describe('computeGeneralization', () => {
  it('each declared activity appears exactly once → 1.0', () => {
    expect(computeGeneralization(events(...LAWFUL), LAWFUL)).toBe(1.0)
  })

  it('empty events → 1.0 (cannot over-fit nothing)', () => {
    expect(computeGeneralization([], LAWFUL)).toBe(1.0)
  })

  it('empty declared lifecycle → 1.0', () => {
    expect(computeGeneralization(events(...LAWFUL), [])).toBe(1.0)
  })

  it('activity repeated 3× when declared once → generalization = 1/3', () => {
    // [A, A, A] against lifecycle [A] → generalization = min(1/3, 1) = 1/3
    const gen = computeGeneralization(events('A', 'A', 'A'), ['A'])
    expect(gen).toBeCloseTo(1 / 3, 5)
  })

  it('activity repeated 2× → generalization = 0.5', () => {
    const gen = computeGeneralization(events('A', 'B', 'A'), ['A', 'B'])
    // A: expected 1, actual 2 → 1/2; B: expected 1, actual 1 → 1; mean = 0.75
    expect(gen).toBeCloseTo(0.75, 5)
  })

  it('extra activities not in declared lifecycle are ignored', () => {
    // Extra 'X' should not penalize generalization (precision handles it)
    const gen = computeGeneralization(events('A', 'X', 'B'), ['A', 'B'])
    expect(gen).toBe(1.0)
  })

  it('missing declared activity → counts as 1.0 for that slot (fitness handles absence)', () => {
    // A appears 0 times — generalization doesn't penalize absences, fitness does
    const gen = computeGeneralization(events('B'), ['A', 'B'])
    // A: expected 1, actual 0 → 1.0; B: expected 1, actual 1 → 1.0; mean = 1.0
    expect(gen).toBe(1.0)
  })
})

describe('checkConformance', () => {
  it('perfect lawful trace → all scores 1.0, overall_score 1.0', () => {
    const result = checkConformance(events(...LAWFUL), LAWFUL)
    expect(result.fitness).toBe(1.0)
    expect(result.precision).toBe(1.0)
    expect(result.simplicity).toBe(1.0)
    expect(result.overall_score).toBeCloseTo(1.0, 5)
    expect(result.variants_discovered).toBe(1)
    expect(result.deviation_points.filter(d => !d.reason.includes('never observed'))).toHaveLength(0)
  })

  it('empty events → fitness=0, overall_score=0', () => {
    const result = checkConformance([], LAWFUL)
    expect(result.fitness).toBe(0)
    expect(result.overall_score).toBe(0)
  })

  it('rogue extra events → fitness=1.0 (all declared present), precision < 1.0', () => {
    const extra = [...LAWFUL, 'HackActivity', 'AnotherHack']
    const result = checkConformance(events(...extra), LAWFUL)
    expect(result.fitness).toBe(1.0)    // all 4 declared activities present
    expect(result.precision).toBeLessThan(1.0) // extra edges in DFG
  })

  it('reversed order → fitness < 1.0', () => {
    const result = checkConformance(events(...[...LAWFUL].reverse()), LAWFUL)
    expect(result.fitness).toBeLessThan(1.0)
    expect(result.deviation_points.length).toBeGreaterThan(0)
  })

  it('Van der Aalst: low fitness → overall_score near 0', () => {
    // Only 1 of 4 activities present, all wrong activities otherwise
    const result = checkConformance(events('X', 'Y', 'Z'), LAWFUL)
    expect(result.overall_score).toBeLessThan(0.5)
  })

  // Chatman Equation: correct inputs → score 1.0 (all 4 Van der Aalst dimensions)
  it('∀ lawful lifecycle traces → all 4 dimensions = 1.0, overall_score = 1.0 (property)', () => {
    fc.assert(
      fc.property(
        fc.array(fc.constantFrom('A', 'B', 'C', 'D'), { minLength: 2, maxLength: 6 }).map(
          acts => [...new Set(acts)] // deduplicate to avoid duplicate activities in lifecycle
        ).filter(acts => acts.length >= 2),
        (lifecycle) => {
          const result = checkConformance(events(...lifecycle), lifecycle)
          expect(result.fitness).toBe(1.0)
          expect(result.precision).toBe(1.0)
          expect(result.simplicity).toBe(1.0)
          expect(result.generalization).toBe(1.0)
          expect(result.overall_score).toBeCloseTo(1.0, 5)
        },
      ),
    )
  })
})
