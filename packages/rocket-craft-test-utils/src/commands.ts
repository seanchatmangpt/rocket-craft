import { spawnSync } from 'node:child_process'
import { blake3 } from '@noble/hashes/blake3'
import type { CommandReceipt, RocketResidual } from './types.js'

function hash(s: string): string {
  return Buffer.from(blake3(Buffer.from(s))).toString('hex')
}

function run(command: string, args: string[], cwd: string): CommandReceipt {
  const start = Date.now()
  let result
  try {
    result = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 10 * 1024 * 1024 })
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    return {
      command: [command, ...args].join(' '),
      cwd,
      exit_code: -1,
      stdout_hash: hash(''),
      stderr_hash: hash(msg),
      duration_ms: Date.now() - start,
      status: 'REFUSED',
      residuals: [{ code: 'COMMAND-SPAWN-FAILED', surface: 'commands', message: msg, severity: 'blocker' }],
    }
  }

  const exit_code = result.status ?? -1
  const stdout = result.stdout ?? ''
  const stderr = result.stderr ?? ''
  const residuals: RocketResidual[] = []

  if (exit_code !== 0) {
    residuals.push({
      code: 'COMMAND-NONZERO-EXIT',
      surface: 'commands',
      message: `${command} exited ${exit_code}: ${stderr.slice(0, 500)}`,
      severity: 'blocker',
      repair_candidate: `Check ${command} output and fix the underlying failure`,
    })
  }

  return {
    command: [command, ...args].join(' '),
    cwd,
    exit_code,
    stdout_hash: hash(stdout),
    stderr_hash: hash(stderr),
    duration_ms: Date.now() - start,
    status: exit_code === 0 ? 'ADMITTED' : 'REFUSED',
    residuals,
  }
}

const REPO = process.cwd()

export const runCargoTest = (cwd = REPO, extra: string[] = []): CommandReceipt =>
  run('cargo', ['test', '--all', ...extra], cwd)

export const runPnpmTest = (cwd = REPO, extra: string[] = []): CommandReceipt =>
  run('pnpm', ['test', ...extra], cwd)

export const runPlaywrightTest = (cwd = REPO, extra: string[] = []): CommandReceipt =>
  run('pnpm', ['exec', 'playwright', 'test', ...extra], cwd)

export const runSupabaseStart = (cwd = REPO): CommandReceipt =>
  run('supabase', ['start'], cwd)

export const runSupabaseDbReset = (cwd = REPO): CommandReceipt =>
  run('supabase', ['db', 'reset'], cwd)

export const runGgen = (cwd = REPO, extra: string[] = []): CommandReceipt =>
  run('ggen', ['sync', ...extra], cwd)

export const runWasm4pm = (cwd = REPO, extra: string[] = []): CommandReceipt =>
  run('wasm4pm', extra, cwd)

export const runRocketHtml5Pipeline = (project: string, cwd = REPO): CommandReceipt =>
  run('./rocket', ['html5', 'pipeline', '--project', project], cwd)
