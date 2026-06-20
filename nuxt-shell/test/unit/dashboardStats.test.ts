// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { aggregateByDay } from '../../server/utils/dashboardStats'

const makeRow = (day: string, verdict: string, source: string, events: number) => ({
  proven_at: `${day}T00:00:00Z`,
  verdict,
  engine_source: source,
  ocel_event_count: events,
})

describe('aggregateByDay', () => {
  it('empty input returns empty array', () => {
    expect(aggregateByDay([])).toEqual([])
  })

  it('single PASS row produces correct bucket', () => {
    const rows = [makeRow('2026-06-19', 'PASS', 'rocket_cli', 3)]
    const result = aggregateByDay(rows)
    expect(result).toHaveLength(1)
    expect(result[0]!.day).toBe('2026-06-19')
    expect(result[0]!.receipts).toBe(1)
    expect(result[0]!.pass_receipts).toBe(1)
    expect(result[0]!.fail_receipts).toBe(0)
    expect(result[0]!.real_ue4_receipts).toBe(1)
    expect(result[0]!.pass_rate_pct).toBe(100)
    expect(result[0]!.avg_ocel_events).toBe(3)
  })

  it('single FAIL row has pass_rate_pct=0', () => {
    const [r] = aggregateByDay([makeRow('2026-06-19', 'FAIL', 'unknown', 0)])
    expect(r!.pass_rate_pct).toBe(0)
    expect(r!.fail_receipts).toBe(1)
    expect(r!.pass_receipts).toBe(0)
  })

  it('groups multiple rows by day correctly', () => {
    const rows = [
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-19', 'FAIL', 'browser', 2),
      makeRow('2026-06-18', 'PASS', 'real_ue4', 5),
    ]
    const result = aggregateByDay(rows)
    expect(result).toHaveLength(2)
    // Sorted descending by day
    expect(result[0]!.day).toBe('2026-06-19')
    expect(result[0]!.receipts).toBe(2)
    expect(result[0]!.pass_receipts).toBe(1)
    expect(result[0]!.fail_receipts).toBe(1)
    expect(result[1]!.day).toBe('2026-06-18')
    expect(result[1]!.real_ue4_receipts).toBe(1)
  })

  it('pass_rate_pct rounds correctly (2 pass, 1 fail → 67%)', () => {
    const rows = [
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-19', 'FAIL', 'browser', 3),
    ]
    const [r] = aggregateByDay(rows)
    expect(r!.pass_rate_pct).toBe(67)
  })

  it('real_ue4_receipts counts rocket_cli and real_ue4 but not browser', () => {
    const rows = [
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-19', 'PASS', 'real_ue4', 3),
      makeRow('2026-06-19', 'PASS', 'browser', 3),
    ]
    const [r] = aggregateByDay(rows)
    expect(r!.real_ue4_receipts).toBe(2)
  })

  it('topN=3 limits to 3 most recent days', () => {
    const rows = [
      makeRow('2026-06-15', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-16', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-17', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-18', 'PASS', 'rocket_cli', 3),
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 3),
    ]
    const result = aggregateByDay(rows, 3)
    expect(result).toHaveLength(3)
    expect(result[0]!.day).toBe('2026-06-19')
    expect(result[2]!.day).toBe('2026-06-17')
  })

  it('avg_ocel_events is mean across all rows in the day', () => {
    const rows = [
      makeRow('2026-06-19', 'PASS', 'rocket_cli', 4),
      makeRow('2026-06-19', 'FAIL', 'browser', 2),
    ]
    const [r] = aggregateByDay(rows)
    expect(r!.avg_ocel_events).toBeCloseTo(3.0)
  })

  it('sessions and unique_players are always null (not computed server-side)', () => {
    const [r] = aggregateByDay([makeRow('2026-06-19', 'PASS', 'rocket_cli', 3)])
    expect(r!.sessions).toBeNull()
    expect(r!.unique_players).toBeNull()
  })

  // Property: total receipts in a bucket = PASS + FAIL + PENDING
  it('∀ bucket: receipts = pass_receipts + fail_receipts + non-PASS-non-FAIL (property)', () => {
    const verdictArb = fc.constantFrom('PASS', 'FAIL', 'PENDING')
    const sourceArb = fc.constantFrom('rocket_cli', 'real_ue4', 'browser', 'unknown')

    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            verdict: verdictArb,
            engine_source: sourceArb,
            ocel_event_count: fc.integer({ min: 0, max: 10 }),
          }),
          { minLength: 1, maxLength: 20 },
        ),
        (rowsRaw) => {
          const rows = rowsRaw.map(r => ({ ...r, proven_at: '2026-06-19T00:00:00Z' }))
          const [bucket] = aggregateByDay(rows)
          if (!bucket) return
          expect(bucket.receipts).toBe(rows.length)
          expect(bucket.pass_receipts + bucket.fail_receipts).toBeLessThanOrEqual(bucket.receipts)
        },
      ),
    )
  })
})
