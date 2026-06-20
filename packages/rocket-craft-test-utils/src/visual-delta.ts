import { blake3 } from '@noble/hashes/blake3'
import type { VisualDeltaResult, RocketResidual } from './types.js'

export function hashBuffer(buf: Buffer | Uint8Array): string {
  return Buffer.from(blake3(buf)).toString('hex')
}

export function computePixelDelta(baseline: Buffer, after: Buffer): number {
  const len = Math.max(baseline.length, after.length)
  let diff = 0
  for (let i = 0; i < len; i++) {
    if ((baseline[i] ?? 0) !== (after[i] ?? 0)) diff++
  }
  return diff
}

export function buildVisualDeltaResult(
  baseline: Buffer,
  after: Buffer,
  opts: { min_changed_pixels?: number } = {}
): VisualDeltaResult {
  const residuals: RocketResidual[] = []
  const baseline_hash = hashBuffer(baseline)
  const after_hash = hashBuffer(after)
  const changed_pixels = computePixelDelta(baseline, after)
  const total = Math.max(baseline.length, after.length)
  const delta_ratio = total > 0 ? changed_pixels / total : 0
  const min = opts.min_changed_pixels ?? 1
  const admitted = changed_pixels >= min

  if (!admitted) {
    residuals.push({
      code: 'VISUAL-DELTA-ZERO',
      surface: 'canvas',
      message: `Canvas pixel delta is ${changed_pixels}, minimum required is ${min}`,
      severity: 'blocker',
      repair_candidate: 'Ensure a game-state-changing intent was admitted before capturing after-screenshot',
    })
  }

  return { baseline_hash, after_hash, changed_pixels, delta_ratio, admitted, residuals }
}
