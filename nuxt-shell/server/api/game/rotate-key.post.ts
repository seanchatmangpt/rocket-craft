/**
 * POST /api/game/rotate-key
 *
 * Rotates the active Ed25519 signing key:
 *   1. Inserts the new public key as status='active' (with replaces_key_id pointing to prior)
 *   2. Updates the prior active key to status='rotating' (grace period)
 *
 * Pattern from ~/dashboard.bak/app/composables/useKeyManagement.js:
 *   rotationSchedule → keygen → rotate → revoke after gracePeriodDays
 *
 * Body: { new_public_key_b64: string, notes?: string }
 * Requires: SUPABASE_SERVICE_ROLE_KEY (write access to signing_keys)
 */

import { createClient } from '@supabase/supabase-js'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const { new_public_key_b64, notes } = body ?? {}

  if (!new_public_key_b64 || typeof new_public_key_b64 !== 'string') {
    throw createError({ statusCode: 400, message: 'new_public_key_b64 is required' })
  }
  if (new_public_key_b64.length < 40) {
    throw createError({ statusCode: 400, message: 'new_public_key_b64 appears too short for a base64 Ed25519 public key' })
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const serviceKey = process.env.SUPABASE_SERVICE_ROLE_KEY
  if (!serviceKey) {
    throw createError({ statusCode: 503, message: 'SUPABASE_SERVICE_ROLE_KEY not configured' })
  }

  const supabase = createClient(supabaseUrl, serviceKey)

  // Find current active key
  const { data: activeKey, error: fetchErr } = await supabase
    .from('signing_keys')
    .select('id, public_key_b64, created_at')
    .eq('status', 'active')
    .maybeSingle()

  if (fetchErr) {
    throw createError({ statusCode: 500, message: `Failed to fetch active key: ${fetchErr.message}` })
  }

  // Demote prior active key FIRST (must happen before inserting new active row
  // because the unique partial index on status='active' allows only one at a time)
  if (activeKey) {
    const { error: demoteErr } = await supabase
      .from('signing_keys')
      .update({ status: 'rotating', rotated_at: new Date().toISOString() })
      .eq('id', activeKey.id)

    if (demoteErr) {
      throw createError({ statusCode: 500, message: `Failed to demote prior key: ${demoteErr.message}` })
    }
  }

  // Insert new key as active (slot is now free)
  const { data: newKey, error: insertErr } = await supabase
    .from('signing_keys')
    .insert({
      public_key_b64: new_public_key_b64,
      status: 'active',
      replaces_key_id: activeKey?.id ?? null,
      notes: notes ?? null,
    })
    .select('id, created_at')
    .single()

  if (insertErr) {
    // If insert fails after demotion, attempt to restore prior key as active
    if (activeKey) {
      await supabase
        .from('signing_keys')
        .update({ status: 'active', rotated_at: null })
        .eq('id', activeKey.id)
    }
    throw createError({ statusCode: 500, message: `Failed to insert new signing key: ${insertErr.message}` })
  }

  return {
    rotated: true,
    new_key_id: newKey.id,
    new_key_created_at: newKey.created_at,
    prior_key_id: activeKey?.id ?? null,
    prior_key_demoted: !!activeKey,
  }
})
