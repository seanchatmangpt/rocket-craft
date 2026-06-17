import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';

test.describe('TPS/DfLSS Playwright Manufacturing Strategy', () => {
  test('verify WASM world drives and generates cryptographic receipt', async ({ page }) => {
    // 1. Load the Factory Output
    const targetUrl = '/Brm-HTML5-Shipping.html';

    const logs: string[] = [];
    page.on('console', (msg) => logs.push(`[${msg.type()}] ${msg.text()}`));

    await page.goto(targetUrl);

    // 2. Wait for Engine Initialization (Jidoka Check 1)
    await page.waitForFunction(() => (window as any).UE4_EngineReady === true, { timeout: 120000 });

    // Ensure rendering has started
    await page.waitForTimeout(500);

    // 3. Baseline Quality Check
    const beforeBuffer = await page.screenshot();

    // 4. Actuate (Drive the vehicle)
    await page.keyboard.press('Space');
    await page.waitForTimeout(100); // Allow physics to move the car up during jump

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

    if (numDiffPixels < MINIMUM_MOTION_THRESHOLD) {
      throw new Error(
        `DefectError: World compiled, but physics/input verification failed. Zero visual delta. Diff pixels: ${numDiffPixels}`
      );
    }

    // 6. Issue Receipt
    const receipt = {
      timestamp: new Date().toISOString(),
      prompt: 'tps-dflss-validation',
      contractHash: crypto.createHash('sha256').update('Rocket-Craft Contract').digest('hex'),
      packagePath: targetUrl,
      visualDelta: numDiffPixels,
      verdict: 'PASS',
      logs: logs,
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

    expect(numDiffPixels).toBeGreaterThan(MINIMUM_MOTION_THRESHOLD);
    console.log(`Receipt generated at ${receiptPath} with visual delta ${numDiffPixels}`);
  });
});
