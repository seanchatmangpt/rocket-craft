/**
 * GET /api/game/process-map
 *
 * Returns a discovered process map for the game session pipeline — derived
 * from the ocel_events table. Computes:
 *   - DFG (Directly-Follows Graph): which activities follow which
 *   - Activity frequency counts
 *   - Fitness verdict (does the declared lifecycle appear in the graph?)
 *
 * This is a server-side approximation of pm4py's inductive miner output,
 * without requiring Python in the server process. For full pm4py analysis,
 * run scripts/pm4py_conformance.py.
 *
 * Query params:
 *   session_id — restrict to a specific session (optional)
 *   limit      — max events to scan (default 1000)
 *
 * Returns: {
 *   nodes: Array<{ id: string; label: string; count: number }>,
 *   edges: Array<{ from: string; to: string; count: number }>,
 *   lifecycle_ok: boolean,
 *   total_events: number,
 * }
 */

import { createClient } from '@supabase/supabase-js'

const DECLARED_LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted']

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const sessionId = query.session_id as string | undefined
  const limit = Math.min(Number(query.limit ?? 1000), 5000)

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY
    ?? process.env.SUPABASE_ANON_KEY
    ?? ''

  const supabase = createClient(supabaseUrl, supabaseKey)

  let q = supabase
    .from('ocel_events')
    .select('session_id, activity, seq, ts_ms')
    .order('session_id', { ascending: true })
    .order('seq', { ascending: true })
    .limit(limit)

  if (sessionId) q = q.eq('session_id', sessionId)

  const { data: events, error } = await q
  if (error) {
    throw createError({ statusCode: 500, message: `ocel_events query failed: ${error.message}` })
  }

  if (!events?.length) {
    return {
      nodes: [],
      edges: [],
      lifecycle_ok: false,
      total_events: 0,
    }
  }

  // Build DFG per session, then aggregate
  const activityCounts = new Map<string, number>()
  const edgeCounts = new Map<string, number>()

  // Group events by session
  const sessions = new Map<string, typeof events>()
  for (const evt of events) {
    const sid = (evt.session_id ?? 'unknown') as string
    if (!sessions.has(sid)) sessions.set(sid, [])
    const bucket = sessions.get(sid)
    if (bucket) bucket.push(evt)
  }

  for (const sessionEvents of sessions.values()) {
    for (let i = 0; i < sessionEvents.length; i++) {
      const cur = sessionEvents[i]
      if (!cur) continue
      const act = cur.activity as string
      activityCounts.set(act, (activityCounts.get(act) ?? 0) + 1)
      if (i + 1 < sessionEvents.length) {
        const nxt = sessionEvents[i + 1]
        if (!nxt) continue
        const next = nxt.activity as string
        const key = `${act}→${next}`
        edgeCounts.set(key, (edgeCounts.get(key) ?? 0) + 1)
      }
    }
  }

  const nodes = Array.from(activityCounts.entries())
    .sort(([, a], [, b]) => b - a)
    .map(([id, count]) => ({ id, label: id, count }))

  const edges = Array.from(edgeCounts.entries())
    .sort(([, a], [, b]) => b - a)
    .map(([key, count]) => {
      const [from, to] = key.split('→')
      return { from, to, count }
    })

  // Check if declared lifecycle appears as a sub-sequence in at least one session
  let lifecycleOk = false
  for (const sessionEvents of sessions.values()) {
    const activities = sessionEvents.map((e) => e.activity as string)
    let matched = 0
    for (const act of activities) {
      if (matched < DECLARED_LIFECYCLE.length && act === DECLARED_LIFECYCLE[matched]) {
        matched++
      }
    }
    if (matched === DECLARED_LIFECYCLE.length) {
      lifecycleOk = true
      break
    }
  }

  return {
    nodes,
    edges,
    lifecycle_ok: lifecycleOk,
    declared_lifecycle: DECLARED_LIFECYCLE,
    total_events: events.length,
    sessions_scanned: sessions.size,
  }
})
