/**
 * GET /api/game/pipeline-health
 *
 * Cached pipeline health snapshot. Pattern: ~/nuxt-layer/server/utils/setDb.ts
 * (Nitro useStorage KV for in-process caching without Redis).
 *
 * Strategy:
 *   - First request hits Supabase pipeline_health view (materialized during migration)
 *   - Result cached in Nitro KV with 20s TTL — fast for the 30s dashboard refresh cycle
 *   - Cache key: 'pipeline:health:v1'
 *   - ?bust=1 query param forces a fresh fetch and updates the cache
 *
 * Returns the same shape as the pipeline_health Supabase view plus:
 *   cached_at     — ISO timestamp when the cache was written
 *   cache_hit     — true if served from KV, false if fresh from Supabase
 */
import { createClient } from '@supabase/supabase-js';

const CACHE_KEY = 'pipeline:health:v1';
const TTL_MS = 20_000;

interface CachedHealth {
  data: Record<string, unknown>;
  cached_at: string;
  expires_at: number;
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const bust = query.bust === '1';

  const storage = useStorage('data');

  // Serve from KV cache if fresh and not busted
  if (!bust) {
    const cached = await storage.getItem<CachedHealth>(CACHE_KEY);
    if (cached && Date.now() < cached.expires_at) {
      return { ...cached.data, cached_at: cached.cached_at, cache_hit: true };
    }
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = (config.public.supabaseUrl as string) || 'http://localhost:54321';
  const serviceKey = (config.supabaseServiceRoleKey as string) || (config.public.supabaseAnonKey as string) || '';

  if (!serviceKey) {
    throw createError({ statusCode: 503, message: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const supabase = createClient<any>(supabaseUrl, serviceKey);
  const { data, error } = await supabase.from('pipeline_health').select('*').single();

  if (error) throw createError({ statusCode: 500, message: error.message });

  const now = new Date().toISOString();
  const entry: CachedHealth = {
    data: data as Record<string, unknown>,
    cached_at: now,
    expires_at: Date.now() + TTL_MS,
  };

  await storage.setItem(CACHE_KEY, entry);

  return { ...data, cached_at: now, cache_hit: false };
});
