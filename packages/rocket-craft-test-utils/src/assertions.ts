/**
 * Pure assertion helpers — throw Error on failure.
 * Test-runner-agnostic: use in Vitest, Playwright, or raw Node.
 */
import type {
  RocketReceipt, RocketResidual, GameIntent, VisualDeltaResult, AdmissionStatus
} from './types.js'
import { validateReceiptChain, detectMutation } from './receipts.js'
import { filterBlockers } from './residuals.js'

function fail(msg: string): never {
  throw new Error(`[rocket-craft-test-utils] ${msg}`)
}

// ── Receipt assertions ──────────────────────────────────────────────────────

export function expectReceiptChainValid(chain: RocketReceipt[]): void {
  const residuals = validateReceiptChain(chain)
  if (residuals.length > 0)
    fail(`Receipt chain invalid:\n${residuals.map(r => `  ${r.code}: ${r.message}`).join('\n')}`)
}

export function expectReceiptChainBroken(chain: RocketReceipt[]): void {
  const residuals = validateReceiptChain(chain)
  if (residuals.length === 0)
    fail('Expected receipt chain to be broken, but it was valid')
}

export function expectReceiptSequenceContiguous(chain: RocketReceipt[]): void {
  for (let i = 0; i < chain.length; i++) {
    if (chain[i].sequence !== i + 1)
      fail(`Sequence gap at index ${i}: expected ${i + 1}, got ${chain[i].sequence}`)
  }
}

export function expectPrevHashLinks(chain: RocketReceipt[]): void {
  for (let i = 1; i < chain.length; i++) {
    if (chain[i].prev_hash !== chain[i - 1].receipt)
      fail(`prev_hash mismatch at index ${i}`)
  }
}

export function expectMutationRefused(original: RocketReceipt, mutated: RocketReceipt): void {
  if (!detectMutation(original, mutated))
    fail('Expected mutation to be detected, but receipts appear identical')
}

export function expectReceiptStatus(receipt: RocketReceipt, status: AdmissionStatus): void {
  if (receipt.status !== status)
    fail(`Expected receipt status "${status}", got "${receipt.status}"`)
}

// ── Residual assertions ─────────────────────────────────────────────────────

export function expectNoBlockers(residuals: RocketResidual[]): void {
  const blockers = filterBlockers(residuals)
  if (blockers.length > 0)
    fail(`Expected no blockers, found ${blockers.length}:\n${blockers.map(r => `  ${r.code}: ${r.message}`).join('\n')}`)
}

export function expectResidual(residuals: RocketResidual[], code: string): void {
  if (!residuals.some(r => r.code === code))
    fail(`Expected residual with code "${code}", not found in [${residuals.map(r => r.code).join(', ')}]`)
}

export function expectRefusal(result: { status: AdmissionStatus }, reason?: string): void {
  if (result.status !== 'REFUSED')
    fail(`Expected status REFUSED, got "${result.status}"${reason ? ': ' + reason : ''}`)
}

export function expectAdmitted(result: { status?: AdmissionStatus }): void {
  if (result.status !== 'ADMITTED')
    fail(`Expected status ADMITTED, got "${result.status}"`)
}

export function expectResidualPublished(report: { residuals: RocketResidual[] }, code: string): void {
  expectResidual(report.residuals, code)
}

// ── GameIntent assertions ───────────────────────────────────────────────────

const KNOWN_RAW_BROWSER_EVENT_TYPES = new Set([
  'click', 'mousedown', 'mouseup', 'mousemove', 'keydown', 'keyup',
  'touchstart', 'touchend', 'pointerdown', 'pointerup',
])

export function expectGameIntentShape(intent: unknown): asserts intent is GameIntent {
  if (typeof intent !== 'object' || intent === null)
    fail('Intent must be an object')
  const i = intent as Record<string, unknown>
  if (typeof i['seq'] !== 'number') fail('Intent must have numeric seq')
  if (typeof i['type'] !== 'string') fail('Intent must have string type')
  if (typeof i['source'] !== 'string') fail('Intent must have string source')
}

export function expectIntentAdmitted(intent: GameIntent): void {
  if (intent.status !== 'ADMITTED')
    fail(`Expected intent "${intent.type}" to be ADMITTED, got "${intent.status}"`)
}

export function expectIntentRefused(intent: GameIntent): void {
  if (intent.status !== 'REFUSED')
    fail(`Expected intent "${intent.type}" to be REFUSED, got "${intent.status}"`)
}

export function expectIntentSequenceContiguous(intents: GameIntent[]): void {
  for (let i = 0; i < intents.length; i++) {
    if (intents[i].seq !== i + 1)
      fail(`Intent sequence gap at index ${i}: expected ${i + 1}, got ${intents[i].seq}`)
  }
}

export function expectNoRawBrowserEventSentToUE4(events: GameIntent[]): void {
  const raw = events.filter(e => KNOWN_RAW_BROWSER_EVENT_TYPES.has(e.type))
  if (raw.length > 0)
    fail(`Raw browser events must not reach UE4: ${raw.map(e => e.type).join(', ')}`)
}

// ── Visual delta assertions ─────────────────────────────────────────────────

export function expectVisualDelta(result: VisualDeltaResult): void {
  if (!result.admitted)
    fail(`Visual delta not admitted. Residuals: ${result.residuals.map(r => r.message).join('; ')}`)
  if ((result.changed_pixels ?? 0) === 0)
    fail('Expected non-zero pixel delta — canvas did not change')
}

export function expectVisualDeltaNotSpinnerOnly(result: VisualDeltaResult): void {
  if (!result.admitted)
    fail('Visual delta result was not admitted')
  // A spinner-only change typically touches < 1% of pixels
  const ratio = result.delta_ratio ?? 0
  if (ratio < 0.001)
    fail(`Delta ratio ${ratio} is suspiciously small — may be spinner-only change`)
}

// ── PWA assertions ──────────────────────────────────────────────────────────

export function expectManifestPresent(manifest: unknown): void {
  if (!manifest || typeof manifest !== 'object')
    fail('PWA manifest must be a non-null object')
  const m = manifest as Record<string, unknown>
  if (!m['name']) fail('PWA manifest missing "name"')
  if (!m['icons']) fail('PWA manifest missing "icons"')
}

export function expectNoPrivateDataPrecached(swManifest: string[]): void {
  const suspicious = swManifest.filter(url =>
    url.includes('token') || url.includes('secret') || url.includes('password') ||
    url.includes('auth') || url.includes('.env')
  )
  if (suspicious.length > 0)
    fail(`Service worker precache contains potentially private URLs: ${suspicious.join(', ')}`)
}

// ── Supabase assertions ─────────────────────────────────────────────────────

export function expectSupabaseHealthPass(result: { status: string; error?: string }): void {
  if (result.status !== 'ok')
    fail(`Supabase health check failed: ${result.error ?? result.status}`)
}
