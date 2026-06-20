// @vitest-environment happy-dom
/**
 * merkle.test.ts — BLAKE3 Merkle root computation tests.
 *
 * The Merkle root over session event_hash values provides a membership
 * commitment: if any event is deleted or reordered, the root changes.
 * Combined with the linear prev_hash chain, both ordering and completeness
 * are provable from a single root hash.
 *
 * Algorithm: binary tree, odd leaves duplicated (Bitcoin Merkle convention).
 * Each parent = BLAKE3(left_hex_string + right_hex_string).
 */

import { describe, it, expect } from 'vitest'
import { computeMerkleRoot } from '../../server/utils/merkle'

const FAKE_HASH = (n: number) => n.toString(16).padStart(64, '0')

describe('computeMerkleRoot', () => {
  // ── Edge cases ────────────────────────────────────────────────────────────

  it('empty list returns null', () => {
    expect(computeMerkleRoot([])).toBeNull()
  })

  it('single hash returns itself (no computation)', () => {
    const h = FAKE_HASH(1)
    expect(computeMerkleRoot([h])).toBe(h)
  })

  // ── Output shape ──────────────────────────────────────────────────────────

  it('two hashes produces a 64-char lowercase hex root', () => {
    const root = computeMerkleRoot([FAKE_HASH(1), FAKE_HASH(2)])
    expect(root).toHaveLength(64)
    expect(root).toMatch(/^[0-9a-f]{64}$/)
  })

  it('three hashes produces a 64-char root (odd leaf duplicated)', () => {
    const root = computeMerkleRoot([FAKE_HASH(1), FAKE_HASH(2), FAKE_HASH(3)])
    expect(root).toHaveLength(64)
    expect(root).toMatch(/^[0-9a-f]{64}$/)
  })

  it('power-of-two count produces a 64-char root', () => {
    const hashes = [1, 2, 3, 4].map(FAKE_HASH)
    expect(computeMerkleRoot(hashes)).toHaveLength(64)
  })

  // ── Determinism ───────────────────────────────────────────────────────────

  it('same inputs always produce the same root', () => {
    const hashes = [1, 2, 3].map(FAKE_HASH)
    expect(computeMerkleRoot(hashes)).toBe(computeMerkleRoot(hashes))
  })

  it('different inputs produce different roots', () => {
    const root1 = computeMerkleRoot([FAKE_HASH(1), FAKE_HASH(2)])
    const root2 = computeMerkleRoot([FAKE_HASH(1), FAKE_HASH(3)])
    expect(root1).not.toBe(root2)
  })

  // ── Tamper detection ──────────────────────────────────────────────────────

  it('changing any leaf changes the root', () => {
    const hashes = [FAKE_HASH(1), FAKE_HASH(2), FAKE_HASH(3), FAKE_HASH(4)]
    const original = computeMerkleRoot(hashes)

    for (let i = 0; i < hashes.length; i++) {
      const tampered = [...hashes]
      tampered[i] = FAKE_HASH(99)
      expect(computeMerkleRoot(tampered)).not.toBe(original)
    }
  })

  it('removing a leaf changes the root', () => {
    const hashes = [FAKE_HASH(1), FAKE_HASH(2), FAKE_HASH(3)]
    const original = computeMerkleRoot(hashes)
    const shorter = computeMerkleRoot([FAKE_HASH(1), FAKE_HASH(2)])
    expect(original).not.toBe(shorter)
  })

  it('reordering leaves changes the root', () => {
    const h1 = FAKE_HASH(1)
    const h2 = FAKE_HASH(2)
    const root1 = computeMerkleRoot([h1, h2])
    const root2 = computeMerkleRoot([h2, h1])
    expect(root1).not.toBe(root2)
  })

  // ── Odd-leaf duplication convention ───────────────────────────────────────

  it('odd list (3 leaves): last leaf is paired with itself at level boundary', () => {
    // [A, B, C] → level1: [hash(A+B), hash(C+C)] → root: hash(hash(A+B)+hash(C+C))
    // Adding a 4th identical-to-3rd leaf should match
    const h = [FAKE_HASH(1), FAKE_HASH(2), FAKE_HASH(3)]
    const odd3 = computeMerkleRoot(h)
    const even4 = computeMerkleRoot([...h, FAKE_HASH(3)]) // duplicate last leaf
    expect(odd3).toBe(even4)
  })
})
