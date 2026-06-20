/**
 * GET /api/game/receipt/[id]
 *
 * Returns a single receipt by ID with full manifest, chain verification status,
 * and OCEL event count. Pattern from dashboard.bak/server/api/receipts/[id].js.
 *
 * Used by ReceiptDrawer.vue and receipts.vue for per-receipt detail.
 *
 * Returns: {
 *   receipt: game_receipt row,
 *   chain_verified: boolean,
 *   ocel_event_count: number,
 *   first_event_at: string | null,
 *   last_event_at: string | null,
 * }
 */

import { createClient } from '@supabase/supabase-js'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id) throw createError({ statusCode: 400, message: 'Receipt ID required' })

  // Basic UUID format validation
  if (!/^[0-9a-f-]{36}$/.test(id)) {
    throw createError({ statusCode: 400, message: 'Invalid receipt ID format' })
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY
    ?? process.env.SUPABASE_ANON_KEY ?? ''
  const supabase = createClient(supabaseUrl, supabaseKey)

  const { data: receipt, error } = await supabase
    .from('game_receipts')
    .select('*')
    .eq('id', id)
    .single()

  if (error || !receipt) {
    throw createError({ statusCode: 404, message: `Receipt ${id} not found` })
  }

  // Chain verification (non-fatal)
  let chainVerified = false
  let ocelEventCount = 0
  let firstEventAt: string | null = null
  let lastEventAt: string | null = null

  if (receipt.session_id) {
    const [chainResult, eventsResult] = await Promise.all([
      supabase.rpc('verify_event_chain', { p_session_id: receipt.session_id }),
      supabase
        .from('ocel_events')
        .select('ts_ms, activity')
        .eq('session_id', receipt.session_id)
        .order('seq', { ascending: true }),
    ])
    chainVerified = chainResult.data === true
    const events = eventsResult.data ?? []
    ocelEventCount = events.length
    if (events.length) {
      const first = events[0] as { ts_ms: number }
      const last = events[events.length - 1] as { ts_ms: number }
      firstEventAt = new Date(first.ts_ms).toISOString()
      lastEventAt = new Date(last.ts_ms).toISOString()
    }
  }

  return {
    receipt,
    chain_verified: chainVerified,
    ocel_event_count: ocelEventCount,
    first_event_at: firstEventAt,
    last_event_at: lastEventAt,
  }
})
