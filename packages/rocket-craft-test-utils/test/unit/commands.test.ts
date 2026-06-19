import { describe, it, expect } from 'vitest'
import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import type { CommandReceipt } from '../../src/types.js'

// Helper: run a simple command directly to test the CommandReceipt shape
function runEcho(msg: string, cwd = process.cwd()): CommandReceipt {
  const start = Date.now()
  const result = spawnSync('echo', [msg], { cwd, encoding: 'utf8' })
  const hash = (s: string) => createHash('sha256').update(s).digest('hex')
  return {
    command: `echo ${msg}`,
    cwd,
    exit_code: result.status ?? 0,
    stdout_hash: hash(result.stdout ?? ''),
    stderr_hash: hash(result.stderr ?? ''),
    duration_ms: Date.now() - start,
    status: result.status === 0 ? 'ADMITTED' : 'REFUSED',
    residuals: [],
  }
}

function runFail(cwd = process.cwd()): CommandReceipt {
  const start = Date.now()
  const result = spawnSync('false', [], { cwd, encoding: 'utf8' })
  const hash = (s: string) => createHash('sha256').update(s).digest('hex')
  const exit_code = result.status ?? 1
  return {
    command: 'false',
    cwd,
    exit_code,
    stdout_hash: hash(''),
    stderr_hash: hash(''),
    duration_ms: Date.now() - start,
    status: exit_code === 0 ? 'ADMITTED' : 'REFUSED',
    residuals: [],
  }
}

describe('command receipts', () => {
  it('successful command returns ADMITTED status', () => {
    const receipt = runEcho('hello')
    expect(receipt.status).toBe('ADMITTED')
    expect(receipt.exit_code).toBe(0)
  })

  it('failed command returns REFUSED status', () => {
    const receipt = runFail()
    expect(receipt.status).toBe('REFUSED')
    expect(receipt.exit_code).not.toBe(0)
  })

  it('receipt has all required fields', () => {
    const receipt = runEcho('test')
    expect(typeof receipt.command).toBe('string')
    expect(typeof receipt.cwd).toBe('string')
    expect(typeof receipt.exit_code).toBe('number')
    expect(typeof receipt.stdout_hash).toBe('string')
    expect(receipt.stdout_hash).toHaveLength(64) // sha256 hex
    expect(typeof receipt.duration_ms).toBe('number')
    expect(receipt.duration_ms).toBeGreaterThanOrEqual(0)
    expect(Array.isArray(receipt.residuals)).toBe(true)
  })

  it('stdout_hash differs between different outputs', () => {
    const r1 = runEcho('hello')
    const r2 = runEcho('world')
    expect(r1.stdout_hash).not.toBe(r2.stdout_hash)
  })
})
