// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  validateEventHash,
  validateRequiredFields,
  findSeqGap,
  findChainBreak,
  admitBatch,
  type OcelEventCandidate,
} from '../../server/utils/ocelAdmission'

const HEX64 = 'a'.repeat(64)
const HEX64_B = 'b'.repeat(64)
const HEX64_C = 'c'.repeat(64)

function makeEvt(seq: number, prev: string | null, hash: string = HEX64): OcelEventCandidate {
  return {
    activity: 'GameSessionStarted',
    timestamp_ms: 1_750_000_000_000 + seq * 1000,
    object_refs: ['session-1'],
    attributes: {},
    event_hash: hash,
    prev_hash: prev,
    seq,
  }
}

// ── validateEventHash ─────────────────────────────────────────────────────────

describe('validateEventHash', () => {
  it('accepts 64 lowercase hex', () => expect(validateEventHash('a'.repeat(64))).toBe(true))
  it('accepts mixed hex digits', () => expect(validateEventHash('0123456789abcdef'.repeat(4))).toBe(true))
  it('rejects 63-char string', () => expect(validateEventHash('a'.repeat(63))).toBe(false))
  it('rejects 65-char string', () => expect(validateEventHash('a'.repeat(65))).toBe(false))
  it('rejects uppercase', () => expect(validateEventHash('A'.repeat(64))).toBe(false))
  it('rejects non-hex chars', () => expect(validateEventHash('g'.repeat(64))).toBe(false))
  it('rejects null', () => expect(validateEventHash(null)).toBe(false))
  it('rejects number', () => expect(validateEventHash(12345)).toBe(false))
})

// ── validateRequiredFields ────────────────────────────────────────────────────

describe('validateRequiredFields', () => {
  const good: OcelEventCandidate = { activity: 'A', timestamp_ms: 1000, seq: 0, event_hash: HEX64, prev_hash: null }
  it('accepts valid event', () => expect(validateRequiredFields(good)).toBe(true))
  it('rejects empty activity', () => expect(validateRequiredFields({ ...good, activity: '' })).toBe(false))
  it('rejects zero timestamp', () => expect(validateRequiredFields({ ...good, timestamp_ms: 0 })).toBe(false))
  it('rejects negative seq', () => expect(validateRequiredFields({ ...good, seq: -1 })).toBe(false))
  it('rejects bad event_hash', () => expect(validateRequiredFields({ ...good, event_hash: 'bad' })).toBe(false))
})

// ── findSeqGap ────────────────────────────────────────────────────────────────

describe('findSeqGap', () => {
  it('contiguous 0,1,2 → -1', () => {
    const evts = [makeEvt(0, null), makeEvt(1, HEX64), makeEvt(2, HEX64)]
    expect(findSeqGap(evts, 0)).toBe(-1)
  })

  it('gap at index 1 (seq 0,2) → 1', () => {
    const evts = [makeEvt(0, null), makeEvt(2, HEX64)]
    expect(findSeqGap(evts, 0)).toBe(1)
  })

  it('single event contiguous', () => {
    expect(findSeqGap([makeEvt(5, null)], 5)).toBe(-1)
  })

  it('starts at expectedStartSeq=3 correctly', () => {
    const evts = [makeEvt(3, null), makeEvt(4, HEX64), makeEvt(5, HEX64)]
    expect(findSeqGap(evts, 3)).toBe(-1)
  })

  it('duplicate seq → gap detected', () => {
    const evts = [makeEvt(0, null), makeEvt(0, HEX64)] // duplicate seq=0
    expect(findSeqGap(evts, 0)).toBe(1)
  })

  // Property: contiguous array always returns -1
  it('∀ contiguous arrays → -1 (property)', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 10 }),
        fc.integer({ min: 1, max: 10 }),
        (start, count) => {
          const evts = Array.from({ length: count }, (_, i) => makeEvt(start + i, null))
          expect(findSeqGap(evts, start)).toBe(-1)
        },
      ),
    )
  })
})

// ── findChainBreak ─────────────────────────────────────────────────────────────

describe('findChainBreak', () => {
  it('first event with null prev → ok', () => {
    expect(findChainBreak([makeEvt(0, null, HEX64)], null)).toBe(-1)
  })

  it('first event with wrong prev → break at 0', () => {
    expect(findChainBreak([makeEvt(0, HEX64_B, HEX64)], null)).toBe(0)
  })

  it('two-event chain intact', () => {
    const evts = [makeEvt(0, null, HEX64), makeEvt(1, HEX64, HEX64_B)]
    expect(findChainBreak(evts, null)).toBe(-1)
  })

  it('two-event chain broken at 1', () => {
    const evts = [makeEvt(0, null, HEX64), makeEvt(1, HEX64_C, HEX64_B)] // wrong prev_hash
    expect(findChainBreak(evts, null)).toBe(1)
  })

  it('append to existing chain (incomingPrevHash != null)', () => {
    const evts = [makeEvt(3, HEX64, HEX64_B)]
    expect(findChainBreak(evts, HEX64)).toBe(-1)
  })

  // Property: tamper any event's prev_hash → break detected
  it('∀ tamper of prev_hash at position i → break at ≤ i (property)', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 5 }),
        fc.integer({ min: 0, max: 4 }),
        (count, tamperIdx) => {
          if (tamperIdx >= count) return // skip invalid combos
          const hashes = Array.from({ length: count }, (_, i) => i.toString(16).padStart(64, '0'))
          const evts: OcelEventCandidate[] = []
          for (let i = 0; i < count; i++) {
            evts.push(makeEvt(i, i === 0 ? null : hashes[i - 1]!, hashes[i]!))
          }
          // Tamper: change prev_hash at tamperIdx to something wrong
          evts[tamperIdx]!.prev_hash = 'f'.repeat(64)
          const breakAt = findChainBreak(evts, null)
          expect(breakAt).toBeLessThanOrEqual(tamperIdx)
          expect(breakAt).toBeGreaterThanOrEqual(0)
        },
      ),
    )
  })
})

// ── admitBatch ────────────────────────────────────────────────────────────────

describe('admitBatch', () => {
  it('valid single-event batch → admit', () => {
    const r = admitBatch({ session_id: 'sess-1', events: [makeEvt(0, null)] })
    expect(r.verdict).toBe('admit')
  })

  it('valid 3-event chain → admit', () => {
    const evts = [
      makeEvt(0, null, HEX64),
      makeEvt(1, HEX64, HEX64_B),
      makeEvt(2, HEX64_B, HEX64_C),
    ]
    const r = admitBatch({ session_id: 'sess-1', events: evts })
    expect(r.verdict).toBe('admit')
  })

  it('missing session_id → reject 400', () => {
    const r = admitBatch({ session_id: '', events: [makeEvt(0, null)] })
    expect(r.verdict).toBe('reject')
    expect(r.statusCode).toBe(400)
  })

  it('empty events → reject 400', () => {
    const r = admitBatch({ session_id: 'sess-1', events: [] })
    expect(r.verdict).toBe('reject')
    expect(r.statusCode).toBe(400)
  })

  it('bad event_hash format → reject 400 with index', () => {
    const evt = { ...makeEvt(0, null), event_hash: 'bad-hash' }
    const r = admitBatch({ session_id: 'sess-1', events: [evt] })
    expect(r.verdict).toBe('reject')
    expect(r.offending_index).toBe(0)
  })

  it('seq gap → reject 422 with index', () => {
    const evts = [makeEvt(0, null, HEX64), makeEvt(2, HEX64, HEX64_B)] // gap: seq 0→2
    const r = admitBatch({ session_id: 'sess-1', events: evts })
    expect(r.verdict).toBe('reject')
    expect(r.statusCode).toBe(422)
    expect(r.offending_index).toBe(1)
  })

  it('prev_hash mismatch → quarantine (tamper evidence)', () => {
    const evts = [
      makeEvt(0, null, HEX64),
      makeEvt(1, HEX64_C, HEX64_B), // wrong prev_hash (should be HEX64)
    ]
    const r = admitBatch({ session_id: 'sess-1', events: evts })
    expect(r.verdict).toBe('quarantine')
    expect(r.statusCode).toBe(422)
    expect(r.offending_index).toBe(1)
  })

  it('session_id mismatch on event → reject 422', () => {
    const evt = { ...makeEvt(0, null), session_id: 'different-session' }
    const r = admitBatch({ session_id: 'sess-1', events: [evt] })
    expect(r.verdict).toBe('reject')
    expect(r.statusCode).toBe(422)
  })

  it('append with incomingPrevHash set correctly → admit', () => {
    const r = admitBatch({
      session_id: 'sess-1',
      events: [makeEvt(3, HEX64, HEX64_B)],
      expectedStartSeq: 3,
      incomingPrevHash: HEX64,
    })
    expect(r.verdict).toBe('admit')
  })

  it('append with wrong incomingPrevHash → quarantine', () => {
    const r = admitBatch({
      session_id: 'sess-1',
      events: [makeEvt(3, HEX64_C, HEX64_B)], // first event prev_hash doesn't match
      expectedStartSeq: 3,
      incomingPrevHash: HEX64, // expected this, but event has HEX64_C
    })
    expect(r.verdict).toBe('quarantine')
  })

  // Chatman Equation: ∀ valid structured batch → admit
  it('∀ correctly structured batch → admit (property)', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 8 }),
        (count) => {
          const hashes = Array.from({ length: count }, (_, i) => i.toString(16).padStart(64, '0'))
          const evts: OcelEventCandidate[] = Array.from({ length: count }, (_, i) => ({
            activity: 'A',
            timestamp_ms: 1_000_000 + i * 1000,
            seq: i,
            event_hash: hashes[i]!,
            prev_hash: i === 0 ? null : hashes[i - 1]!,
          }))
          const r = admitBatch({ session_id: 'sess-prop', events: evts })
          expect(r.verdict).toBe('admit')
        },
      ),
    )
  })
})
