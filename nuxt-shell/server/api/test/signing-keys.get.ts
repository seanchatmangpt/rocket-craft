/**
 * GET /api/test/signing-keys
 *
 * Observability + test hook for the Ed25519 key-rotation invariant: there must be
 * AT MOST ONE active signing key at any time (enforced by the partial unique index
 * on status='active'), and rotation must never leave ZERO active keys (the
 * recovery path in rotate-key restores the prior key if the new insert fails).
 *
 * Returns: { active_count, rotating_count, revoked_count, total, active_key_id }
 *
 * Security: test/dev only (same gate as the other test endpoints). Returns only
 * key ids + status (never private material — there is none here; public keys only).
 */

import { createClient } from '@supabase/supabase-js';

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);

  const nodeEnv = process.env.NODE_ENV ?? 'production';
  const allow = process.env.ALLOW_SESSION_SEED === '1' || nodeEnv === 'test' || nodeEnv === 'development';
  if (!allow) {
    throw createError({ statusCode: 403, statusMessage: 'signing-keys is test/dev only (set ALLOW_SESSION_SEED=1)' });
  }

  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = config.supabaseServiceRoleKey as string;
  if (!serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'SUPABASE_SERVICE_ROLE_KEY not set' });
  }

  const sb = createClient<any>(supabaseUrl, serviceKey);
  const { data, error } = await sb
    .from('signing_keys')
    .select('id, status, created_at')
    .order('created_at', { ascending: false });
  if (error) {
    throw createError({ statusCode: 500, statusMessage: `signing_keys query failed: ${error.message}` });
  }

  const keys = (data ?? []) as Array<{ id: string; status: string; created_at: string }>;
  const byStatus = (s: string) => keys.filter((k) => k.status === s);
  const active = byStatus('active');

  return {
    active_count: active.length,
    rotating_count: byStatus('rotating').length,
    revoked_count: byStatus('revoked').length,
    total: keys.length,
    active_key_id: active[0]?.id ?? null,
  };
});
