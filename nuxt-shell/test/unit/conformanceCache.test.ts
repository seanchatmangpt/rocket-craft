import { describe, it, expect } from 'vitest';
import {
  cacheConformance,
  getCachedConformance,
  invalidateSession,
  conformanceCacheStats,
} from '../../server/utils/conformanceCache';
import type { ConformanceResult } from '../../server/utils/processMining';

const LIFECYCLE = ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'];

function makeResult(score: number): ConformanceResult {
  return {
    fitness: score,
    precision: score,
    simplicity: score,
    generalization: score,
    overall_score: score,
    variants_discovered: 1,
    deviation_points: [],
    traces: [LIFECYCLE],
  };
}

describe('conformanceCache', () => {
  // Note: the module-level cache is shared across tests.
  // Use distinct sessionIds per test to avoid cross-test contamination.

  it('miss on empty cache returns null', () => {
    expect(getCachedConformance('no-such-session', LIFECYCLE)).toBeNull();
  });

  it('store then retrieve returns same result', () => {
    const result = makeResult(0.95);
    cacheConformance('sess-a', LIFECYCLE, result);
    const hit = getCachedConformance('sess-a', LIFECYCLE);
    expect(hit).not.toBeNull();
    expect(hit!.overall_score).toBe(0.95);
    expect(hit!.fitness).toBe(0.95);
  });

  it('different lifecycle produces different cache key (no cross-contamination)', () => {
    const result = makeResult(0.8);
    cacheConformance('sess-b', LIFECYCLE, result);
    // Different lifecycle → different key → miss
    const other = getCachedConformance('sess-b', ['A', 'B']);
    expect(other).toBeNull();
  });

  it('lifecycle key is order-insensitive (sorted)', () => {
    const result = makeResult(0.9);
    cacheConformance('sess-c', ['C', 'A', 'B'], result);
    // Retrieve with different order — should hit same slot
    const hit = getCachedConformance('sess-c', ['A', 'B', 'C']);
    expect(hit).not.toBeNull();
    expect(hit!.overall_score).toBe(0.9);
  });

  it('invalidateSession evicts all entries for that session', () => {
    cacheConformance('sess-d', LIFECYCLE, makeResult(0.7));
    cacheConformance('sess-d', ['A', 'B'], makeResult(0.6));
    invalidateSession('sess-d');
    expect(getCachedConformance('sess-d', LIFECYCLE)).toBeNull();
    expect(getCachedConformance('sess-d', ['A', 'B'])).toBeNull();
  });

  it('invalidateSession does not evict other sessions', () => {
    cacheConformance('sess-e', LIFECYCLE, makeResult(0.88));
    cacheConformance('sess-f', LIFECYCLE, makeResult(0.77));
    invalidateSession('sess-e');
    expect(getCachedConformance('sess-e', LIFECYCLE)).toBeNull();
    expect(getCachedConformance('sess-f', LIFECYCLE)).not.toBeNull(); // unaffected
  });

  it('conformanceCacheStats returns numeric fields', () => {
    const stats = conformanceCacheStats();
    expect(typeof stats.entries).toBe('number');
    expect(stats.capacity).toBe(512);
    expect(stats.ttl_ms).toBe(10 * 60 * 1000);
  });

  it('second store with same key overwrites result', () => {
    cacheConformance('sess-g', LIFECYCLE, makeResult(0.5));
    cacheConformance('sess-g', LIFECYCLE, makeResult(0.99));
    const hit = getCachedConformance('sess-g', LIFECYCLE);
    expect(hit!.overall_score).toBe(0.99);
  });
});
