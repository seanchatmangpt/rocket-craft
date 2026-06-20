/**
 * POST /api/game/session
 *
 * Creates a new game_sessions row and returns the DB-assigned UUID.
 * Moves session creation server-side so the service role key is used
 * instead of the anon client — no RLS bypass required from the browser.
 *
 * Body: { browser_session_id: string, engine_source?: string }
 * Returns: { session_id: string, started_at: string }
 */

import { createClient } from '@supabase/supabase-js'

export default defineEventHandler(async (event) => {
  const body = await readBody(event).catch(() => ({}))
  const browserSessionId: string | undefined = body?.browser_session_id
  const engineSource: string = body?.engine_source ?? 'browser'

  if (!browserSessionId) {
    throw createError({ statusCode: 400, message: 'browser_session_id is required' })
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY
    ?? process.env.SUPABASE_ANON_KEY ?? ''
  const supabase = createClient(supabaseUrl, supabaseKey)

  const startedAt = new Date().toISOString()

  const { data, error } = await supabase
    .from('game_sessions')
    .insert({
      player_id: null,
      session_started_at: startedAt,
      session_ended_at: null,
      engine_source: engineSource,
      is_alive: true,
      ocel_event_count: 0,
      receipt_hash: null,
      metadata: { browser_session_id: browserSessionId },
    })
    .select('id')
    .single()

  if (error || !data) {
    throw createError({ statusCode: 500, message: error?.message ?? 'session insert failed' })
  }

  return { session_id: data.id as string, started_at: startedAt }
})
