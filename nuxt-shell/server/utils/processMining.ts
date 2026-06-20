/**
 * server/utils/processMining.ts
 *
 * Pure TypeScript inductive process mining for OCEL event logs.
 * Produces fitness, precision, and simplicity scores without pm4py.
 *
 * Implements the IEEE Process Mining spec subset needed for Van der Aalst doctrine:
 *   - Directly-Follows Graph (DFG) discovery
 *   - Token-replay fitness against a declared lifecycle
 *   - Precision: fraction of DFG edges that the model allows
 *   - Simplicity: penalizes loops and self-arcs
 *   - Variant analysis: unique traces (activity sequences)
 *
 * These metrics expose the gap between "shape is correct" and "process is lawful."
 */

export interface MiningEvent {
  activity: string
  timestamp_ms: number
  seq: number
}

export interface DirectlyFollowsGraph {
  /** edges[A][B] = count of times B directly follows A */
  edges: Map<string, Map<string, number>>
  startActivities: Map<string, number>
  endActivities: Map<string, number>
  activityCounts: Map<string, number>
}

export interface ConformanceResult {
  /** 0-1: fraction of declared lifecycle activities witnessed in lawful order */
  fitness: number
  /** 0-1: fraction of observed transitions allowed by the declared lifecycle */
  precision: number
  /** 0-1: absence of loops, self-arcs, and repeated activities */
  simplicity: number
  /**
   * 0-1: how well the model generalises — activities each appear only as many times as
   * the declared lifecycle expects them. Low generalization = overfitting to one trace.
   * Van der Aalst 4th quality dimension (completes the fitness/precision/simplicity trio).
   */
  generalization: number
  /** Combined score: 4th-root of fitness * precision * simplicity * generalization */
  overall_score: number
  /** Number of unique activity sequences observed */
  variants_discovered: number
  /** Events where the sequence deviated from the declared lifecycle */
  deviation_points: DeviationPoint[]
  /** All unique traces in this session */
  traces: string[][]
}

export interface DeviationPoint {
  seq: number
  expected_activity: string | null
  actual_activity: string
  reason: string
}

// ── DFG Discovery ─────────────────────────────────────────────────────────────

export function discoverDfg(events: MiningEvent[]): DirectlyFollowsGraph {
  const sorted = [...events].sort((a, b) => a.seq - b.seq)
  const edges = new Map<string, Map<string, number>>()
  const startActivities = new Map<string, number>()
  const endActivities = new Map<string, number>()
  const activityCounts = new Map<string, number>()

  for (let i = 0; i < sorted.length; i++) {
    const curr = sorted[i]!
    activityCounts.set(curr.activity, (activityCounts.get(curr.activity) ?? 0) + 1)

    if (i === 0) {
      startActivities.set(curr.activity, (startActivities.get(curr.activity) ?? 0) + 1)
    }

    if (i === sorted.length - 1) {
      endActivities.set(curr.activity, (endActivities.get(curr.activity) ?? 0) + 1)
    }

    if (i < sorted.length - 1) {
      const next = sorted[i + 1]!
      if (!edges.has(curr.activity)) edges.set(curr.activity, new Map())
      const targets = edges.get(curr.activity)!
      targets.set(next.activity, (targets.get(next.activity) ?? 0) + 1)
    }
  }

  return { edges, startActivities, endActivities, activityCounts }
}

// ── Token Replay Fitness ───────────────────────────────────────────────────────

/**
 * Replay events against a declared lifecycle (ordered list of expected activities).
 * Fitness = (matched activities) / (declared activities).
 * An activity "matches" if it appears in the correct position in the trace.
 */
export function computeFitness(
  events: MiningEvent[],
  declaredLifecycle: string[],
): { fitness: number; matched: number; deviations: DeviationPoint[] } {
  if (declaredLifecycle.length === 0 || events.length === 0) {
    return { fitness: 0, matched: 0, deviations: [] }
  }

  const sorted = [...events].sort((a, b) => a.seq - b.seq)
  const deviations: DeviationPoint[] = []
  let declaredIdx = 0
  let matched = 0

  for (const evt of sorted) {
    if (declaredIdx < declaredLifecycle.length) {
      if (evt.activity === declaredLifecycle[declaredIdx]) {
        // Perfect match — advance declared pointer
        matched++
        declaredIdx++
      } else if (declaredLifecycle.includes(evt.activity)) {
        // Activity appears in lifecycle but out of order — skip forward in declared
        const ahead = declaredLifecycle.indexOf(evt.activity, declaredIdx)
        if (ahead > declaredIdx) {
          // Skipped activities in declared lifecycle
          deviations.push({
            seq: evt.seq,
            expected_activity: declaredLifecycle[declaredIdx]!,
            actual_activity: evt.activity,
            reason: `out-of-order: ${declaredLifecycle.slice(declaredIdx, ahead).join('→')} were skipped`,
          })
          declaredIdx = ahead + 1
          matched++ // credit for eventually appearing
        }
      } else {
        // Extra event not in declared lifecycle
        deviations.push({
          seq: evt.seq,
          expected_activity: declaredLifecycle[declaredIdx] ?? null,
          actual_activity: evt.activity,
          reason: `extra activity not in declared lifecycle`,
        })
      }
    } else {
      // More events than declared — extra events after lifecycle complete
      deviations.push({
        seq: evt.seq,
        expected_activity: null,
        actual_activity: evt.activity,
        reason: 'event after declared lifecycle exhausted',
      })
    }
  }

  // Penalize for missing declared activities (declaredIdx didn't reach end)
  const missing = declaredLifecycle.length - declaredIdx
  if (missing > 0) {
    deviations.push({
      seq: sorted[sorted.length - 1]?.seq ?? -1,
      expected_activity: declaredLifecycle[declaredIdx] ?? null,
      actual_activity: '(missing)',
      reason: `${missing} declared activit${missing === 1 ? 'y' : 'ies'} never observed`,
    })
  }

  return {
    fitness: matched / declaredLifecycle.length,
    matched,
    deviations,
  }
}

// ── Precision ─────────────────────────────────────────────────────────────────

/**
 * Precision = fraction of observed DFG edges that the declared lifecycle allows.
 * A lifecycle [A→B→C] allows edges {A→B, B→C}. Any other edge reduces precision.
 */
export function computePrecision(dfg: DirectlyFollowsGraph, declaredLifecycle: string[]): number {
  // Build allowed edge set from declared lifecycle
  const allowed = new Set<string>()
  for (let i = 0; i < declaredLifecycle.length - 1; i++) {
    allowed.add(`${declaredLifecycle[i]}→${declaredLifecycle[i + 1]}`)
  }

  let totalEdges = 0
  let allowedEdges = 0

  for (const [from, targets] of dfg.edges) {
    for (const to of targets.keys()) {
      totalEdges++
      if (allowed.has(`${from}→${to}`)) allowedEdges++
    }
  }

  return totalEdges === 0 ? 1.0 : allowedEdges / totalEdges
}

// ── Simplicity ────────────────────────────────────────────────────────────────

/**
 * Simplicity = 1 - loop_fraction.
 * Loops are self-arcs (A→A) or back-edges (B→A where A appeared before B).
 */
export function computeSimplicity(
  events: MiningEvent[],
  dfg: DirectlyFollowsGraph,
): number {
  const sorted = [...events].sort((a, b) => a.seq - b.seq)

  // Activity first occurrence index
  const firstSeen = new Map<string, number>()
  sorted.forEach((e, i) => {
    if (!firstSeen.has(e.activity)) firstSeen.set(e.activity, i)
  })

  let totalEdgeWeight = 0
  let loopWeight = 0

  for (const [from, targets] of dfg.edges) {
    for (const [to, count] of targets) {
      totalEdgeWeight += count
      // Self-arc or back-edge = loop
      if (from === to || (firstSeen.get(to) ?? Infinity) < (firstSeen.get(from) ?? Infinity)) {
        loopWeight += count
      }
    }
  }

  return totalEdgeWeight === 0 ? 1.0 : 1.0 - loopWeight / totalEdgeWeight
}

// ── Generalization ────────────────────────────────────────────────────────────

/**
 * Generalization measures whether the model is over-fitted to the observed log.
 * For a simple lifecycle [A, B, C], perfect generalization means each activity
 * appears exactly once (the model could replay new traces without being too specific).
 *
 * Implementation: for each activity in the declared lifecycle, count how many times
 * it actually appears vs the expected count (1 per lifecycle step). Activities that
 * appear far more than expected reduce generalization.
 *
 *   generalization = mean( min(expected/actual, 1) ) over all declared activities
 *
 * Activities not in the declared lifecycle are ignored (they penalize precision, not
 * generalization). An empty log or empty lifecycle returns 1.0 (cannot over-fit nothing).
 */
export function computeGeneralization(
  events: MiningEvent[],
  declaredLifecycle: string[],
): number {
  if (declaredLifecycle.length === 0 || events.length === 0) return 1.0

  // Count how many times each declared activity appears in the log
  const observed = new Map<string, number>()
  for (const e of events) {
    if (declaredLifecycle.includes(e.activity)) {
      observed.set(e.activity, (observed.get(e.activity) ?? 0) + 1)
    }
  }

  // Expected count per activity = number of times it appears in declared lifecycle
  const expectedCounts = new Map<string, number>()
  for (const act of declaredLifecycle) {
    expectedCounts.set(act, (expectedCounts.get(act) ?? 0) + 1)
  }

  let sum = 0
  for (const [act, expected] of expectedCounts) {
    const actual = observed.get(act) ?? 0
    if (actual === 0) {
      // Activity never appeared → no over-fitting concern; fitness handles the absence
      sum += 1.0
    } else {
      // Penalize if actual >> expected (activity repeated more than lifecycle expects)
      sum += Math.min(expected / actual, 1.0)
    }
  }
  return sum / expectedCounts.size
}

// ── Variant Discovery ─────────────────────────────────────────────────────────

/**
 * A variant is a unique sequence of activities (the "trace").
 * For a single session, there is one trace. Returns it as a string array.
 */
export function discoverVariants(events: MiningEvent[]): string[][] {
  const sorted = [...events].sort((a, b) => a.seq - b.seq)
  // One session = one trace. But handle repeated activities by listing all.
  return [sorted.map(e => e.activity)]
}

// ── Full Conformance Check ────────────────────────────────────────────────────

/**
 * Run full conformance analysis: DFG + fitness + precision + simplicity.
 * Returns a ConformanceResult that can be attached to any chain-verify response.
 */
export function checkConformance(
  events: MiningEvent[],
  declaredLifecycle: string[],
): ConformanceResult {
  const dfg = discoverDfg(events)
  const { fitness, deviations } = computeFitness(events, declaredLifecycle)
  const precision = computePrecision(dfg, declaredLifecycle)
  const simplicity = computeSimplicity(events, dfg)
  const generalization = computeGeneralization(events, declaredLifecycle)

  // 4th-root geometric mean — all four Van der Aalst dimensions must be high
  const overall_score = fitness > 0 && precision > 0 && simplicity > 0 && generalization > 0
    ? Math.pow(fitness * precision * simplicity * generalization, 0.25)
    : 0

  const traces = discoverVariants(events)

  return {
    fitness,
    precision,
    simplicity,
    generalization,
    overall_score,
    variants_discovered: traces.length,
    deviation_points: deviations,
    traces,
  }
}
