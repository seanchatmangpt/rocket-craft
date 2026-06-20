/**
 * E2E test hooks for game session lifecycle.
 *
 * Gap 1 fix: automatically close any alive game_sessions left by previous
 * Playwright runs. Without this, stale sessions accumulate in Supabase and
 * corrupt `verify_event_chain` + `pipeline_health` results.
 *
 * Usage in spec:
 *   import { setupSessionCleanup } from './hooks/session-cleanup';
 *   test.beforeAll(setupSessionCleanup.beforeAll);
 *   test.afterAll(setupSessionCleanup.afterAll);
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';

const SUPABASE_URL = process.env.SUPABASE_URL || 'http://localhost:54321';
const SUPABASE_ANON_KEY = process.env.SUPABASE_ANON_KEY ||
  process.env.SUPABASE_SERVICE_ROLE_KEY ||
  'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let _client: SupabaseClient<any> | null = null;

function getClient() {
  if (!_client) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    _client = createClient<any>(SUPABASE_URL, SUPABASE_ANON_KEY);
  }
  return _client;
}

export let testSessionId: string | null = null;

export const setupSessionCleanup = {
  /** Before the test suite: close any stale sessions from prior runs */
  async beforeAll() {
    const sb = getClient();
    const tenMinAgo = new Date(Date.now() - 10 * 60 * 1000).toISOString();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data } = await (sb as any)
      .from('game_sessions')
      .select('id')
      .eq('is_alive', true)
      .lt('session_started_at', tenMinAgo);

    if (data?.length) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await (sb as any)
        .from('game_sessions')
        .update({ is_alive: false, session_ended_at: new Date().toISOString() })
        .in('id', data.map((r: { id: string }) => r.id));
      console.log(`[session-cleanup] Closed ${data.length} stale session(s) before test suite`);
    }
  },

  /** After the test suite: close the session opened by this run */
  async afterAll() {
    if (!testSessionId) return;
    const sb = getClient();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    await (sb as any)
      .from('game_sessions')
      .update({ is_alive: false, session_ended_at: new Date().toISOString() })
      .eq('id', testSessionId);
    console.log(`[session-cleanup] Closed test session ${testSessionId}`);
    testSessionId = null;
  },

  /** Register the session ID created during the test so afterAll can close it */
  setSessionId(id: string) {
    testSessionId = id;
  },
};
