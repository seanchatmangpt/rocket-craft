import { describe, it, expect } from 'vitest'
import {
  createResidual, publishResidual, filterBlockers, hasBlockers, residualFromError,
} from '../../src/residuals.js'

describe('residuals', () => {
  it('createResidual returns correct shape', () => {
    const r = createResidual('TEST-001', 'surface', 'test message', 'blocker', 'fix it')
    expect(r.code).toBe('TEST-001')
    expect(r.surface).toBe('surface')
    expect(r.severity).toBe('blocker')
    expect(r.repair_candidate).toBe('fix it')
  })

  it('publishResidual appends to array', () => {
    const arr = createResidual('A', 's', 'm')
    const arr2: typeof arr[] = []
    publishResidual(arr2, arr)
    expect(arr2).toHaveLength(1)
  })

  it('filterBlockers returns only blockers', () => {
    const residuals = [
      createResidual('A', 's', 'm', 'info'),
      createResidual('B', 's', 'm', 'blocker'),
      createResidual('C', 's', 'm', 'warn'),
    ]
    const blockers = filterBlockers(residuals)
    expect(blockers).toHaveLength(1)
    expect(blockers[0].code).toBe('B')
  })

  it('hasBlockers returns false when no blockers', () => {
    const residuals = [createResidual('A', 's', 'm', 'info')]
    expect(hasBlockers(residuals)).toBe(false)
  })

  it('residualFromError wraps Error message', () => {
    const r = residualFromError(new Error('boom'), 'surface')
    expect(r.message).toBe('boom')
    expect(r.severity).toBe('error')
  })

  it('residualFromError handles non-Error objects', () => {
    const r = residualFromError('plain string error', 'surface')
    expect(r.message).toBe('plain string error')
  })
})
