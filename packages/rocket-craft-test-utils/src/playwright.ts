/**
 * Playwright helpers — require @playwright/test as a peer dep.
 * Import only from E2E test files, not unit or Nuxt runtime tests.
 */
import type { Page } from '@playwright/test'
import type { GameIntent, VisualDeltaResult } from './types.js'
import { buildVisualDeltaResult } from './visual-delta.js'

export async function waitForRocketShellReady(page: Page): Promise<void> {
  await page.waitForSelector('[data-testid="btn-start-walkthrough"]', { timeout: 15_000 })
}

export async function waitForUE4CanvasReady(page: Page): Promise<void> {
  // Waits for the canvas element the UE4 script targets
  await page.waitForSelector('canvas', { timeout: 60_000 })
  // Also wait for the rocket:ue4 engine-ready bridge event if it fires
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await page.evaluate((): Promise<void> => new Promise((resolve: any) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const w = globalThis as any
    if (w.__rocketEngineReady) { resolve(); return }
    const onReady = () => { resolve(); w.removeEventListener('rocket:ue4:ready', onReady) }
    w.addEventListener('rocket:ue4:ready', onReady)
    setTimeout(resolve, 30_000)
  }))
}

export async function emitGameIntent(page: Page, intent: Omit<GameIntent, 'seq'>): Promise<void> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await page.evaluate((i: any) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(globalThis as any).dispatchEvent(new CustomEvent('rocket:intent', {
      detail: { seq: Date.now(), intent: i, timestamp: new Date().toISOString() },
    }))
  }, intent)
}

export async function readBridgeEvents(page: Page): Promise<unknown[]> {
  return page.evaluate(() => {
    const events: unknown[] = []
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(globalThis as any).addEventListener('rocket:intent', (e: Event) => {
      events.push((e as CustomEvent).detail)
    })
    return events
  })
}

export async function captureCanvasScreenshot(page: Page, name: string): Promise<Buffer> {
  const canvas = page.locator('canvas').first()
  return canvas.screenshot({ path: `playwright-screenshots/${name}.png` })
}

export async function computeVisualDelta(baseline: Buffer, after: Buffer): Promise<VisualDeltaResult> {
  return buildVisualDeltaResult(baseline, after, { min_changed_pixels: 100 })
}

export async function assertCanvasDeltaAfterIntent(
  page: Page,
  intent: Omit<GameIntent, 'seq'>
): Promise<VisualDeltaResult> {
  const baseline = await captureCanvasScreenshot(page, `baseline-${intent.type}`)
  await emitGameIntent(page, intent)
  await page.waitForTimeout(500)
  const after = await captureCanvasScreenshot(page, `after-${intent.type}`)
  const result = await computeVisualDelta(baseline, after)
  if (!result.admitted) {
    throw new Error(
      `Canvas did not change after intent "${intent.type}": ${result.residuals.map(r => r.message).join('; ')}`
    )
  }
  return result
}

export async function recordPlaywrightEvidence(
  page: Page,
  result: VisualDeltaResult & { name?: string }
): Promise<void> {
  const name = result.name ?? 'evidence'
  await page.screenshot({ path: `playwright-screenshots/${name}-final.png` })
}
