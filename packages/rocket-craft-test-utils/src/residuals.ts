import type { RocketResidual } from './types.js'

export function createResidual(
  code: string,
  surface: string,
  message: string,
  severity: RocketResidual['severity'] = 'error',
  repair_candidate?: string
): RocketResidual {
  return { code, surface, message, severity, ...(repair_candidate ? { repair_candidate } : {}) }
}

export function publishResidual(arr: RocketResidual[], r: RocketResidual): RocketResidual[] {
  arr.push(r)
  return arr
}

export function filterBlockers(residuals: RocketResidual[]): RocketResidual[] {
  return residuals.filter(r => r.severity === 'blocker')
}

export function hasBlockers(residuals: RocketResidual[]): boolean {
  return filterBlockers(residuals).length > 0
}

export function residualFromError(err: unknown, surface: string, code = 'RUNTIME-ERROR'): RocketResidual {
  const message = err instanceof Error ? err.message : String(err)
  return { code, surface, message, severity: 'error' }
}

export function admitLater(code: string, surface: string, reason: string): RocketResidual {
  return {
    code: `ADMIT_LATER:${code}`,
    surface,
    message: reason,
    severity: 'info',
    repair_candidate: 'Schedule for next milestone',
  }
}
