// esbuild build script for pwa-staff TypeScript sources.
// Injects SUPABASE_URL and SUPABASE_ANON_KEY as compile-time constants so the
// production bundle never falls through to the localhost defaults in config.ts.
//
// Usage (CI / Vercel):
//   SUPABASE_URL=https://…supabase.co SUPABASE_ANON_KEY=eyJ… node esbuild.config.mjs
//
// Local dev (no env vars set):
//   node esbuild.config.mjs
//   → __SUPABASE_URL__ / __SUPABASE_ANON_KEY__ remain undefined at runtime so
//     config.ts falls through to the localhost 54321 defaults automatically.

import { build } from 'esbuild';
import { glob } from 'fs/promises';

// Collect all src/*.ts entry points dynamically so new modules are picked up
// without updating this file.
const entries = [];
for await (const f of glob('src/*.ts')) {
  entries.push(f);
}

const supabaseUrl = process.env.SUPABASE_URL ?? '';
const supabaseAnonKey = process.env.SUPABASE_ANON_KEY ?? '';

await build({
  entryPoints: entries,
  bundle: true,
  outdir: 'dist',
  // Replace __SUPABASE_URL__ and __SUPABASE_ANON_KEY__ with the env-var values
  // at build time (values become JS string literals in the output).
  // When the env vars are empty the defines are omitted so config.ts falls
  // back to runtime resolution (window.__ROCKET_CONFIG__) then to local defaults.
  define:
    supabaseUrl && supabaseAnonKey
      ? {
          __SUPABASE_URL__: JSON.stringify(supabaseUrl),
          __SUPABASE_ANON_KEY__: JSON.stringify(supabaseAnonKey),
        }
      : {},
  platform: 'browser',
  format: 'esm',
});
