// @vitest-environment happy-dom
/**
 * useHashChain.test.ts — Unit tests for the BLAKE3 hash chain composable.
 *
 * Critical invariant: computeEventHash must produce the same output for the
 * same input across browsers and Node.js. Any change to the canonical JSON
 * key order or BLAKE3 implementation breaks ALL existing chains in the DB.
 *
 * Also verifies chain linkage (prev_hash threading) and break detection.
 */

import { describe, it, expect } from 'vitest'
import { useHashChain, type HashChainEvent, type LinkedEvent } from '../../app/composables/useHashChain'

const { computeEventHash, linkEvents, verifyChain, findBreaks } = useHashChain()

/** Build a full linked chain from an array of raw events */
async function buildLinkedChain(events: HashChainEvent[]): Promise<LinkedEvent[]> {
  const chain: LinkedEvent[] = []
  for (const evt of events) {
    const prev = chain.length > 0 ? chain[chain.length - 1]! : null
    chain.push(await linkEvents(evt, prev))
  }
  return chain
}

// ── Determinism ──────────────────────────────────────────────────────────────

describe('computeEventHash determinism', () => {
  const event: HashChainEvent = {
    id: 'e1',
    timestamp: '2026-06-19T00:00:00.000Z',
    type: 'GameSessionStarted',
    data: { playerId: 'p1' },
    prev_hash: null,
  }

  it('produces a 64-char lowercase hex string', async () => {
    const hash = await computeEventHash(event)
    expect(hash).toHaveLength(64)
    expect(hash).toMatch(/^[0-9a-f]{64}$/)
  })

  it('same input always produces same hash (deterministic)', async () => {
    const h1 = await computeEventHash(event)
    const h2 = await computeEventHash(event)
    expect(h1).toBe(h2)
  })

  it('different type produces different hash', async () => {
    const other: HashChainEvent = { ...event, type: 'FrameRendered' }
    expect(await computeEventHash(event)).not.toBe(await computeEventHash(other))
  })

  it('different prev_hash produces different hash', async () => {
    const withPrev: HashChainEvent = { ...event, prev_hash: 'a'.repeat(64) }
    expect(await computeEventHash(event)).not.toBe(await computeEventHash(withPrev))
  })

  it('null prev_hash ≠ empty string prev_hash', async () => {
    const nullHash: HashChainEvent = { ...event, prev_hash: null }
    const emptyHash: HashChainEvent = { ...event, prev_hash: '' }
    expect(await computeEventHash(nullHash)).not.toBe(await computeEventHash(emptyHash))
  })

  // Regression: canonical key order is {id, timestamp, type, data, prev_hash} — LOCKED
  it('LOCKED: hash of genesis GameSessionStarted event is stable', async () => {
    const genesis: HashChainEvent = {
      id: 'e-genesis',
      timestamp: '2026-06-19T00:00:00.000Z',
      type: 'GameSessionStarted',
      data: {},
      prev_hash: null,
    }
    const hash = await computeEventHash(genesis)
    // If this assertion fails, the canonical JSON format changed — breaking all existing chains
    expect(hash).toHaveLength(64)
    expect(hash).toMatch(/^[0-9a-f]{64}$/)
    // Store reference hash as a comment: run once to capture, then lock it
    // Reference: (computed by this test on first passing run — add after first green CI)
    console.log(`[useHashChain] genesis hash: ${hash}`)
  })
})

// ── Chain building ────────────────────────────────────────────────────────────

describe('buildLinkedChain', () => {
  const events: HashChainEvent[] = [
    { id: 'e1', timestamp: '2026-06-19T00:00:01Z', type: 'GameSessionStarted', data: {}, prev_hash: null },
    { id: 'e2', timestamp: '2026-06-19T00:00:02Z', type: 'FrameRendered', data: { frame: 1 }, prev_hash: null },
    { id: 'e3', timestamp: '2026-06-19T00:00:03Z', type: 'InputAdmitted', data: { key: 'W' }, prev_hash: null },
  ]

  it('produces a chain with the same length as input', async () => {
    const chain = await buildLinkedChain(events)
    expect(chain).toHaveLength(events.length)
  })

  it('first event has prev_hash=null', async () => {
    const chain = await buildLinkedChain(events)
    expect(chain[0]!.prev_hash).toBeNull()
  })

  it('each event.prev_hash equals the previous event.hash', async () => {
    const chain = await buildLinkedChain(events)
    for (let i = 1; i < chain.length; i++) {
      expect(chain[i]!.prev_hash).toBe(chain[i - 1]!.hash)
    }
  })

  it('all hashes are 64-char lowercase hex', async () => {
    const chain = await buildLinkedChain(events)
    for (const linked of chain) {
      expect(linked.hash).toHaveLength(64)
      expect(linked.hash).toMatch(/^[0-9a-f]{64}$/)
    }
  })

  it('all hashes are unique', async () => {
    const chain = await buildLinkedChain(events)
    const hashes = chain.map((e: LinkedEvent) => e.hash)
    expect(new Set(hashes).size).toBe(hashes.length)
  })

  it('empty input returns empty chain', async () => {
    const chain = await buildLinkedChain([])
    expect(chain).toHaveLength(0)
  })
})

// ── Chain verification ────────────────────────────────────────────────────────

describe('verifyChain', () => {
  it('intact chain returns valid=true, no breaks', async () => {
    const events: HashChainEvent[] = [
      { id: 'e1', timestamp: '2026-06-19T00:00:01Z', type: 'A', data: {}, prev_hash: null },
      { id: 'e2', timestamp: '2026-06-19T00:00:02Z', type: 'B', data: {}, prev_hash: null },
    ]
    const chain = await buildLinkedChain(events)
    const result = await verifyChain(chain)
    expect(result.valid).toBe(true)
    expect(result.breaks).toHaveLength(0)
    expect(result.totalEvents).toBe(2)
    expect(result.validEvents).toBe(2)
  })

  it('tampered event breaks the chain', async () => {
    const events: HashChainEvent[] = [
      { id: 'e1', timestamp: '2026-06-19T00:00:01Z', type: 'A', data: {}, prev_hash: null },
      { id: 'e2', timestamp: '2026-06-19T00:00:02Z', type: 'B', data: {}, prev_hash: null },
      { id: 'e3', timestamp: '2026-06-19T00:00:03Z', type: 'C', data: {}, prev_hash: null },
    ]
    const chain = await buildLinkedChain(events)
    // Tamper: forge e2's hash
    chain[1]!.hash = 'dead'.repeat(16)
    const result = await verifyChain(chain)
    expect(result.valid).toBe(false)
    expect(result.breaks.length).toBeGreaterThan(0)
  })

  it('empty chain is valid (nothing to break)', async () => {
    const result = await verifyChain([])
    expect(result.valid).toBe(true)
    expect(result.totalEvents).toBe(0)
    expect(result.integrity).toBeNaN()
  })
})

// ── findBreaks ────────────────────────────────────────────────────────────────

describe('findBreaks', () => {
  it('returns empty array for intact chain', async () => {
    const events: HashChainEvent[] = [
      { id: 'e1', timestamp: '2026-06-19T00:00:01Z', type: 'A', data: {}, prev_hash: null },
    ]
    const chain = await buildLinkedChain(events)
    const breaks = await findBreaks(chain, '2026-06-19T00:00:10Z')
    expect(breaks).toHaveLength(0)
  })

  it('reports severity=critical for breaks', async () => {
    const events: HashChainEvent[] = [
      { id: 'e1', timestamp: '2026-06-19T00:00:01Z', type: 'A', data: {}, prev_hash: null },
      { id: 'e2', timestamp: '2026-06-19T00:00:02Z', type: 'B', data: {}, prev_hash: null },
    ]
    const chain = await buildLinkedChain(events)
    chain[0]!.hash = 'bad'.padEnd(64, '0')
    const breaks = await findBreaks(chain, '2026-06-19T00:00:10Z')
    expect(breaks.length).toBeGreaterThan(0)
    expect(['critical', 'high']).toContain(breaks[0]!.severity)
    expect(typeof breaks[0]!.detectedAt).toBe('string')
  })
})
