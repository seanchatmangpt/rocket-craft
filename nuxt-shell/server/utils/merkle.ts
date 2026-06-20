/**
 * Server-side BLAKE3 Merkle tree — mirrors useHashChain.computeMerkleRoot.
 *
 * Binary tree, odd leaves duplicated (same convention as Bitcoin Merkle trees).
 * The root is BLAKE3(left_hex + right_hex) at each level.
 *
 * Single hash → returns that hash (no computation needed).
 * Empty list  → returns null.
 *
 * Used by chain-verify and evidence-pack to produce a membership commitment
 * over all event_hash values in a session.
 */

import { blake3 } from '@noble/hashes/blake3.js';

function blake3Hex(input: string): string {
  const bytes = blake3(new TextEncoder().encode(input));
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

export function computeMerkleRoot(hashes: string[]): string | null {
  if (!hashes.length) return null;
  if (hashes.length === 1) return hashes[0]!;

  let level = [...hashes];
  while (level.length > 1) {
    const next: string[] = [];
    for (let i = 0; i < level.length; i += 2) {
      const left = level[i]!;
      const right = level[i + 1] ?? left; // duplicate odd leaf
      next.push(blake3Hex(left + right));
    }
    level = next;
  }
  return level[0]!;
}
