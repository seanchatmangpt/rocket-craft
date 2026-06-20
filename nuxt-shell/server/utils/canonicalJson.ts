/**
 * Recursive canonical JSON — the SINGLE source of truth for OCEL event hashing
 * across browser, Nuxt server, and the Rust CLI.
 *
 * Byte-matches rocket-sdk/src/supabase.rs::canonical_json:
 *   - objects: keys sorted recursively, serialized as {"k":v,...} (no spaces)
 *   - arrays: elements in order, [a,b,...]
 *   - scalars: JSON.stringify (=== serde_json::to_string for our JSON-safe data)
 *
 * Why recursive (not the old `JSON.stringify(obj, Object.keys(obj).sort())`):
 * the array-replacer form applies the top-level key list at ALL nesting levels,
 * silently DROPPING nested `attributes` keys from the hash. That made the hash
 * ignore attribute contents AND diverge from the Rust CLI (which hashes them) —
 * so Rust-cooked sessions replayed as hash_convergent=false. Recursive covers
 * attributes and converges with Rust + evidence-pack.
 */
export function canonicalJSON(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'string') {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map(canonicalJSON).join(',')}]`;
  }
  const obj = value as Record<string, unknown>;
  const keys = Object.keys(obj).sort();
  return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalJSON(obj[k])}`).join(',')}}`;
}
