import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { createHash } from 'crypto';
import { PNG } from 'pngjs';
import { blake3 } from '@noble/hashes/blake3.js';

const sha256Hash = (buf: Buffer) => createHash('sha256').update(buf).digest('hex');
const blake3Hash = (buf: Buffer) => Buffer.from(blake3(buf)).toString('hex');

test.describe('Gundam Factory Walkthrough Projection E2E', () => {
  test('verify gundam factory walkthrough and generate receipt', async ({ page }) => {
    let currentCell = 'local serving cell';
    try {
      const targetUrl = process.env.TARGET_GAME_URL || '/Brm.html';
      const logs: string[] = [];
      
      page.on('console', msg => {
        console.log(`BROWSER LOG: ${msg.text()}`);
        logs.push(msg.text());
      });

      console.log(`Loading URL: ${targetUrl}`);
      const response = await page.goto(targetUrl);
      expect(response && response.ok()).toBe(true);

      currentCell = 'WebGL/runtime cell';
      // Wait for engine readiness signal
      await page.waitForFunction(
        () =>
          (window as any).Module?.calledMain === true ||
          ((document.querySelector('canvas') as HTMLCanvasElement | null)?.width ?? 0) > 0,
        { timeout: 120000 }
      );

      console.log('Engine readiness signal detected. Waiting 30s for map load/asset streaming...');
      await page.waitForTimeout(30000);

      // Click the canvas to focus
      try {
        const canvas = page.locator('canvas').first();
        await canvas.click({ timeout: 5000 });
        console.log('Canvas clicked successfully to focus');
      } catch (e) {
        console.warn('Canvas click failed, falling back to focus:', e);
        try {
          await page.focus('#canvas');
        } catch (e2) {
          console.warn('Canvas focus failed:', e2);
        }
      }

      await page.waitForTimeout(2000);

      // Open console and load the gameplay level
      console.log('Opening console via Backquote key...');
      await page.keyboard.press('Backquote');
      await page.waitForTimeout(1000);
      console.log('Typing command: open barbarian-1');
      await page.keyboard.type('open barbarian-1');
      await page.waitForTimeout(500);
      await page.keyboard.press('Enter');
      console.log('Waiting 15s for map transition...');
      await page.waitForTimeout(15000);

      // Capture baseline screenshot in the gameplay level
      const beforeBuffer1 = await page.screenshot();
      await page.waitForTimeout(200); // minimal wait to reduce idle background noise
      const beforeBuffer2 = await page.screenshot();

      const pixelmatchModule = await import('pixelmatch');
      const pixelmatch = pixelmatchModule.default;

      const imgB1 = PNG.sync.read(beforeBuffer1);
      const imgB2 = PNG.sync.read(beforeBuffer2);
      const { width, height } = imgB1;
      let diffIdle: PNG;
      let idleDeltaPixels = 0;

      if (imgB1.width !== imgB2.width || imgB1.height !== imgB2.height) {
        console.log(`Idle render shape changed from ${imgB1.width}x${imgB1.height} to ${imgB2.width}x${imgB2.height}.`);
        idleDeltaPixels = Math.max(imgB1.width * imgB1.height, imgB2.width * imgB2.height);
        diffIdle = imgB2;
      } else {
        diffIdle = new PNG({ width, height });
        idleDeltaPixels = pixelmatch(imgB1.data, imgB2.data, diffIdle.data, width, height, { threshold: 0.1 });
      }
      console.log(`Idle background animation delta: ${idleDeltaPixels}px`);

      // 4. Actuate: Inject movement input: press and hold the 'W' and 'Space' keys for 8 seconds, then release
      currentCell = 'input-binding cell';
      console.log('Injecting movement input: holding W and Space keys...');
      const inputTrace: string[] = [];
      
      await page.keyboard.down('W');
      inputTrace.push('W');
      await page.keyboard.down('Space');
      inputTrace.push('Space');
      
      await page.waitForTimeout(8000);
      
      await page.keyboard.up('W');
      await page.keyboard.up('Space');
      console.log('Released W and Space keys.');

      // 5. Final Verification
      currentCell = 'visual-delta cell';
      const afterBuffer = await page.screenshot();

      const imgAfter = PNG.sync.read(afterBuffer);
      let diffActuated: PNG;
      let numDiffPixels = 0;

      if (imgB2.width !== imgAfter.width || imgB2.height !== imgAfter.height) {
        console.log(`Actuated render shape changed from ${imgB2.width}x${imgB2.height} to ${imgAfter.width}x${imgAfter.height}.`);
        numDiffPixels = Math.max(imgB2.width * imgB2.height, imgAfter.width * imgAfter.height);
        diffActuated = imgAfter;
      } else {
        diffActuated = new PNG({ width, height });
        numDiffPixels = pixelmatch(imgB2.data, imgAfter.data, diffActuated.data, width, height, {
          threshold: 0.1,
        });
      }
      console.log(`Actuated visual delta: ${numDiffPixels}px`);

      // Count non-black pixels
      let nonBlackPixels = 0;
      for (let i = 0; i < imgB2.data.length; i += 4) {
        if ((imgB2.data[i] + imgB2.data[i+1] + imgB2.data[i+2]) > 10) {
          nonBlackPixels++;
        }
      }
      console.log(`Non-black rendered pixels: ${nonBlackPixels}`);

      // Has visual content: > 1000 non-black pixels
      // Has motion: > idleDeltaPixels + 50
      const threshold = idleDeltaPixels + 50;
      const hasVisualContent = nonBlackPixels > 1000;
      const hasMotion = numDiffPixels > threshold;
      const verdict = (hasMotion && hasVisualContent) ? 'PASS' : 'FAIL';
      console.log(`Visual proof: motion=${hasMotion} content=${hasVisualContent} verdict=${verdict}`);

      // Calculate BLAKE3/SHA256 hashes of the screenshots
      const beforeBlake3 = blake3Hash(beforeBuffer2);
      const beforeSha256 = sha256Hash(beforeBuffer2);
      const afterBlake3 = blake3Hash(afterBuffer);
      const afterSha256 = sha256Hash(afterBuffer);

      // Find Brm.wasm to compute its BLAKE3 hash
      let wasmPath = '';
      const wasmCandidates = [
        path.resolve(__dirname, '..', 'manufactured', 'Brm.wasm'),
        path.resolve('/tmp/brm-html5-archive/HTML5/Brm.wasm'),
        path.resolve(__dirname, '..', '..', 'versions', 'Brm427', 'Binaries', 'HTML5', 'Brm.wasm'),
      ];
      for (const candidate of wasmCandidates) {
        if (fs.existsSync(candidate)) {
          wasmPath = candidate;
          break;
        }
      }
      if (!wasmPath) {
        throw new Error('Could not find Brm.wasm to hash');
      }
      const wasmBytes = fs.readFileSync(wasmPath);
      const output_hash = 'blake3:' + blake3Hash(wasmBytes);

      // Write JSON receipt
      currentCell = 'receipt/audit cell';
      const receiptPayload = {
        timestamp: new Date().toISOString(),
        run_id: `gundam-factory-${Date.now()}`,
        output_hash,
        screenshots: {
          before_base64: beforeBuffer2.toString('base64'),
          before_blake3: beforeBlake3,
          before_sha256: beforeSha256,
          after_base64: afterBuffer.toString('base64'),
          after_blake3: afterBlake3,
          after_sha256: afterSha256,
        },
        consoleLogs: logs,
        inputTrace,
        visualDelta: numDiffPixels,
        verdict,
      };

      const receiptString = JSON.stringify(receiptPayload, null, 2);
      const signature = sha256Hash(Buffer.from(receiptString));

      const finalReceipt = {
        ...receiptPayload,
        signature,
      };

      const resultsDir = path.join(__dirname, '..', 'test-results');
      if (!fs.existsSync(resultsDir)) {
        fs.mkdirSync(resultsDir, { recursive: true });
      }

      const receiptPath = path.join(resultsDir, 'gundam-factory-playwright-receipt.json');
      fs.writeFileSync(receiptPath, JSON.stringify(finalReceipt, null, 2));
      console.log(`Receipt successfully signed and written to ${receiptPath}`);

      const diffPath = path.join(resultsDir, 'gundam-factory-diff.png');
      fs.writeFileSync(diffPath, PNG.sync.write(diffActuated));

      expect(hasVisualContent).toBe(true);
      expect(numDiffPixels).toBeGreaterThan(threshold);
      expect(verdict).toBe('PASS');
    } catch (error) {
      console.error(`[ANDON PULL] DEFECT DETECTED IN CELL: ${currentCell}`);
      throw error;
    }
  });
});
