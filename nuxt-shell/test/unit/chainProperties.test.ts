// @vitest-environment happy-dom
/**
 * chainProperties.test.ts — property-based proofs for BLAKE3 chain + Merkle root.
 *
 * Ported from truex's deterministic-convergence + tamper-resistance pattern.
 * Uses fast-check to generate thousands of random inputs and prove invariants
 * that exhaustive hand-written tests cannot cover:
 *
 *   1. Deterministic convergence: same input → same Merkle root (always)
 *   2. Tamper resistance: any leaf mutation changes the root
 *   3. Append-monotonicity: root(A) ≠ root(A + extra_leaf) — no hash collision from appending
 *   4. Empty sentinel: null for zero-length input, always
 *   5. Single-leaf identity: single hash returns itself unchanged
 *   6. Odd-leaf duplication: root([A,B,C]) === root([A,B,C,C]) always
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { computeMerkleRoot } from '../../server/utils/merkle'

// Arbitrary: a 64-char lowercase hex string (valid BLAKE3 event_hash shape)
// Build 64-char lowercase hex strings via fast-check without deprecated APIs
const hexNybble = fc.integer({ min: 0, max: 15 }).map(n => n.toString(16))
const hexHash = fc.array(hexNybble, { minLength: 64, maxLength: 64 }).map(a => a.join(''))

// Arbitrary: a non-empty list of hex hashes (1..8 elements)
const hashList = fc.array(hexHash, { minLength: 1, maxLength: 8 })

describe('computeMerkleRoot — property-based proofs', () => {
  // ── Invariant 1: Deterministic convergence ─────────────────────────────────

  it('same input always produces the same root (determinism)', () => {
    fc.assert(
      fc.property(hashList, (hashes) => {
        const r1 = computeMerkleRoot(hashes)
        const r2 = computeMerkleRoot([...hashes]) // fresh copy, same values
        expect(r1).toBe(r2)
      }),
    )
  })

  // ── Invariant 2: Tamper resistance — any leaf mutation changes the root ────

  it('mutating any single leaf changes the root', () => {
    const differentHash = hexHash

    fc.assert(
      fc.property(
        fc.array(hexHash, { minLength: 1, maxLength: 6 }),
        fc.integer({ min: 0, max: 5 }),
        differentHash,
        (hashes, indexUnclamped, newLeaf) => {
          if (hashes.length === 0) return // skip empty (fc won't generate this but guard anyway)
          const idx = indexUnclamped % hashes.length
          const original = hashes[idx]!

          // Only test when the replacement is actually different
          if (newLeaf === original) return

          const rootBefore = computeMerkleRoot(hashes)
          const tampered = [...hashes]
          tampered[idx] = newLeaf
          const rootAfter = computeMerkleRoot(tampered)

          expect(rootBefore).not.toBe(rootAfter)
        },
      ),
    )
  })

  // ── Invariant 3: Append-monotonicity ──────────────────────────────────────

  it('appending a distinct extra leaf changes the root', () => {
    fc.assert(
      fc.property(
        fc.array(hexHash, { minLength: 1, maxLength: 6 }),
        hexHash,
        (hashes, extra) => {
          // If extra is the same as the last leaf, odd-leaf duplication would
          // make [A,B,C] === [A,B,C,C] — that's the *intended* Bitcoin convention,
          // not a bug. Only assert when the extra is different from the last leaf.
          const lastLeaf = hashes[hashes.length - 1]
          if (extra === lastLeaf) return

          const rootBefore = computeMerkleRoot(hashes)
          const rootAfter = computeMerkleRoot([...hashes, extra])
          expect(rootBefore).not.toBe(rootAfter)
        },
      ),
    )
  })

  // ── Invariant 4: Empty sentinel ────────────────────────────────────────────

  it('empty list always returns null regardless of anything', () => {
    // Exhaustive: the only input is [], so no fc needed — but framing it as a
    // property makes the contract explicit and visible in the test output.
    expect(computeMerkleRoot([])).toBeNull()
  })

  // ── Invariant 5: Single-leaf identity ─────────────────────────────────────

  it('single leaf always returns itself unchanged', () => {
    fc.assert(
      fc.property(hexHash, (h) => {
        expect(computeMerkleRoot([h])).toBe(h)
      }),
    )
  })

  // ── Invariant 6: Odd-leaf duplication (Bitcoin Merkle convention) ──────────

  it('root([A…Z]) === root([A…Z, Z]) — last leaf duplicated', () => {
    fc.assert(
      fc.property(
        fc.array(hexHash, { minLength: 2, maxLength: 7 }).filter(a => a.length % 2 === 1),
        (oddList) => {
          const withDuplicate = [...oddList, oddList[oddList.length - 1]!]
          expect(computeMerkleRoot(oddList)).toBe(computeMerkleRoot(withDuplicate))
        },
      ),
    )
  })

  // ── Invariant 7: Output is always 64-char lowercase hex ───────────────────

  it('non-empty input always produces a 64-char lowercase hex root', () => {
    fc.assert(
      fc.property(hashList, (hashes) => {
        const root = computeMerkleRoot(hashes)
        expect(root).toMatch(/^[0-9a-f]{64}$/)
      }),
    )
  })
})
