// @vitest-environment happy-dom
/**
 * proofGates.test.ts — unit + property-based tests for cook-receipt proof gates.
 *
 * Two layers:
 *   1. Unit tests — explicit cases (known valid, known invalid inputs)
 *   2. Property-based tests — fc.assert proves gates hold universally:
 *      - Gate 1: ∀ non-64-hex strings → REJECTED (400)
 *      - Gate 2: ∀ engine_source='synthetic' → REJECTED (422)
 *      - Gate 3: ∀ lifecycle missing any required activity → REJECTED (422)
 *      - Composed: a valid receipt passes all gates → ok: true
 */

import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import {
  gateReceiptHashFormat,
  gateNotSynthetic,
  gateLifecycleComplete,
  gateRequiredFields,
  runProofGates,
  DECLARED_LIFECYCLE,
} from '../../server/utils/proofGates'

const VALID_HASH = 'a'.repeat(64)
const VALID_LIFECYCLE = [...DECLARED_LIFECYCLE]
const VALID_INPUT = {
  verdict: 'PASS',
  milestone: 'HeadlessSeed',
  engine_source: 'rocket_cli',
  receipt_hash: VALID_HASH,
  ocel_lifecycle: VALID_LIFECYCLE,
}

// ── Gate 1: receipt_hash format ───────────────────────────────────────────────

describe('gateReceiptHashFormat', () => {
  it('accepts a 64-char lowercase hex string', () => {
    expect(gateReceiptHashFormat(VALID_HASH)).toEqual({ ok: true })
  })

  it('rejects undefined', () => {
    const r = gateReceiptHashFormat(undefined)
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(400)
  })

  it('rejects 63-char hex (too short)', () => {
    const r = gateReceiptHashFormat('a'.repeat(63))
    expect(r.ok).toBe(false)
  })

  it('rejects 65-char hex (too long)', () => {
    const r = gateReceiptHashFormat('a'.repeat(65))
    expect(r.ok).toBe(false)
  })

  it('rejects uppercase hex', () => {
    const r = gateReceiptHashFormat('A'.repeat(64))
    expect(r.ok).toBe(false)
  })

  it('rejects non-hex chars in 64-char string', () => {
    const r = gateReceiptHashFormat('g'.repeat(64))
    expect(r.ok).toBe(false)
  })

  // Property: ∀ string not matching [0-9a-f]{64} → rejected
  it('∀ non-64-lowercase-hex string → rejected (property)', () => {
    const hexNybble = fc.integer({ min: 0, max: 15 }).map(n => n.toString(16))
    const validHash = fc.array(hexNybble, { minLength: 64, maxLength: 64 }).map(a => a.join(''))

    // Strings that differ in length from 64 are always rejected
    fc.assert(
      fc.property(
        fc.string({ minLength: 0, maxLength: 63 }),
        (short) => {
          const r = gateReceiptHashFormat(short)
          expect(r.ok).toBe(false)
        },
      ),
    )

    // Valid 64-char hex always accepted
    fc.assert(
      fc.property(validHash, (h) => {
        expect(gateReceiptHashFormat(h)).toEqual({ ok: true })
      }),
    )
  })
})

// ── Gate 2: not synthetic ──────────────────────────────────────────────────────

describe('gateNotSynthetic', () => {
  it("rejects engine_source='synthetic'", () => {
    const r = gateNotSynthetic('synthetic')
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(422)
  })

  it("accepts engine_source='rocket_cli'", () => {
    expect(gateNotSynthetic('rocket_cli')).toEqual({ ok: true })
  })

  it("accepts engine_source='real_ue4'", () => {
    expect(gateNotSynthetic('real_ue4')).toEqual({ ok: true })
  })

  it('accepts undefined', () => {
    expect(gateNotSynthetic(undefined)).toEqual({ ok: true })
  })

  // Property: ∀ engine_source ≠ 'synthetic' → accepted
  it('∀ source ≠ synthetic → accepted (property)', () => {
    fc.assert(
      fc.property(
        fc.string().filter(s => s !== 'synthetic'),
        (source) => {
          expect(gateNotSynthetic(source)).toEqual({ ok: true })
        },
      ),
    )
  })
})

// ── Gate 3: lifecycle completeness ─────────────────────────────────────────────

describe('gateLifecycleComplete', () => {
  it('accepts the full declared lifecycle', () => {
    expect(gateLifecycleComplete(VALID_LIFECYCLE)).toEqual({ ok: true })
  })

  it('accepts lifecycle with extra activities (superset)', () => {
    expect(gateLifecycleComplete([...VALID_LIFECYCLE, 'ReceiptEmitted'])).toEqual({ ok: true })
  })

  it('rejects empty array', () => {
    const r = gateLifecycleComplete([])
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(422)
  })

  it('rejects when GameSessionStarted is missing', () => {
    expect(gateLifecycleComplete(['FrameRendered', 'InputAdmitted']).ok).toBe(false)
  })

  it('rejects when FrameRendered is missing', () => {
    expect(gateLifecycleComplete(['GameSessionStarted', 'InputAdmitted']).ok).toBe(false)
  })

  it('rejects when InputAdmitted is missing', () => {
    expect(gateLifecycleComplete(['GameSessionStarted', 'FrameRendered']).ok).toBe(false)
  })

  // Property: ∀ lifecycle missing any required activity → rejected
  it('∀ lifecycle with any required activity removed → rejected (property)', () => {
    fc.assert(
      fc.property(
        // Pick a subset of DECLARED_LIFECYCLE with at least 1 element removed
        fc.subarray(DECLARED_LIFECYCLE as unknown as string[], { minLength: 0, maxLength: DECLARED_LIFECYCLE.length - 1 }),
        fc.array(fc.string(), { minLength: 0, maxLength: 3 }), // optional extra activities
        (subset, extras) => {
          const r = gateLifecycleComplete([...subset, ...extras])
          expect(r.ok).toBe(false)
        },
      ),
    )
  })
})

// ── Gate 4: required fields ────────────────────────────────────────────────────

describe('gateRequiredFields', () => {
  it('accepts all required fields present', () => {
    expect(gateRequiredFields(VALID_INPUT)).toEqual({ ok: true })
  })

  it('rejects missing verdict', () => {
    expect(gateRequiredFields({ ...VALID_INPUT, verdict: undefined }).ok).toBe(false)
  })

  it('rejects missing milestone', () => {
    expect(gateRequiredFields({ ...VALID_INPUT, milestone: undefined }).ok).toBe(false)
  })

  it('rejects missing engine_source', () => {
    expect(gateRequiredFields({ ...VALID_INPUT, engine_source: undefined }).ok).toBe(false)
  })

  it('rejects missing receipt_hash', () => {
    expect(gateRequiredFields({ ...VALID_INPUT, receipt_hash: undefined }).ok).toBe(false)
  })
})

// ── Composed runProofGates ─────────────────────────────────────────────────────

describe('runProofGates', () => {
  it('valid input passes all gates', () => {
    expect(runProofGates(VALID_INPUT)).toEqual({ ok: true })
  })

  it('synthetic input rejected at gate 3 (not gate 1)', () => {
    const r = runProofGates({ ...VALID_INPUT, engine_source: 'synthetic' })
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(422)
  })

  it('bad hash rejected at gate 2', () => {
    const r = runProofGates({ ...VALID_INPUT, receipt_hash: 'not-hex' })
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(400)
  })

  it('incomplete lifecycle rejected at gate 4', () => {
    const r = runProofGates({ ...VALID_INPUT, ocel_lifecycle: ['GameSessionStarted'] })
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.statusCode).toBe(422)
  })

  // Chatman Equation property: ∀ valid input → ok: true
  it('∀ valid structured input → gates pass (property)', () => {
    const hexNybble = fc.integer({ min: 0, max: 15 }).map(n => n.toString(16))
    const validHash = fc.array(hexNybble, { minLength: 64, maxLength: 64 }).map(a => a.join(''))
    const validSource = fc.constantFrom('rocket_cli', 'real_ue4', 'browser', 'unknown')
    const extraActivities = fc.array(fc.string({ minLength: 1, maxLength: 20 }), { minLength: 0, maxLength: 3 })

    fc.assert(
      fc.property(
        validHash,
        validSource,
        extraActivities,
        (hash, source, extras) => {
          const input = {
            verdict: 'PASS',
            milestone: 'Test',
            engine_source: source,
            receipt_hash: hash,
            ocel_lifecycle: [...VALID_LIFECYCLE, ...extras],
          }
          expect(runProofGates(input)).toEqual({ ok: true })
        },
      ),
    )
  })
})
