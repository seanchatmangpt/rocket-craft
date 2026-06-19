import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { blake3 } from '@noble/hashes/blake3';
const blake3Hex = (s: string | Buffer): string => Buffer.from(blake3(typeof s === 'string' ? Buffer.from(s) : s)).toString('hex');
const hashJsonString = blake3Hex;
import { PNG } from 'pngjs';
import { createClient } from '@supabase/supabase-js';

const SUPABASE_URL = process.env.SUPABASE_URL || 'http://localhost:54321';
const SUPABASE_ANON_KEY = process.env.SUPABASE_ANON_KEY || 'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sb = createClient<any>(SUPABASE_URL, SUPABASE_ANON_KEY);

/** Poll Supabase for a row matching the receipt_hash (Gap 3 fix). */
async function pollForReceiptPersistence(receiptHash: string, timeoutMs = 15_000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { data } = await (sb as any)
      .from('game_receipts')
      .select('id, verdict')
      .eq('receipt_hash', receiptHash)
      .limit(1);
    if (Array.isArray(data) && data.length > 0) return true;
    await new Promise(r => setTimeout(r, 1_000));
  }
  return false;
}

test.describe('TPS/DfLSS Playwright Manufacturing Strategy', () => {
  test('verify WASM world drives and generates cryptographic receipt', async ({ page }) => {
    let currentCell = 'local serving cell';
    try {
      // 1. Load the Factory Output (Dynamically loaded from ENV)
      const targetUrl = process.env.TARGET_GAME_URL || '/Brm.html';

      const logs: string[] = [];
      page.on('console', msg => {
        console.log(`BROWSER LOG: ${msg.text()}`);
        logs.push(msg.text());
      });

      // NOTE: Brm.UE4.js's sessionStorage override is disabled at the source level
      // (if ('') guard is always false). The game starts in its default map.
      // Visual delta proof is based on menu/title screen animation (>20 px change).
      const response = await page.goto(targetUrl);
      expect(response && response.ok()).toBe(true);

      // 2. Wait for Engine Initialization (Jidoka Check 1)
      // Poll for multiple readiness signals because real UE4 HTML5 packages built by
      // SpeculativeCoder's 4.27-html5-es3 fork do NOT set window.UE4_EngineReady.
      // Real Emscripten output signals readiness via window.Module.calledMain (Emscripten
      // lifecycle flag) or canvas dimensions becoming non-zero (first rendered pixels).
      // window.UE4_EngineReady is retained for stub/mock compatibility in CI environments.
      currentCell = 'WebGL/runtime cell';
      // Wait for WASM main() to start — this fires early in startup
      await page.waitForFunction(
        () =>
          (window as any).Module?.calledMain === true ||
          ((document.querySelector('canvas') as HTMLCanvasElement | null)?.width ?? 0) > 0,
        { timeout: 120000 }
      );

      // UE4 HTML5 needs significant time after calledMain before it renders:
      // asset streaming, map load, first frame. 30s covers typical startup.
      await page.waitForTimeout(30000);

      // Click the canvas to give it focus and drive pointer events (UE4 HTML5 requires a
      // pointer interaction before it will accept keyboard events on most browsers).
      try {
        const canvas = page.locator('canvas').first();
        await canvas.click({ timeout: 5000 });
        console.log('Canvas clicked successfully');
      } catch (e) {
        console.warn('Canvas click failed, falling back to focus:', e);
        try {
          await page.focus('#canvas');
        } catch (e2) {
          console.warn('Canvas focus also failed:', e2);
        }
      }

      // 3. Dynamic Baseline Quality Check — wait extra after click for first render frame
      await page.waitForTimeout(5000);
      const beforeBuffer1 = await page.screenshot();
      await page.waitForTimeout(5000); // Wait to capture idle animation noise
      const beforeBuffer2 = await page.screenshot();

      const pixelmatchModule = await import('pixelmatch');
      const pixelmatch = pixelmatchModule.default;

      const imgB1 = PNG.sync.read(beforeBuffer1);
      const imgB2 = PNG.sync.read(beforeBuffer2);
      const { width, height } = imgB1;
      const diffIdle = new PNG({ width, height });

      const idleDeltaPixels = pixelmatch(imgB1.data, imgB2.data, diffIdle.data, width, height, { threshold: 0.1 });
      console.log(`Idle background animation delta: ${idleDeltaPixels}px`);

      // 4. Actuate (Drive the vehicle or interact with the game's first UI)
      currentCell = 'input-binding cell';
      const inputTrace: string[] = [];
      await page.keyboard.down('Space');
      inputTrace.push('Space');
      await page.keyboard.down('W');
      inputTrace.push('W');
      await page.waitForTimeout(8000); // 8s — UE4 HTML5 physics tick rate is slower; allow motion to accumulate
      await page.keyboard.up('Space');
      await page.keyboard.up('W');

      // 5. Final Verification (Jidoka Check 2)
      currentCell = 'visual-delta cell';
      const afterBuffer = await page.screenshot();

      // Calculate Pixel Delta against the second baseline
      const imgAfter = PNG.sync.read(afterBuffer);
      const diffActuated = new PNG({ width, height });

      const numDiffPixels = pixelmatch(imgB2.data, imgAfter.data, diffActuated.data, width, height, {
        threshold: 0.1,
      });
      console.log(`Actuated visual delta: ${numDiffPixels}px`);

      // Primary proof: the rendered frame must not be blank (all-black canvas).
      // A real UE4 WebGL2 render will have non-zero pixel values in the screenshot.
      // Count non-black pixels (R+G+B > 10 to avoid compression noise).
      let nonBlackPixels = 0;
      for (let i = 0; i < imgB2.data.length; i += 4) {
        if ((imgB2.data[i] + imgB2.data[i+1] + imgB2.data[i+2]) > 10) nonBlackPixels++;
      }
      console.log(`Non-black rendered pixels: ${nonBlackPixels}`);

      // The game renders a real UI (login form, title screen, etc.) — even a static frame
      // is proof of a live WebGL2 render if it has >1000 non-black pixels.
      // Dynamic motion proof (idleDeltaPixels + 50) is preferred but not required
      // when the game first-loads to a static screen.
      const MINIMUM_MOTION_THRESHOLD = idleDeltaPixels + 50;
      const hasVisualContent = nonBlackPixels > 1000;
      const hasMotion = numDiffPixels > MINIMUM_MOTION_THRESHOLD;
      const verdict = (hasMotion && hasVisualContent) ? 'PASS' : 'FAIL';
      const visualDelta = numDiffPixels;
      console.log(`Visual proof: motion=${hasMotion} content=${hasVisualContent} verdict=${verdict}`);

      // Reassign beforeBuffer and diff for receipt compatibility
      const beforeBuffer = beforeBuffer2;
      const diff = diffActuated;
      // 6. Gather receipt data
      currentCell = 'receipt/audit cell';
      let prompt = 'tps-dflss-validation';
      try {
        const specPath = '/Users/sac/rocket-craft/spec.json';
        if (fs.existsSync(specPath)) {
          const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));
          if (spec.history && spec.history.length > 0) {
            for (let i = spec.history.length - 1; i >= 0; i--) {
              if (spec.history[i].details && spec.history[i].details.modification_intent) {
                prompt = spec.history[i].details.modification_intent;
                break;
              }
            }
          }
        }
      } catch (e) {
        console.error('Failed to parse spec.json for prompt:', e);
      }

      let contractHash = '';
      try {
        const specPath = '/Users/sac/rocket-craft/spec.json';
        if (fs.existsSync(specPath)) {
          contractHash = hashJsonString(fs.readFileSync(specPath, 'utf-8'));
        }
      } catch (e) {
        console.error('Failed to hash spec.json:', e);
      }

      let buildLog = '';
      try {
        const logPath = '/Users/sac/rocket-craft/deploy.log';
        if (fs.existsSync(logPath)) {
          buildLog = fs.readFileSync(logPath, 'utf8');
        }
      } catch (e) {
        console.error('Failed to read deploy.log:', e);
      }

      const gameFileName = path.basename(targetUrl);
      const packagePath = path.resolve(__dirname, '..', 'manufactured', gameFileName);
      const browserUrl = page.url();

      const screenshots = {
        before: beforeBuffer.toString('base64'),
        after: afterBuffer.toString('base64'),
      };

      // Compute output_hash (BLAKE3 hex of the .wasm artifact) for receipt validation
      let output_hash = '';
      const wasmCandidates = [
        path.resolve(__dirname, '..', '..', 'versions', 'Brm427', 'Binaries', 'HTML5', 'Brm.wasm'),
        path.resolve(__dirname, '..', '..', 'versions', '4.27.0', 'Binaries', 'HTML5', 'Brm.wasm'),
        path.resolve('/tmp/brm-html5-archive/HTML5/Brm.wasm'),
      ];
      for (const candidate of wasmCandidates) {
        if (fs.existsSync(candidate)) {
          const wasmBytes = fs.readFileSync(candidate);
          output_hash = 'blake3:' + blake3Hex(wasmBytes);
          break;
        }
      }
      const run_id = `tps-dflss-${Date.now()}`;

      // 7. Issue Receipt
      const receipt = {
        timestamp: new Date().toISOString(),
        run_id,
        output_hash,
        prompt,
        contractHash,
        buildLog,
        packagePath,
        browserUrl,
        screenshots,
        consoleLogs: logs,
        inputTrace,
        visualDelta: visualDelta,
        verdict,
      };

      const receiptString = JSON.stringify(receipt, null, 2);
      const receiptSignature = hashJsonString(receiptString);

      const finalReceipt = {
        ...receipt,
        signature: receiptSignature,
      };

      const resultsDir = path.join(__dirname, '..', 'test-results');
      if (!fs.existsSync(resultsDir)) {
        fs.mkdirSync(resultsDir, { recursive: true });
      }

      const receiptPath = path.join(resultsDir, 'tps-dflss-receipt.json');
      fs.writeFileSync(receiptPath, JSON.stringify(finalReceipt, null, 2));

      const diffPath = path.join(resultsDir, 'tps-dflss-diff.png');
      fs.writeFileSync(diffPath, PNG.sync.write(diff));

      console.log(`Receipt generated at ${receiptPath} with visual delta ${visualDelta} and verdict ${verdict}`);

      // Gap 3 fix: verify receipt was persisted to Supabase (not just written to local JSON).
      // The Nuxt game shell's commitReceipt() calls /api/game/receipt which writes to game_receipts.
      // We poll for the row by receipt_hash so Playwright asserts DB persistence, not just local state.
      if (verdict === 'PASS') {
        currentCell = 'supabase-receipt-persistence cell';
        const receiptHashForLookup = receiptSignature;
        const persisted = await pollForReceiptPersistence(receiptHashForLookup, 15_000);
        if (!persisted) {
          // Hard assertion: game.vue now auto-commits when OCEL lifecycle reaches
          // PROVEN_LIFECYCLE = [GameSessionStarted, FrameRendered, InputAdmitted].
          // If Supabase doesn't have the receipt within 15s, the auto-commit failed.
          console.error(
            `[Gap 3] Receipt not found in Supabase after 15s — ` +
            `hash=${receiptHashForLookup.slice(0, 16)}… ` +
            `Check that game.vue auto-commit watcher fired.`
          );
          expect(persisted, 'Receipt must be persisted to Supabase within 15s of PASS verdict').toBe(true);
        } else {
          console.log(`[Gap 3] Receipt persisted to Supabase ✓ (hash=${receiptHashForLookup.slice(0, 16)}…)`);
        }
      }

      currentCell = 'visual-delta cell';
      if (verdict === 'FAIL') {
        throw new Error(
          `DefectError: World compiled but visual proof failed. Non-black pixels: ${nonBlackPixels}, motion delta: ${numDiffPixels}`
        );
      }

      expect(verdict).toBe('PASS');
    } catch (error) {
      console.error(`[ANDON PULL] DEFECT DETECTED IN CELL: ${currentCell}`);
      throw error;
    }
  });
});
