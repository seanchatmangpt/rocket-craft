// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';

/**
 * Architectural fitness function — prevents the proof-format drift class that
 * produced multiple real bugs this project: six independent event_hash writers
 * and three Ed25519 payload builders that silently disagreed.
 *
 * RULES (enforced at test time, so CI blocks regressions):
 *  1. No file may use the attr-dropping array-replacer canonical pattern
 *     `JSON.stringify(x, Object.keys(x).sort())` — it drops nested keys and
 *     diverges from the recursive canonical. Use canonicalJSON / canonicalOcelEventHash.
 *  2. Only the approved shared utils may DEFINE a `canonicalize`/`canonicalJSON`
 *     function; endpoints/composables must import the shared one.
 *
 * If you're adding a new hashing surface: import the shared util, don't re-implement.
 */

const ROOT = join(__dirname, '..', '..');
const SCAN_DIRS = [join(ROOT, 'server'), join(ROOT, 'app')];
const APPROVED_DEFINERS = ['canonicalJson.ts', 'ocelEventHash.ts'];

function walk(dir: string): string[] {
  const out: string[] = [];
  let entries: string[] = [];
  try { entries = readdirSync(dir); } catch { return out; }
  for (const e of entries) {
    if (e === 'node_modules' || e === '.nuxt' || e === 'dist') continue;
    const p = join(dir, e);
    const st = statSync(p);
    if (st.isDirectory()) out.push(...walk(p));
    else if (/\.(ts|vue)$/.test(e)) out.push(p);
  }
  return out;
}

const FILES = SCAN_DIRS.flatMap(walk);

// Strip line comments so documentation mentioning the old pattern doesn't trip the rule.
function codeOnly(src: string): string {
  return src.split('\n').filter((l) => !l.trim().startsWith('*') && !l.trim().startsWith('//')).join('\n');
}

describe('canonical proof-format guard', () => {
  it('scans a non-trivial number of source files', () => {
    expect(FILES.length).toBeGreaterThan(20);
  });

  it('no file uses the attr-dropping array-replacer canonical pattern', () => {
    // Matches JSON.stringify(<expr>, Object.keys(<expr>).sort())
    const RE = /JSON\.stringify\s*\([^,]+,\s*Object\.keys\s*\([^)]*\)\.sort\(\)\s*\)/;
    const offenders = FILES.filter((f) => RE.test(codeOnly(readFileSync(f, 'utf8'))));
    expect(offenders, `Use canonicalJSON instead of the array-replacer in:\n${offenders.join('\n')}`).toEqual([]);
  });

  it('only the approved shared utils define a canonical function', () => {
    const DEF_RE = /\bfunction\s+canonical(JSON|ize)\b|\bconst\s+canonical(JSON|ize)\s*=/;
    const offenders = FILES.filter((f) => {
      if (APPROVED_DEFINERS.some((a) => f.endsWith(a))) return false;
      return DEF_RE.test(codeOnly(readFileSync(f, 'utf8')));
    });
    expect(offenders, `Import canonicalJSON from utils instead of defining a local canonical in:\n${offenders.join('\n')}`).toEqual([]);
  });
});
