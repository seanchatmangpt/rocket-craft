/**
 * PATCH /api/game/session/[id]
 *
 * Updates a game_sessions row (is_alive, ocel_event_count, session_ended_at,
 * receipt_hash). Used by useGameSessionPersistence to:
 *   - sync event count during gameplay
 *   - close the session (session_ended_at + receipt_hash)
 *
 * Body: Partial<{ is_alive, ocel_event_count, session_ended_at, receipt_hash }>
 * Returns: { updated: true, session_id }
 */

import { createClient } from '@supabase/supabase-js'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id || !/^[0-9a-f-]{36}$/.test(id)) {
    throw createError({ statusCode: 400, message: 'Valid session UUID required' })
  }

  const body = await readBody(event).catch(() => ({}))
  const allowed = ['is_alive', 'ocel_event_count', 'session_ended_at', 'receipt_hash']
  const patch: Record<string, unknown> = {}
  for (const key of allowed) {
    if (key in body) patch[key] = body[key]
  }

  if (Object.keys(patch).length === 0) {
    throw createError({ statusCode: 400, message: 'No updatable fields provided' })
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY
    ?? process.env.SUPABASE_ANON_KEY ?? ''
  const supabase = createClient(supabaseUrl, supabaseKey)

  const { error } = await supabase
    .from('game_sessions')
    .update(patch)
    .eq('id', id)

  if (error) {
    throw createError({ statusCode: 500, message: error.message })
  }

  return { updated: true, session_id: id }
})
