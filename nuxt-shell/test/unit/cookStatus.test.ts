// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { inferCookStatus, buildCookSummary } from '../../server/utils/cookStatus'

describe('inferCookStatus', () => {
  it('empty log returns idle', () => {
    expect(inferCookStatus([])).toBe('idle')
  })

  it('detects failed from "cook failed" in tail', () => {
    const lines = [...Array(5).fill('Building...'), 'LogInit: cook failed: ShaderCompileError']
    expect(inferCookStatus(lines)).toBe('failed')
  })

  it('detects failed from "error:" in tail', () => {
    expect(inferCookStatus(['error: cannot open file abc.h'])).toBe('failed')
  })

  it('detects failed from "build failed" in tail', () => {
    expect(inferCookStatus(['Build Failed.'])).toBe('failed')
  })

  it('detects done from "package completed"', () => {
    expect(inferCookStatus(['Package Completed'])).toBe('done')
  })

  it('detects done from "cook completed"', () => {
    expect(inferCookStatus(['Cook Completed.'])).toBe('done')
  })

  it('detects done from "success" (matched by automation tool output)', () => {
    // UAT prints "AutomationTool exiting with ExitCode=0; Success!" — "success" keyword matches
    expect(inferCookStatus(['AutomationTool exiting with ExitCode=0; Success!'])).toBe('done')
  })

  it('detects cooking from "cooking" in tail', () => {
    expect(inferCookStatus(['[Cooking] Brm/Content/Materials/M_Rock.uasset'])).toBe('cooking')
  })

  it('detects cooking from "shadercompile"', () => {
    expect(inferCookStatus(['ShaderCompileWorker: Compiling 128 shaders'])).toBe('cooking')
  })

  it('detects cooking from "buildcookrun"', () => {
    expect(inferCookStatus(['BuildCookRun: Starting cook pass'])).toBe('cooking')
  })

  it('failed takes priority over done in same tail', () => {
    expect(inferCookStatus(['Package Completed', 'Cook Failed: out of memory'])).toBe('failed')
  })

  it('done takes priority over cooking in same tail (no errors)', () => {
    expect(inferCookStatus(['Cooking assets...', 'Package Completed'])).toBe('done')
  })

  it('only last 20 lines are inspected', () => {
    // Put "Cook Failed" in line 1 (outside window), "success" in last line
    const lines = ['Cook Failed: old error', ...Array(25).fill('...'), 'Package Completed']
    // The 'Cook Failed' is >20 lines back so should be ignored
    expect(inferCookStatus(lines)).toBe('done')
  })

  // Property: idle lines never produce a non-idle status
  it('∀ lines with no keywords → idle (property)', () => {
    fc.assert(
      fc.property(
        fc.array(fc.string().filter(s => !['cook', 'error', 'build', 'package', 'success', 'shader', 'cooking'].some(k => s.toLowerCase().includes(k))), { minLength: 1, maxLength: 15 }),
        (lines) => {
          const status = inferCookStatus(lines)
          expect(status).toBe('idle')
        },
      ),
    )
  })
})

describe('buildCookSummary', () => {
  const baseOpts = {
    logLines: [],
    logFile: null,
    project: 'Brm',
    lastReceipt: null,
    cookEvents: [],
    tailLines: 50,
  }

  it('no logs + no receipt → idle', () => {
    const s = buildCookSummary(baseOpts)
    expect(s.status).toBe('idle')
    expect(s.project).toBe('Brm')
    expect(s.verdict).toBeNull()
  })

  it('no logs + PASS receipt → done', () => {
    const s = buildCookSummary({
      ...baseOpts,
      lastReceipt: { verdict: 'PASS', proven_at: '2026-06-19T00:00:00Z', output_hash: 'abc', engine_source: 'rocket_cli' },
    })
    expect(s.status).toBe('done')
    expect(s.verdict).toBe('PASS')
  })

  it('no logs + FAIL receipt → idle (no log evidence)', () => {
    const s = buildCookSummary({
      ...baseOpts,
      lastReceipt: { verdict: 'FAIL', proven_at: '2026-06-19T00:00:00Z', output_hash: null, engine_source: 'rocket_cli' },
    })
    // No log lines → can't infer, fallback to idle (not FAIL)
    expect(s.status).toBe('idle')
  })

  it('with log lines, log status takes precedence over receipt', () => {
    const s = buildCookSummary({
      ...baseOpts,
      logLines: ['Cook Failed: disk full'],
      lastReceipt: { verdict: 'PASS', proven_at: '2026-06-19T00:00:00Z', output_hash: 'abc', engine_source: 'rocket_cli' },
    })
    expect(s.status).toBe('failed')
  })

  it('tailLines caps returned log_tail', () => {
    const s = buildCookSummary({
      ...baseOpts,
      logLines: Array.from({ length: 100 }, (_, i) => `line ${i}`),
      tailLines: 10,
    })
    expect(s.log_tail).toHaveLength(10)
    expect(s.log_tail[0]).toBe('line 90')
  })

  it('passes cook_events through', () => {
    const events = ['CookStarted', 'WasmPackaged', 'PackageVerified']
    const s = buildCookSummary({ ...baseOpts, cookEvents: events })
    expect(s.cook_events).toEqual(events)
  })

  it('passes log_file through', () => {
    const s = buildCookSummary({ ...baseOpts, logFile: '/tmp/ue4-cook-brm-123.log' })
    expect(s.log_file).toBe('/tmp/ue4-cook-brm-123.log')
  })

  it('output_hash from receipt is surfaced in summary', () => {
    const s = buildCookSummary({
      ...baseOpts,
      lastReceipt: { verdict: 'PASS', proven_at: '2026-06-19T00:00:00Z', output_hash: 'a'.repeat(64), engine_source: 'rocket_cli' },
    })
    expect(s.output_hash).toHaveLength(64)
  })
})
