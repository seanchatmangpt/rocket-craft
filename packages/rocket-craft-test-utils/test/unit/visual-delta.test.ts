import { describe, it, expect } from 'vitest'
import { computePixelDelta, hashBuffer, buildVisualDeltaResult } from '../../src/visual-delta.js'

describe('visual-delta', () => {
  it('identical buffers produce zero delta', () => {
    const buf = Buffer.from([1, 2, 3, 4, 5])
    expect(computePixelDelta(buf, buf)).toBe(0)
  })

  it('different buffers produce non-zero delta', () => {
    const a = Buffer.from([1, 2, 3])
    const b = Buffer.from([1, 9, 3])
    expect(computePixelDelta(a, b)).toBe(1)
  })

  it('buffers of different lengths use max length', () => {
    const a = Buffer.from([1, 2])
    const b = Buffer.from([1, 2, 3])
    expect(computePixelDelta(a, b)).toBe(1) // extra byte at index 2
  })

  it('hashBuffer returns 64-char hex string', () => {
    const h = hashBuffer(Buffer.from('hello'))
    expect(h).toHaveLength(64)
    expect(/^[0-9a-f]+$/.test(h)).toBe(true)
  })

  it('hashBuffer is deterministic', () => {
    const buf = Buffer.from('same content')
    expect(hashBuffer(buf)).toBe(hashBuffer(buf))
  })

  it('buildVisualDeltaResult admits when pixels changed', () => {
    const baseline = Buffer.from(new Array(1000).fill(0))
    const after = Buffer.from(new Array(1000).fill(255))
    const result = buildVisualDeltaResult(baseline, after, { min_changed_pixels: 1 })
    expect(result.admitted).toBe(true)
    expect(result.changed_pixels).toBe(1000)
    expect(result.residuals).toHaveLength(0)
  })

  it('buildVisualDeltaResult refuses when delta is zero', () => {
    const buf = Buffer.from([1, 2, 3])
    const result = buildVisualDeltaResult(buf, buf, { min_changed_pixels: 1 })
    expect(result.admitted).toBe(false)
    expect(result.residuals.some(r => r.code === 'VISUAL-DELTA-ZERO')).toBe(true)
  })
})
