import { describe, it, expect } from 'vitest';
import { classifyCookLogLine, COOK_LOG_PATTERNS } from '../../server/utils/cookLogPatterns';

/**
 * Cook-log classification contract. The SSE endpoint streams cook progress by
 * classifying UAT log lines into OCEL activities. Untested, a pattern drift would
 * silently misclassify or drop cook stages with no other signal.
 */

describe('classifyCookLogLine', () => {
  it('classifies the canonical cook lifecycle lines', () => {
    const cases: Array<[string, string]> = [
      ['RunUAT.sh BuildCookRun -project=Brm', 'CookStarted'],
      ['Running ./HTML5Setup.sh', 'HTML5SetupStarted'],
      ['LogCook: Display: Cooking package: /Game/Maps/Entry', 'PackageCooking'],
      ['LogCook: Display: Cook complete', 'CookComplete'],
      ['LogShaderCompilers: compiling', 'ShaderCompileStarted'],
      ['Shaders compiled successfully', 'ShadersCompiled'],
      ['LogSave: Display: Saving package /Game/Foo', 'AssetSaveStarted'],
      ['emcc -O3 main.cpp', 'EmscriptenInvoked'],
      ['wasm-opt -O2 Brm.wasm', 'WasmOptimized'],
      ['LogPak: Display: Created pak file Brm.pak', 'PakComplete'],
      ['Staging complete', 'StagingComplete'],
      ['Packaging complete', 'PackageComplete'],
      ['BuildCookRun: Completed', 'CookFinished'],
    ];
    for (const [line, activity] of cases) {
      expect(classifyCookLogLine(line)?.activity, line).toBe(activity);
    }
  });

  it('extracts detail for PackageCooking / PakComplete', () => {
    expect(classifyCookLogLine('LogCook: Display: Cooking package: /Game/Maps/Entry')?.detail)
      .toBe('/Game/Maps/Entry');
    expect(classifyCookLogLine('LogPak: Display: Created pak file Brm.pak')?.detail)
      .toBe('Brm.pak');
  });

  it('classifies errors and failures', () => {
    expect(classifyCookLogLine('Error: Error: cook aborted')?.activity).toBe('CookError');
    expect(classifyCookLogLine('AutomationTool returned exit code 1')?.activity).toBe('CookFailed');
    expect(classifyCookLogLine('FAILED: link step')?.activity).toBe('CookFailed');
    // Error lines carry the full line as detail for forensics.
    expect(classifyCookLogLine('ERROR: missing asset')?.detail).toBe('ERROR: missing asset');
  });

  it('returns null for unmatched lines', () => {
    expect(classifyCookLogLine('LogTemp: Verbose: nothing interesting')).toBeNull();
    expect(classifyCookLogLine('')).toBeNull();
  });

  it('first-match-wins: a phase line is not misread as an error', () => {
    // BuildCookRun: Completed comes before the generic Error patterns; a line that
    // is a completion must classify as CookFinished, never CookError.
    expect(classifyCookLogLine('BuildCookRun: Completed')?.activity).toBe('CookFinished');
    // CookLog: Error: must win over the generic 'Error:' substring (more specific first).
    expect(classifyCookLogLine('CookLog: Error: bad thing')?.activity).toBe('CookError');
  });

  it('pattern table is non-empty and every entry has match + activity', () => {
    expect(COOK_LOG_PATTERNS.length).toBeGreaterThan(20);
    for (const p of COOK_LOG_PATTERNS) {
      expect(typeof p.match).toBe('string');
      expect(p.match.length).toBeGreaterThan(0);
      expect(typeof p.activity).toBe('string');
    }
  });
});
