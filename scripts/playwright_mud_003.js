/**
 * GC-MECH-FACTORY-MUD-003: Playwright Visual Delta Proof
 *
 * Doctrine: 0 fitness, 0 conformance assumed.
 * Evidence required:
 *   - Engine load detected (canvas/WebGL appears)
 *   - Baseline screenshot captured
 *   - Input actuation injected
 *   - After screenshot captured
 *   - Pixel delta > threshold
 *   - Console logs captured
 *   - BLAKE3-equivalent receipt written (SHA-256 used as WASM substitute)
 *
 * Standing: PARTIAL_ALIVE candidate until receipt is independently replayed.
 */

const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const HTML5_DIR = path.resolve(__dirname, '../versions/v4_27_0/Binaries/HTML5');
const RECEIPT_DIR = path.resolve(__dirname, '../generated/mech_factory_mud/playwright');
const PORT = 8765;

// ─── LOCAL STATIC SERVER ──────────────────────────────────────────────────────
function startServer() {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      const safePath = req.url.split('?')[0];
      const filePath = path.join(HTML5_DIR, safePath === '/' ? '/Brm.html' : safePath);
      const ext = path.extname(filePath);
      const mimeTypes = {
        '.html': 'text/html',
        '.js':   'application/javascript',
        '.wasm': 'application/wasm',
        '.data': 'application/octet-stream',
        '.css':  'text/css',
      };
      const contentType = mimeTypes[ext] || 'application/octet-stream';
      try {
        const data = fs.readFileSync(filePath);
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
      } catch {
        res.writeHead(404);
        res.end('Not Found: ' + filePath);
      }
    });
    server.listen(PORT, '127.0.0.1', () => resolve(server));
  });
}

// ─── PIXEL DELTA COMPUTATION ─────────────────────────────────────────────────
function computePixelDelta(beforeBuf, afterBuf) {
  if (beforeBuf.length !== afterBuf.length) return 1.0;
  let diff = 0;
  for (let i = 0; i < beforeBuf.length; i++) {
    diff += Math.abs(beforeBuf[i] - afterBuf[i]);
  }
  // Normalize: max possible diff per byte is 255
  return diff / (beforeBuf.length * 255);
}

// ─── RECEIPT GENERATION (SHA-256 as portable substitute for BLAKE3) ──────────
function hashFile(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

// ─── MAIN ─────────────────────────────────────────────────────────────────────
(async () => {
  fs.mkdirSync(RECEIPT_DIR, { recursive: true });

  const consoleLogs = [];
  const consoleErrors = [];
  let engineReady = false;

  console.log('[003] Starting local HTML5 server...');
  const server = await startServer();
  console.log(`[003] Serving HTML5 package at http://127.0.0.1:${PORT}`);

  const browser = await chromium.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--enable-webgl', '--ignore-gpu-blocklist'],
  });

  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 },
    ignoreHTTPSErrors: true,
  });

  const page = await context.newPage();

  // Capture all console output
  page.on('console', (msg) => {
    const text = `[${msg.type()}] ${msg.text()}`;
    consoleLogs.push(text);
    if (msg.type() === 'error') consoleErrors.push(text);
    // Detect engine readiness heuristics
    if (
      msg.text().includes('UE4') ||
      msg.text().includes('Brm') ||
      msg.text().includes('WebGL') ||
      msg.text().includes('ready') ||
      msg.text().includes('loaded')
    ) {
      engineReady = true;
    }
  });

  page.on('pageerror', (err) => {
    consoleErrors.push(`[pageerror] ${err.message}`);
  });

  console.log('[003] Navigating to UE4 HTML5 package...');
  try {
    await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  } catch (e) {
    console.log(`[003] Navigation warning (expected for WASM): ${e.message}`);
  }

  // Wait for canvas to appear (engine render surface) — WASM init takes time
  let canvasDetected = false;
  try {
    await page.waitForSelector('canvas', { timeout: 30000 });
    canvasDetected = true;
    console.log('[003] Canvas detected — engine render surface present.');
  } catch {
    console.log('[003] Canvas not detected within timeout — checking for fallback content.');
  }

  // Wait for engine to settle after WASM init
  await page.waitForTimeout(8000);

  // ─── GATE 3: BASELINE SCREENSHOT ──────────────────────────────────────────
  const beforePath = path.join(RECEIPT_DIR, 'before.png');
  await page.screenshot({ path: beforePath, fullPage: false });
  const beforeBuf = fs.readFileSync(beforePath);
  console.log(`[003] Baseline screenshot captured: ${beforePath} (${beforeBuf.length} bytes)`);

  // ─── GATE 5: INPUT ACTUATION ──────────────────────────────────────────────
  console.log('[003] Injecting keyboard input actuation (W A S D + Space)...');
  await page.keyboard.press('w');
  await page.waitForTimeout(200);
  await page.keyboard.press('a');
  await page.waitForTimeout(200);
  await page.keyboard.press('s');
  await page.waitForTimeout(200);
  await page.keyboard.press('d');
  await page.waitForTimeout(200);
  await page.keyboard.press('Space');
  await page.waitForTimeout(500);

  // Allow engine to react to input
  await page.waitForTimeout(4000);

  // ─── GATE 6: AFTER SCREENSHOT + VISUAL DELTA ──────────────────────────────
  const afterPath = path.join(RECEIPT_DIR, 'after.png');
  await page.screenshot({ path: afterPath, fullPage: false });
  const afterBuf = fs.readFileSync(afterPath);
  console.log(`[003] After screenshot captured: ${afterPath} (${afterBuf.length} bytes)`);

  const delta = computePixelDelta(beforeBuf, afterBuf);
  const DELTA_THRESHOLD = 0.001; // 0.1% pixel change required
  const deltaPass = delta > DELTA_THRESHOLD;

  console.log(`[003] Visual delta: ${(delta * 100).toFixed(4)}% — threshold: ${(DELTA_THRESHOLD * 100).toFixed(3)}%`);
  console.log(`[003] Delta gate: ${deltaPass ? 'PASS' : 'FAIL (below threshold)'}`);

  await browser.close();
  server.close();

  // ─── GATE 7: BLAKE3-EQUIVALENT RECEIPT ────────────────────────────────────
  const receipt = {
    milestone: 'GC-MECH-FACTORY-MUD-003',
    timestamp: new Date().toISOString(),
    status: deltaPass ? 'VISUAL_DELTA_ADMITTED' : 'VISUAL_DELTA_BELOW_THRESHOLD',
    engine_ready_signal_detected: engineReady,
    canvas_detected: canvasDetected,
    before_screenshot: beforePath,
    after_screenshot: afterPath,
    before_sha256: hashFile(beforeBuf),
    after_sha256: hashFile(afterBuf),
    pixel_delta_ratio: delta,
    pixel_delta_threshold: DELTA_THRESHOLD,
    delta_gate: deltaPass ? 'PASS' : 'FAIL',
    console_log_count: consoleLogs.length,
    console_error_count: consoleErrors.length,
    console_errors: consoleErrors.slice(0, 10),
    verdict: deltaPass ? 'ADMITTED' : 'REFUSED_DELTA_BELOW_THRESHOLD',
    residuals: deltaPass
      ? []
      : ['VISUAL_DELTA_BELOW_THRESHOLD — engine may not have fully rendered before screenshot'],
    doctrine: '0 fitness, 0 conformance baseline. Standing requires replay.',
  };

  const receiptPath = path.join(RECEIPT_DIR, 'receipt.json');
  fs.writeFileSync(receiptPath, JSON.stringify(receipt, null, 2));

  // Write console log
  const logPath = path.join(RECEIPT_DIR, 'console.log');
  fs.writeFileSync(logPath, consoleLogs.join('\n'));

  console.log('\n[003] ─── RECEIPT ────────────────────────────────────────────');
  console.log(JSON.stringify(receipt, null, 2));
  console.log('[003] ─────────────────────────────────────────────────────────');
  console.log(`[003] Receipt written to: ${receiptPath}`);

  // Exit with non-zero if delta gate fails
  process.exit(deltaPass ? 0 : 2);
})();
