/**
 * server/utils/cookStatus.ts
 *
 * Pure functions extracted from cook-status.get.ts.
 * Testable without Nitro, filesystem, or a running UE4 cook.
 */

export type CookStatusValue = 'idle' | 'cooking' | 'done' | 'failed'

/** UAT log patterns → cook status (ordered: failures checked first). */
const FAILED_PATTERNS = ['cook failed', 'error:', 'build failed', 'exception', 'fatal:']
const DONE_PATTERNS = ['package completed', 'cook completed', 'automation tool exiting with exitcode=0', 'success']
const COOKING_PATTERNS = ['cooking', 'shadercompile', 'buildcookrun', 'packaging', 'copying files']

export function inferCookStatus(logLines: string[]): CookStatusValue {
  if (!logLines.length) return 'idle'
  const tail = logLines.slice(-20).join('\n').toLowerCase()
  if (FAILED_PATTERNS.some(p => tail.includes(p))) return 'failed'
  if (DONE_PATTERNS.some(p => tail.includes(p))) return 'done'
  if (COOKING_PATTERNS.some(p => tail.includes(p))) return 'cooking'
  return 'idle'
}

export interface CookJobSummary {
  status: CookStatusValue
  project: string
  verdict: string | null
  proven_at: string | null
  output_hash: string | null
  engine_source: string | null
  log_tail: string[]
  log_file: string | null
  cook_events: string[]
}

/** Merge log-derived status with DB receipt data into a single summary. */
export function buildCookSummary(opts: {
  logLines: string[]
  logFile: string | null
  project: string
  lastReceipt: { verdict: string; proven_at: string; output_hash: string | null; engine_source: string } | null
  cookEvents: string[]
  tailLines: number
}): CookJobSummary {
  const { logLines, logFile, project, lastReceipt, cookEvents, tailLines } = opts
  const tail = logLines.slice(-tailLines)
  const status = tail.length
    ? inferCookStatus(tail)
    : lastReceipt?.verdict === 'PASS' ? 'done' : 'idle'

  return {
    status,
    project,
    verdict: lastReceipt?.verdict ?? null,
    proven_at: lastReceipt?.proven_at ?? null,
    output_hash: lastReceipt?.output_hash ?? null,
    engine_source: lastReceipt?.engine_source ?? null,
    log_tail: tail,
    log_file: logFile,
    cook_events: cookEvents,
  }
}
