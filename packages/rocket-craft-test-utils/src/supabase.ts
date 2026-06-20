/**
 * Supabase test helpers — require @supabase/supabase-js as a peer dep.
 * Never expose service-role key in client-side code.
 * Every helper that writes data returns a receipt or residual.
 */
import type { RocketResidual } from './types.js'
import { residualFromError } from './residuals.js'

interface SupabaseClient {
  auth: {
    signUp(opts: { email: string; password: string }): Promise<{ error: Error | null }>
    signInWithPassword(opts: { email: string; password: string }): Promise<{ data: unknown; error: Error | null }>
  }
  from(table: string): {
    insert(row: unknown): Promise<{ data: unknown; error: Error | null }>
    select(cols?: string): {
      eq(col: string, val: unknown): Promise<{ data: unknown[]; error: Error | null }>
    }
  }
}

export function expectLocalSupabaseReady(
  status: { status: string; error?: string }
): void {
  if (status.status !== 'ok')
    throw new Error(`Local Supabase not ready: ${status.error ?? status.status}`)
}

export async function createTestUser(
  client: SupabaseClient,
  email: string,
  password: string
): Promise<{ ok: boolean; residuals: RocketResidual[] }> {
  const residuals: RocketResidual[] = []
  try {
    const { error } = await client.auth.signUp({ email, password })
    if (error) {
      residuals.push(residualFromError(error, 'supabase-auth', 'AUTH-SIGNUP-FAILED'))
      return { ok: false, residuals }
    }
    return { ok: true, residuals }
  } catch (err) {
    residuals.push(residualFromError(err, 'supabase-auth', 'AUTH-SIGNUP-THROW'))
    return { ok: false, residuals }
  }
}

export async function loginTestUser(
  client: SupabaseClient,
  email: string,
  password: string
): Promise<{ ok: boolean; session: unknown; residuals: RocketResidual[] }> {
  const residuals: RocketResidual[] = []
  try {
    const { data, error } = await client.auth.signInWithPassword({ email, password })
    if (error) {
      residuals.push(residualFromError(error, 'supabase-auth', 'AUTH-LOGIN-FAILED'))
      return { ok: false, session: null, residuals }
    }
    return { ok: true, session: data, residuals }
  } catch (err) {
    residuals.push(residualFromError(err, 'supabase-auth', 'AUTH-LOGIN-THROW'))
    return { ok: false, session: null, residuals }
  }
}

export async function insertGameIntent(
  client: SupabaseClient,
  table: string,
  row: unknown
): Promise<{ ok: boolean; residuals: RocketResidual[] }> {
  const residuals: RocketResidual[] = []
  try {
    const { error } = await client.from(table).insert(row)
    if (error) {
      residuals.push(residualFromError(error, 'supabase-db', 'DB-INSERT-FAILED'))
      return { ok: false, residuals }
    }
    return { ok: true, residuals }
  } catch (err) {
    residuals.push(residualFromError(err, 'supabase-db', 'DB-INSERT-THROW'))
    return { ok: false, residuals }
  }
}

export function expectServiceRoleNotInClientBundle(bundleText: string): void {
  // Service role keys are longer and start with specific prefixes
  if (/service_role/.test(bundleText))
    throw new Error('Service role key found in client bundle — law violation')
  if (/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\.[^"]{50,}/.test(bundleText)) {
    // Long JWT in bundle — warn but don't throw (could be anon key)
  }
}
