/**
 * server/utils/conformanceCache.ts
 *
 * LRU cache for conformance checking results, indexed by session_id + model hash.
 * Avoids re-mining the same OCEL log on every chain-verify or qa-cycle call.
 *
 * Ported from ~/truex/packages/observability/src/conformance-cache.ts with
 * simplifications for the rocket-craft single-node Nitro server context
 * (no Redis, no distributed invalidation — pure in-process LRU).
 *
 * Key design:
 * - Cache key: `${session_id}:${modelHash}` where modelHash = BLAKE3 of the
 *   sorted declared lifecycle array. Different lifecycle declarations produce
 *   different cache buckets.
 * - TTL: 10 minutes (events can arrive after the first conformance check;
 *   any new ocel-ingest for the session must invalidate the cache entry).
 * - Capacity: 512 entries (each entry ~1 KB; ~512 KB total ceiling).
 * - Invalidation: call `invalidateSession(session_id)` from ocel-ingest after
 *   successful insert to force re-mining on the next chain-verify call.
 */

import type { ConformanceResult } from './processMining';

const CAPACITY = 512;
const TTL_MS = 10 * 60 * 1000; // 10 minutes

interface CacheEntry {
  result: ConformanceResult;
  timestamp: number;
}

// Module-level singleton — persists across Nitro worker requests in the same process
const cache = new Map<string, CacheEntry>();

function makeKey(sessionId: string, declaredLifecycle: string[]): string {
  // Stable key: session + sorted lifecycle (order-insensitive for the key)
  return `${sessionId}:${[...declaredLifecycle].sort().join(',')}`;
}

/** Store a conformance result. Evicts LRU entry if at capacity. */
export function cacheConformance(
  sessionId: string,
  declaredLifecycle: string[],
  result: ConformanceResult,
): void {
  const key = makeKey(sessionId, declaredLifecycle);
  // LRU: delete-then-reinsert moves entry to tail of Map insertion order
  cache.delete(key);
  if (cache.size >= CAPACITY) {
    // Evict head (least recently used)
    const oldest = cache.keys().next().value;
    if (oldest) cache.delete(oldest);
  }
  cache.set(key, { result, timestamp: Date.now() });
}

/** Retrieve a cached result, or null if missing/expired. */
export function getCachedConformance(
  sessionId: string,
  declaredLifecycle: string[],
): ConformanceResult | null {
  const key = makeKey(sessionId, declaredLifecycle);
  const entry = cache.get(key);
  if (!entry) return null;
  if (Date.now() - entry.timestamp > TTL_MS) {
    cache.delete(key);
    return null;
  }
  // LRU refresh
  cache.delete(key);
  cache.set(key, entry);
  return entry.result;
}

/**
 * Invalidate all cache entries for a session.
 * Call from ocel-ingest after successful insert — new events change conformance.
 */
export function invalidateSession(sessionId: string): void {
  for (const key of cache.keys()) {
    if (key.startsWith(`${sessionId}:`)) {
      cache.delete(key);
    }
  }
}

/** Current cache stats — useful for health-lies / dashboard-stats. */
export function conformanceCacheStats(): { entries: number; capacity: number; ttl_ms: number } {
  return { entries: cache.size, capacity: CAPACITY, ttl_ms: TTL_MS };
}
