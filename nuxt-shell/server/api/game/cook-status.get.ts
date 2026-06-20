/**
 * GET /api/game/cook-status
 *
 * Returns the current HTML5 cook pipeline status by reading:
 *   1. The latest ue4-cook*.log tail (if COOK_LOG_DIR is set)
 *   2. The latest game_receipts row with milestone='HTML5CookVerify'
 *   3. Active sessions with engine_source matching real cook pipeline
 *
 * Used by CI to poll cook progress without SSHing into the build machine.
 * Also used by pipeline.vue to show a live cook status badge.
 *
 * Query params:
 *   lines  — log tail lines (default 50, max 200)
 *   project — filter by project name (default: Brm)
 *
 * Returns: {
 *   status: 'idle' | 'cooking' | 'done' | 'failed'
 *   project: string
 *   last_receipt: { verdict, proven_at, output_hash } | null
 *   log_tail: string[]    — last N log lines (empty if COOK_LOG_DIR not set)
 *   log_file: string | null
 *   cook_events: string[] — activities found in latest cook session
 * }
 */

import { createClient } from '@supabase/supabase-js'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { buildCookSummary } from '../../utils/cookStatus'

function latestCookLog(logDir: string, _project: string): { path: string; lines: string[] } | null {
  try {
    const files = readdirSync(logDir)
      .filter(f => f.startsWith('ue4-cook') && f.endsWith('.log'))
      .map(f => ({ f, mtime: statSync(join(logDir, f)).mtimeMs }))
      .sort((a, b) => b.mtime - a.mtime)

    if (!files.length) return null
    const latest = files[0]!
    const content = readFileSync(join(logDir, latest.f), 'utf-8')
    const lines = content.split('\n').filter(Boolean)
    return { path: join(logDir, latest.f), lines }
  } catch {
    return null
  }
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const lines = Math.min(Number(query.lines ?? 50), 200)
  const project = (query.project as string) ?? 'Brm'

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY
    ?? process.env.SUPABASE_ANON_KEY
    ?? ''

  const supabase = createClient(supabaseUrl, supabaseKey)

  // Fetch latest cook receipt
  const { data: lastReceipt } = await supabase
    .from('game_receipts')
    .select('verdict, proven_at, output_hash, engine_source, ocel_lifecycle, session_id')
    .eq('milestone', 'HTML5CookVerify')
    .order('proven_at', { ascending: false })
    .limit(1)
    .maybeSingle()

  // Fetch cook events from the latest cook session
  let cookEvents: string[] = []
  if (lastReceipt?.session_id) {
    const { data: events } = await supabase
      .from('ocel_events')
      .select('activity')
      .eq('session_id', lastReceipt.session_id)
      .order('seq', { ascending: true })
    cookEvents = (events ?? []).map(e => e.activity as string)
  }
  if (!cookEvents.length && lastReceipt?.ocel_lifecycle) {
    cookEvents = lastReceipt.ocel_lifecycle as string[]
  }

  // Read cook log if available
  const logDir = process.env.COOK_LOG_DIR ?? '/tmp'
  const logResult = latestCookLog(logDir, project)

  const summary = buildCookSummary({
    logLines: logResult?.lines ?? [],
    logFile: logResult?.path ?? null,
    project,
    lastReceipt: lastReceipt ?? null,
    cookEvents,
    tailLines: lines,
  })

  return {
    status: summary.status,
    project: summary.project,
    last_receipt: lastReceipt
      ? {
          verdict: lastReceipt.verdict,
          proven_at: lastReceipt.proven_at,
          output_hash: lastReceipt.output_hash,
          engine_source: lastReceipt.engine_source,
        }
      : null,
    log_tail: summary.log_tail,
    log_file: summary.log_file,
    cook_events: summary.cook_events,
  }
})
