import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';

test.describe('TPS/DfLSS Playwright Manufacturing Strategy', () => {
  test('verify WASM world drives and generates cryptographic receipt', async ({ page }) => {
    // 1. Load the Factory Output
    const targetUrl = '/manufactured/Brm-HTML5-Shipping.html';

    const logs: string[] = [];
    page.on('console', (msg) => logs.push(`[${msg.type()}] ${msg.text()}`));

    await page.goto(targetUrl);

    // 2. Wait for Engine Initialization (Jidoka Check 1)
    await page.waitForFunction(() => (window as any).UE4_EngineReady === true, { timeout: 120000 });

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
    const inputTrace: string[] = [];
    await page.keyboard.down('Space');
    inputTrace.push('Space');
    await page.keyboard.down('W');
    inputTrace.push('W');
    await page.waitForTimeout(200); // Allow multiple animation frames to register movement
    await page.keyboard.up('Space');
    await page.keyboard.up('W');

    // 5. Final Verification (Jidoka Check 2)
    const afterBuffer = await page.screenshot();

    // Calculate Pixel Delta
    const img1 = PNG.sync.read(beforeBuffer);
    const img2 = PNG.sync.read(afterBuffer);
    const { width, height } = img1;
    const diff = new PNG({ width, height });

    const numDiffPixels = pixelmatch(img1.data, img2.data, diff.data, width, height, {
      threshold: 0.1,
    });

    const MINIMUM_MOTION_THRESHOLD = 50; // Arbitrary pixel threshold
    const verdict = numDiffPixels >= MINIMUM_MOTION_THRESHOLD ? 'PASS' : 'FAIL';

    // 6. Gather receipt data
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
        contractHash = crypto.createHash('sha256').update(specContent).digest('hex');
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

    const packagePath = path.resolve(__dirname, '..', 'manufactured', 'Brm-HTML5-Shipping.html');
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
    const receiptSignature = crypto.createHash('sha256').update(receiptString).digest('hex');

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

    if (verdict === 'FAIL') {
      throw new Error(
        `DefectError: World compiled, but physics/input verification failed. Zero/low visual delta. Diff pixels: ${numDiffPixels}`
      );
    }

    expect(numDiffPixels).toBeGreaterThan(MINIMUM_MOTION_THRESHOLD);
  });
});
