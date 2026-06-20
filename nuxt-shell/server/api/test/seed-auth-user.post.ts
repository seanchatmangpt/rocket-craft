/**
 * POST /api/test/seed-auth-user
 *
 * Creates (or reuses) a real Supabase auth.users row via the admin API and
 * returns an access token, so E2E/RLS tests can log in as a genuine owner
 * instead of bypassing auth. Closes the gap where ALLOW_ANON_GAME had to skip
 * the auth guard entirely — with a real user, owner-scoped RLS policies
 * (migration 0016) are actually exercised.
 *
 * Body: { email?: string, password?: string, username?: string }
 *   Defaults to a deterministic dev pilot so tests are repeatable.
 * Returns: { user_id, email, access_token, refresh_token }
 *
 * Security: test/dev only (same gate as session-seed). Uses the service-role
 * key, which never leaves the server. NEVER enable in production.
 *
 * Pattern: dashboard.bak/scripts/init-supabase.js (auth.admin.createUser) +
 * expo-supabase-ai-template/supabase/seed.sql (deterministic fixture user).
 */

import { createClient } from '@supabase/supabase-js';

const DEFAULT_EMAIL = 'dev-pilot@rocket-craft.local';
const DEFAULT_PASSWORD = 'rocket-craft-dev-pilot';
const DEFAULT_USERNAME = 'dev-pilot';

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event);

  // Same gate as session-seed: test/dev only.
  const nodeEnv = process.env.NODE_ENV ?? 'production';
  const allow = process.env.ALLOW_SESSION_SEED === '1' || nodeEnv === 'test' || nodeEnv === 'development';
  if (!allow) {
    throw createError({
      statusCode: 403,
      statusMessage: 'seed-auth-user is only available in test/dev (set ALLOW_SESSION_SEED=1)',
    });
  }

  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = config.supabaseServiceRoleKey as string;
  if (!serviceKey) {
    throw createError({ statusCode: 503, statusMessage: 'SUPABASE_SERVICE_ROLE_KEY not set — admin user creation requires service role' });
  }

  const body = await readBody(event).catch(() => ({}));
  const email: string = body?.email ?? DEFAULT_EMAIL;
  const password: string = body?.password ?? DEFAULT_PASSWORD;
  const username: string = body?.username ?? DEFAULT_USERNAME;

  // Admin client (service role) — required for auth.admin.* operations.
  const admin = createClient(supabaseUrl, serviceKey, {
    auth: { autoRefreshToken: false, persistSession: false },
  });

  // Create the auth user. If it already exists, fall through to sign-in.
  let userId: string | null = null;
  const { data: created, error: createErr } = await admin.auth.admin.createUser({
    email,
    password,
    email_confirm: true, // skip the email verification round-trip
    user_metadata: { username, seeded: true },
  });

  if (created?.user) {
    userId = created.user.id;
  } else if (createErr && !/already.*regist|already.*exist|email.*exist/i.test(createErr.message)) {
    // A real failure (not "already exists") — surface it.
    throw createError({ statusCode: 500, statusMessage: `createUser failed: ${createErr.message}` });
  }

  // Ensure a linked players row exists (owner_id = auth user id) so owner-scoped
  // RLS and the leaderboard trigger have a player to attribute receipts to.
  if (userId) {
    await admin.from('players')
      .upsert({ id: userId, username, high_score: 0 }, { onConflict: 'id' });
  }

  // Sign in with a normal anon client to obtain a usable access token for the browser.
  const anonKey = (config.public.supabaseAnonKey as string) || '';
  if (!anonKey) {
    throw createError({ statusCode: 503, statusMessage: 'SUPABASE_ANON_KEY not set — cannot mint access token' });
  }
  const authClient = createClient(supabaseUrl, anonKey, {
    auth: { autoRefreshToken: false, persistSession: false },
  });
  const { data: session, error: signInErr } = await authClient.auth.signInWithPassword({ email, password });
  if (signInErr || !session?.session) {
    throw createError({ statusCode: 500, statusMessage: `sign-in failed: ${signInErr?.message ?? 'no session'}` });
  }

  return {
    user_id: session.session.user.id,
    email,
    username,
    access_token: session.session.access_token,
    refresh_token: session.session.refresh_token,
  };
});
