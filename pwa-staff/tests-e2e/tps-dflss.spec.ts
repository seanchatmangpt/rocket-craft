import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { hashJsonString } from '@wasm4pm/contracts';
import { PNG } from 'pngjs';

test.describe('TPS/DfLSS Playwright Manufacturing Strategy', () => {
  test('verify WASM world drives and generates cryptographic receipt', async ({ page }) => {
    let currentCell = 'local serving cell';
    try {
      // 1. Load the Factory Output (Dynamically loaded from ENV)
      const targetUrl = process.env.TARGET_GAME_URL || '/manufactured/Brm-HTML5-Shipping.html';

      const logs: string[] = [];
      page.on('console', msg => {
        console.log(`BROWSER LOG: ${msg.text()}`);
        logs.push(msg.text());
      });

      const response = await page.goto(targetUrl);
      expect(response && response.ok()).toBe(true);

      // 2. Wait for Engine Initialization (Jidoka Check 1)
      // Poll for multiple readiness signals because real UE4 HTML5 packages built by
      // SpeculativeCoder's 4.27-html5-es3 fork do NOT set window.UE4_EngineReady.
      // Real Emscripten output signals readiness via window.Module.calledMain (Emscripten
      // lifecycle flag) or canvas dimensions becoming non-zero (first rendered pixels).
      // window.UE4_EngineReady is retained for stub/mock compatibility in CI environments.
      currentCell = 'WebGL/runtime cell';
      await page.waitForFunction(
        () =>
          (window as any).UE4_EngineReady === true ||
          (window as any).Module?.calledMain === true ||
          ((document.querySelector('canvas') as HTMLCanvasElement | null)?.width ?? 0) > 0,
        { timeout: 120000 }
      );

      // Ensure rendering has started
      await page.waitForTimeout(500);

      // Focus the canvas to accept keyboard input
      try {
        await page.focus('#canvas');
      } catch (e) {
        console.warn('Canvas element could not be focused', e);
      }

      // 3. Baseline Quality Check
      const beforeBuffer = await page.screenshot();

      // 4. Actuate (Drive the vehicle)
      currentCell = 'input-binding cell';
      const inputTrace: string[] = [];
      await page.keyboard.down('Space');
      inputTrace.push('Space');
      await page.keyboard.down('W');
      inputTrace.push('W');
      await page.waitForTimeout(1000); // 1s — enough for physics tick + visual change in UE4
      await page.keyboard.up('Space');
      await page.keyboard.up('W');

      // 5. Final Verification (Jidoka Check 2)
      currentCell = 'visual-delta cell';
      const afterBuffer = await page.screenshot();

      // Calculate Pixel Delta
      const img1 = PNG.sync.read(beforeBuffer);
      const img2 = PNG.sync.read(afterBuffer);
      const { width, height } = img1;
      const diff = new PNG({ width, height });

      const pixelmatchModule = await import('pixelmatch');
      const pixelmatch = pixelmatchModule.default;

      const numDiffPixels = pixelmatch(img1.data, img2.data, diff.data, width, height, {
        threshold: 0.1,
      });

      const MINIMUM_MOTION_THRESHOLD = 100; // Require real motion — loading screen animations still pass
      const verdict = numDiffPixels >= MINIMUM_MOTION_THRESHOLD ? 'PASS' : 'FAIL';

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
          const specContent = fs.readFileSync(specPath);
          contractHash = hashJsonString(specContent.toString('utf-8'));
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

      // 7. Issue Receipt
      const receipt = {
        timestamp: new Date().toISOString(),
        prompt,
        contractHash,
        buildLog,
        packagePath,
        browserUrl,
        screenshots,
        consoleLogs: logs,
        inputTrace,
        visualDelta: numDiffPixels,
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

      console.log(`Receipt generated at ${receiptPath} with visual delta ${numDiffPixels} and verdict ${verdict}`);

      currentCell = 'visual-delta cell';
      if (verdict === 'FAIL') {
        throw new Error(
          `DefectError: World compiled, but physics/input verification failed. Zero/low visual delta. Diff pixels: ${numDiffPixels}`
        );
      }

      expect(numDiffPixels).toBeGreaterThan(MINIMUM_MOTION_THRESHOLD);
    } catch (error) {
      console.error(`[ANDON PULL] DEFECT DETECTED IN CELL: ${currentCell}`);
      throw error;
    }
  });
});
