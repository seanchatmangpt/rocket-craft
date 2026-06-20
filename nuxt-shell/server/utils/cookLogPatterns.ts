/**
 * Cook-log → OCEL activity classification. Single source of truth for the SSE
 * endpoint (cook-log.get.ts) so the pattern table is unit-tested.
 *
 * Mirrors the Rust CookLogParser COOK_PATTERNS (tools/rocket-sdk/src/html5.rs).
 * Order matters — first match wins. If this drifts from the Rust table, the live
 * cook-progress stream silently misclassifies (or drops) stages with no signal.
 */

export interface CookLogClassification {
  activity: string;
  detail?: string;
}

// First match wins. Success/phase patterns precede error patterns so a line that
// contains both (rare) classifies as the phase, not an error.
export const COOK_LOG_PATTERNS: ReadonlyArray<{ match: string; activity: string }> = [
  // More-specific 'BuildCookRun: Completed' MUST precede the generic 'BuildCookRun'
  // (first match wins) — otherwise the completion line is misread as CookStarted
  // and CookFinished is never emitted.
  { match: 'BuildCookRun: Completed', activity: 'CookFinished' },
  { match: 'BuildCookRun', activity: 'CookStarted' },
  { match: 'HTML5Setup.sh', activity: 'HTML5SetupStarted' },
  { match: 'HTML5Setup', activity: 'HTML5SetupStarted' },
  { match: 'Success!', activity: 'HTML5SetupComplete' },
  { match: 'LogCook: Display: Cooking package', activity: 'PackageCooking' },
  { match: 'LogCook: Display: Cook complete', activity: 'CookComplete' },
  { match: 'Total cook time', activity: 'CookComplete' },
  { match: 'LogCook: Display: Finished cooking', activity: 'CookComplete' },
  { match: 'LogShaderCompilers:', activity: 'ShaderCompileStarted' },
  { match: 'ShaderCompileWorker', activity: 'ShaderCompileStarted' },
  { match: 'Shaders compiled', activity: 'ShadersCompiled' },
  { match: 'LogSave: Display: Saving package', activity: 'AssetSaveStarted' },
  { match: 'LogSave: Display: Saving cooked', activity: 'AssetSaveStarted' },
  { match: 'LogHTML5PlatformEditor', activity: 'WasmBuildStarted' },
  { match: 'emcc', activity: 'EmscriptenInvoked' },
  { match: 'wasm-opt', activity: 'WasmOptimized' },
  { match: 'LogPak: Display: Collecting files', activity: 'PakStarted' },
  { match: 'LogPak: Display: Created pak file', activity: 'PakComplete' },
  { match: 'LogStageAndPackage', activity: 'StagingStarted' },
  { match: 'Staging complete', activity: 'StagingComplete' },
  { match: 'Archiving', activity: 'ArchiveStarted' },
  { match: 'Packaging complete', activity: 'PackageComplete' },
  { match: 'Package was created', activity: 'PackageCreated' },
  { match: 'CookLog: Error:', activity: 'CookError' },
  { match: 'Error: Error:', activity: 'CookError' },
  { match: 'Error:', activity: 'CookError' },
  { match: 'ERROR:', activity: 'CookError' },
  { match: 'FAILED:', activity: 'CookFailed' },
  { match: 'returned exit code', activity: 'CookFailed' },
  { match: 'exception was thrown', activity: 'CookFailed' },
];

/** Classify one cook-log line into an OCEL activity (+ optional detail), or null. */
export function classifyCookLogLine(line: string): CookLogClassification | null {
  for (const { match, activity } of COOK_LOG_PATTERNS) {
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
