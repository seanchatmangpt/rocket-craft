/**
 * POST /api/game/cook-trigger
 *
 * Triggers the rocket HTML5 cook pipeline via the CLI and returns a job_id
 * for polling via GET /api/game/cook-status.
 *
 * This endpoint makes the entire cook→verify→receipt loop triggerable via HTTP
 * with no SSH or human interaction — the final link for CI headless E2E.
 *
 * Pattern: nuxt-supabase-book chapter-11 task-runner-engine / Go TaskRunner.
 *
 * Body: { project?: string, target?: string }
 * Returns: { job_id, status: 'queued', project, triggered_at }
 *
 * The job runs async via child_process.spawn. Poll cook-status for progress.
 * Requires: ROCKET_CLI_PATH env var (default: ./rocket in the repo root)
 *           ROCKET_CRAFT_ROOT env var (default: process.cwd())
 *           ALLOW_COOK_TRIGGER=1 (required for non-development environments)
 */

import { spawn } from 'node:child_process'
import { join } from 'node:path'
import { randomUUID } from 'node:crypto'

interface CookJob {
  job_id: string
  project: string
  target: string
  status: 'queued' | 'running' | 'done' | 'failed'
  triggered_at: string
  pid?: number
  exit_code?: number
}

// In-process job registry (resets on server restart — use Supabase for persistence)
const activeJobs = new Map<string, CookJob>()

export default defineEventHandler(async (event) => {
  // Guard: only allowed in dev or when explicitly enabled
  const isDev = process.env.NODE_ENV === 'development'
  const allowed = isDev || process.env.ALLOW_COOK_TRIGGER === '1'
  if (!allowed) {
    throw createError({
      statusCode: 403,
      message: 'Cook trigger not allowed. Set ALLOW_COOK_TRIGGER=1 to enable.',
    })
  }

  const body = await readBody(event)
  const project = (body?.project as string) ?? 'Brm'
  const target = (body?.target as string) ?? project

  // Guard: one active cook per project at a time
  for (const job of activeJobs.values()) {
    if (job.project === project && job.status === 'running') {
      throw createError({
        statusCode: 409,
        message: `Cook already running for project ${project} (job_id=${job.job_id})`,
      })
    }
  }

  const job_id = randomUUID()
  const triggered_at = new Date().toISOString()

  const job: CookJob = {
    job_id,
    project,
    target,
    status: 'queued',
    triggered_at,
  }
  activeJobs.set(job_id, job)

  // Spawn cook pipeline async (do not await)
  const repoRoot = process.env.ROCKET_CRAFT_ROOT ?? process.cwd()
  const rocketCli = process.env.ROCKET_CLI_PATH ?? join(repoRoot, 'rocket')

  const cookProcess = spawn(
    rocketCli,
    ['html5', 'pipeline', '--project', project],
    {
      cwd: repoRoot,
      env: { ...process.env },
      stdio: 'pipe',
      detached: false,
    },
  )

  job.pid = cookProcess.pid
  job.status = 'running'

  // Collect stdout/stderr to COOK_LOG_DIR
  const logDir = process.env.COOK_LOG_DIR ?? '/tmp'
  const logPath = join(logDir, `ue4-cook-${project.toLowerCase()}-${Date.now()}.log`)
  const { createWriteStream } = await import('node:fs')
  const logStream = createWriteStream(logPath, { flags: 'a' })

  cookProcess.stdout?.pipe(logStream)
  cookProcess.stderr?.pipe(logStream)

  cookProcess.on('close', (code) => {
    job.status = code === 0 ? 'done' : 'failed'
    job.exit_code = code ?? -1
    logStream.end()
    // Evict old jobs (keep last 20)
    const keys = Array.from(activeJobs.keys())
    if (keys.length > 20) {
      activeJobs.delete(keys[0]!)
    }
  })

  return {
    job_id,
    status: 'queued' as const,
    project,
    target,
    pid: cookProcess.pid,
    log_path: logPath,
    triggered_at,
    poll: `/api/game/cook-status?project=${project}`,
  }
})
