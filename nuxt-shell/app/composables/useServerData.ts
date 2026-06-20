/**
 * useServerData — hydration-safe payload caching composable.
 *
 * Pattern from ~/dashboard.bak/app/composables/useServerData.ts, adapted for
 * rocket-craft's Nuxt 3 setup.
 *
 * Why this exists vs. Nitro KV (pipeline-health.get.ts):
 *   - Nitro KV caches between *requests* (server-side, cleared per deploy)
 *   - This caches *within* one SSR→hydration cycle: the server computes once,
 *     serialises the result into the HTML payload, and the browser rehydrates
 *     without a second round-trip. Eliminates the "loading…" flash for
 *     above-the-fold data like pipeline health and daily stats.
 *
 * Usage:
 *   const health = await useServerData('pipeline:health', () =>
 *     $fetch('/api/game/pipeline-health'), { ttl: 20_000 })
 */

interface ServerDataOptions {
  /** Client-side revalidation TTL in ms (default: 60 s). 0 = never revalidate. */
  ttl?: number;
}

/**
 * Returns `T | null`. On the server: runs `fetcher()`, stores result in Nuxt
 * payload under `key`. On the client: reads from payload (instant rehydration),
 * then revalidates after `ttl` ms if the data is stale.
 */
export async function useServerData<T = unknown>(
  key: string,
  fetcher: () => Promise<T>,
  options: ServerDataOptions = {},
): Promise<Ref<T | null>> {
  const { ttl = 60_000 } = options;
  const nuxtApp = useNuxtApp();

  // Shared reactive ref — server writes, client reads from payload then watches.
  const result = useState<T | null>(`sdata:${key}`, () => null);

  if (import.meta.server) {
    // Server: always fetch fresh and stash in payload.
    try {
      result.value = await fetcher();
      // nuxtApp.payload.data is persisted into the HTML payload for hydration.
      nuxtApp.payload.data[`sdata:${key}`] = result.value;
    } catch (e) {
      console.warn(`[useServerData] fetch failed for key="${key}":`, e);
    }
    return result;
  }

  // Client: restore from payload first (zero network cost for initial render).
  const fromPayload = nuxtApp.payload.data[`sdata:${key}`] as T | undefined;
  if (fromPayload !== undefined) {
    result.value = fromPayload;
  }

  // Client-side revalidation — fire after TTL so navigation stays snappy.
  if (ttl > 0 && import.meta.client) {
    setTimeout(async () => {
      try {
        result.value = await fetcher();
      } catch (e) {
        console.warn(`[useServerData] revalidation failed for key="${key}":`, e);
      }
    }, ttl);
  }

  return result;
}
