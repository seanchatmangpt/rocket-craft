/**
 * GET /api/game/cook-log
 *
 * Server-Sent Events (SSE) endpoint that tails ~/ue4-cook-latest.log and
 * emits structured OCEL activity events in real time.
 *
 * Each SSE event has the shape:
 *   { activity, raw_line, timestamp_ms, detail?, line_no }
 *
 * The endpoint:
 * 1. Sends all lines already in the log (catch-up mode)
 * 2. Polls for new lines every 500 ms until the client disconnects
 *
 * OCEL activity classification mirrors the Rust CookLogParser pattern table.
 * Duplicate activities (same activity name already seen) are still emitted so
 * the client can see progress through repeated cook stages (e.g. PackageCooking
 * fires for every asset). A `line_no` counter lets the client detect gaps.
 */

import { createEventStream } from 'h3';
import { createReadStream, statSync, existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { createInterface } from 'node:readline';
import { join } from 'node:path';

const LOG_PATH = join(homedir(), 'ue4-cook-latest.log');
const POLL_MS = 500;

interface CookLogEvent {
  activity: string;
  raw_line: string;
  timestamp_ms: number;
  detail?: string;
  line_no: number;
}

// Pattern table: mirrors Rust CookLogParser COOK_PATTERNS (tools/rocket-sdk/src/html5.rs).
// Order matters — first match wins. Keep in sync with the Rust table.
const PATTERNS: Array<{ match: string; activity: string }> = [
  // ── UAT entry / setup ────────────────────────────────────────────────────
  { match: 'BuildCookRun',                       activity: 'CookStarted' },
  { match: 'HTML5Setup.sh',                      activity: 'HTML5SetupStarted' },
  { match: 'HTML5Setup',                         activity: 'HTML5SetupStarted' },
  { match: 'Success!',                           activity: 'HTML5SetupComplete' },
  // ── Cook phase ───────────────────────────────────────────────────────────
  { match: 'LogCook: Display: Cooking package',  activity: 'PackageCooking' },
  { match: 'LogCook: Display: Cook complete',    activity: 'CookComplete' },
  { match: 'Total cook time',                    activity: 'CookComplete' },
  { match: 'LogCook: Display: Finished cooking', activity: 'CookComplete' },
  // ── Shader compilation ───────────────────────────────────────────────────
  { match: 'LogShaderCompilers:',                activity: 'ShaderCompileStarted' },
  { match: 'ShaderCompileWorker',                activity: 'ShaderCompileStarted' },
  { match: 'Shaders compiled',                   activity: 'ShadersCompiled' },
  // ── Asset save phase ─────────────────────────────────────────────────────
  { match: 'LogSave: Display: Saving package',   activity: 'AssetSaveStarted' },
  { match: 'LogSave: Display: Saving cooked',    activity: 'AssetSaveStarted' },
  // ── WASM / Emscripten compilation ────────────────────────────────────────
  { match: 'LogHTML5PlatformEditor',             activity: 'WasmBuildStarted' },
  { match: 'emcc',                               activity: 'EmscriptenInvoked' },
  { match: 'wasm-opt',                           activity: 'WasmOptimized' },
  // ── Pak / staging ────────────────────────────────────────────────────────
  { match: 'LogPak: Display: Collecting files',  activity: 'PakStarted' },
  { match: 'LogPak: Display: Created pak file',  activity: 'PakComplete' },
  { match: 'LogStageAndPackage',                 activity: 'StagingStarted' },
  { match: 'Staging complete',                   activity: 'StagingComplete' },
  { match: 'Archiving',                          activity: 'ArchiveStarted' },
  // ── Package finalisation ─────────────────────────────────────────────────
  { match: 'Packaging complete',                 activity: 'PackageComplete' },
  { match: 'Package was created',                activity: 'PackageCreated' },
  { match: 'BuildCookRun: Completed',            activity: 'CookFinished' },
  // ── Errors (last — only if no success pattern matched first) ─────────────
  { match: 'CookLog: Error:',                    activity: 'CookError' },
  { match: 'Error: Error:',                      activity: 'CookError' },
  { match: 'Error:',                             activity: 'CookError' },
  { match: 'ERROR:',                             activity: 'CookError' },
  { match: 'FAILED:',                            activity: 'CookFailed' },
  { match: 'returned exit code',                 activity: 'CookFailed' },
  { match: 'exception was thrown',               activity: 'CookFailed' },
];

function classifyLine(line: string): Pick<CookLogEvent, 'activity' | 'detail'> | null {
  for (const { match, activity } of PATTERNS) {
    if (line.includes(match)) {
      let detail: string | undefined;
      if (activity === 'PackageCooking') {
        detail = line.split('Cooking package:')[1]?.trim();
      } else if (activity === 'CookComplete') {
        detail = line.split('Total cook time')[1]?.trim();
      } else if (activity === 'PakComplete') {
        detail = line.split('Created pak file')[1]?.trim();
      } else if (activity === 'CookError' || activity === 'CookFailed') {
        detail = line.trim();
      }
      return { activity, detail };
    }
  }
  return null;
}

async function readLinesFrom(path: string, fromByte: number): Promise<{ lines: string[]; newByte: number }> {
  if (!existsSync(path)) return { lines: [], newByte: 0 };
  const stat = statSync(path);
  if (stat.size <= fromByte) return { lines: [], newByte: fromByte };

  return new Promise((resolve) => {
    const stream = createReadStream(path, { start: fromByte });
    const rl = createInterface({ input: stream });
    const lines: string[] = [];
    rl.on('line', l => lines.push(l));
    rl.on('close', () => resolve({ lines, newByte: stat.size }));
    rl.on('error', () => resolve({ lines: [], newByte: fromByte }));
  });
}

export default defineEventHandler(async (event) => {
  const stream = createEventStream(event);

  let byteOffset = 0;
  let lineNo = 0;
  const cookStartMs = Date.now();

  // Emit a heartbeat first so the client knows the stream is open.
  await stream.push({ data: JSON.stringify({ activity: 'StreamOpened', line_no: 0, timestamp_ms: cookStartMs, raw_line: '' }) });

  const poll = async () => {
    const { lines, newByte } = await readLinesFrom(LOG_PATH, byteOffset);
    byteOffset = newByte;

    for (const raw_line of lines) {
      lineNo++;
      const classified = classifyLine(raw_line);
      if (!classified) continue;

      const evt: CookLogEvent = {
        activity: classified.activity,
        raw_line,
        timestamp_ms: cookStartMs + lineNo * 100,
        detail: classified.detail,
        line_no: lineNo,
      };
      await stream.push({ data: JSON.stringify(evt) });
    }
  };

  // Catch-up: send all lines already in the log.
  await poll();

  // Tail: poll for new lines until client disconnects.
  const interval = setInterval(poll, POLL_MS);
  stream.onClosed(() => clearInterval(interval));

  return stream.send();
});
