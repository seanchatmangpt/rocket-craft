// @vitest-environment happy-dom
/**
 * useAdmitRoute.test.ts
 *
 * Tests for the route admission guard's pure `checkRoute` function.
 * Identity hierarchy: anonymous < authenticated < verified < mfa_verified
 *
 * This is a port of ~/expo-supabase-ai-template auth guard logic to Nuxt.
 * The same invariants must hold:
 *   - anonymous identity is denied any route requiring authentication
 *   - actualIdx < requiredIdx → INSUFFICIENT_IDENTITY_LEVEL
 *   - missing roles → MISSING_ROLE refusal
 *   - no constraints → admitted
 */

import { describe, it, expect } from 'vitest'
import {
  checkRoute,
  type IdentityBoundary,
  type RouteDefinition,
  type AdmitRouteResult,
} from '../../app/composables/useAdmitRoute'

// ── Anonymous identity ────────────────────────────────────────────────────────

describe('checkRoute — anonymous identity', () => {
  it('anonymous → anonymous route: admitted', () => {
    const result: AdmitRouteResult = checkRoute('anonymous', { requiredIdentityBoundary: 'anonymous' }, [])
    expect(result.admitted).toBe(true)
    expect(result.refusal).toBeUndefined()
  })

  it('anonymous → authenticated route: UNAUTHENTICATED refusal', () => {
    const result = checkRoute('anonymous', { requiredIdentityBoundary: 'authenticated' }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('UNAUTHENTICATED')
    expect(result.refusal?.requiredIdentityBoundary).toBe('authenticated')
    expect(result.refusal?.actualIdentityBoundary).toBe('anonymous')
  })

  it('anonymous → verified route: UNAUTHENTICATED (not just insufficient)', () => {
    const result = checkRoute('anonymous', { requiredIdentityBoundary: 'verified' }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('UNAUTHENTICATED')
  })

  it('anonymous → mfa_verified route: UNAUTHENTICATED', () => {
    const result = checkRoute('anonymous', { requiredIdentityBoundary: 'mfa_verified' }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('UNAUTHENTICATED')
  })

  it('anonymous + no requiredIdentityBoundary → admitted', () => {
    const result = checkRoute('anonymous', {}, [])
    expect(result.admitted).toBe(true)
  })
})

// ── Authenticated identity ────────────────────────────────────────────────────

describe('checkRoute — authenticated identity', () => {
  it('authenticated → anonymous route: admitted (supersets allowed)', () => {
    const result = checkRoute('authenticated', { requiredIdentityBoundary: 'anonymous' }, [])
    expect(result.admitted).toBe(true)
  })

  it('authenticated → authenticated route: admitted', () => {
    const result = checkRoute('authenticated', { requiredIdentityBoundary: 'authenticated' }, [])
    expect(result.admitted).toBe(true)
  })

  it('authenticated → verified route: INSUFFICIENT_IDENTITY_LEVEL', () => {
    const result = checkRoute('authenticated', { requiredIdentityBoundary: 'verified' }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('INSUFFICIENT_IDENTITY_LEVEL')
    expect(result.refusal?.actualIdentityBoundary).toBe('authenticated')
    expect(result.refusal?.requiredIdentityBoundary).toBe('verified')
  })

  it('authenticated → mfa_verified route: INSUFFICIENT_IDENTITY_LEVEL', () => {
    const result = checkRoute('authenticated', { requiredIdentityBoundary: 'mfa_verified' }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('INSUFFICIENT_IDENTITY_LEVEL')
  })
})

// ── mfa_verified identity (highest level) ────────────────────────────────────

describe('checkRoute — mfa_verified identity', () => {
  const identities: IdentityBoundary[] = ['anonymous', 'authenticated', 'verified', 'mfa_verified']

  for (const required of identities) {
    it(`mfa_verified → ${required} route: admitted`, () => {
      const result = checkRoute('mfa_verified', { requiredIdentityBoundary: required }, [])
      expect(result.admitted).toBe(true)
    })
  }
})

// ── Role checks ───────────────────────────────────────────────────────────────

describe('checkRoute — role requirements', () => {
  it('has required role → admitted', () => {
    const result = checkRoute('authenticated', { requiredRoles: ['admin'] }, ['admin', 'user'])
    expect(result.admitted).toBe(true)
  })

  it('missing one required role → MISSING_ROLE', () => {
    const result = checkRoute('authenticated', { requiredRoles: ['admin', 'superuser'] }, ['admin'])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.code).toBe('MISSING_ROLE')
    expect(result.refusal?.missingRoles).toContain('superuser')
    expect(result.refusal?.missingRoles).not.toContain('admin')
  })

  it('missing all required roles → MISSING_ROLE with all in missingRoles', () => {
    const result = checkRoute('authenticated', { requiredRoles: ['admin', 'superuser'] }, [])
    expect(result.admitted).toBe(false)
    expect(result.refusal?.missingRoles).toEqual(['admin', 'superuser'])
  })

  it('identity check runs before role check (anonymous still gets UNAUTHENTICATED, not MISSING_ROLE)', () => {
    const result = checkRoute('anonymous', { requiredIdentityBoundary: 'authenticated', requiredRoles: ['admin'] }, [])
    expect(result.refusal?.code).toBe('UNAUTHENTICATED')
  })

  it('no requiredRoles → role check skipped', () => {
    const result = checkRoute('authenticated', { requiredRoles: [] }, [])
    expect(result.admitted).toBe(true)
  })
})

// ── Route with no constraints ─────────────────────────────────────────────────

describe('checkRoute — unconstrained route', () => {
  const identities: IdentityBoundary[] = ['anonymous', 'authenticated', 'verified', 'mfa_verified']

  for (const identity of identities) {
    it(`${identity} → unconstrained route: admitted`, () => {
      const route: RouteDefinition = {}
      const result = checkRoute(identity, route, [])
      expect(result.admitted).toBe(true)
      expect(result.refusal).toBeUndefined()
    })
  }
})
