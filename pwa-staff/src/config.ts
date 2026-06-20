// Runtime and build-time configuration for Supabase.
//
// Resolution order (first defined wins):
//   1. window.__ROCKET_CONFIG__  — injected at runtime by the production server/CDN
//   2. __SUPABASE_URL__ / __SUPABASE_ANON_KEY__ — replaced by esbuild --define at build time
//      (these map to SUPABASE_URL / SUPABASE_ANON_KEY environment variables)
//   3. Hard-coded local development defaults

declare global {
  interface Window {
    __ROCKET_CONFIG__?: {
      supabaseUrl?: string;
      supabaseAnonKey?: string;
    };
  }

  // Replaced by esbuild --define at build time; declared here so TypeScript is happy.
  const __SUPABASE_URL__: string | undefined;
  const __SUPABASE_ANON_KEY__: string | undefined;
}

function resolveConfig(): { supabaseUrl: string; supabaseAnonKey: string } {
  // 1. Runtime injection (production CDN / server renders a <script> that sets this)
  if (typeof window !== 'undefined' && window.__ROCKET_CONFIG__) {
    const { supabaseUrl, supabaseAnonKey } = window.__ROCKET_CONFIG__;
    if (supabaseUrl && supabaseAnonKey) {
      return { supabaseUrl, supabaseAnonKey };
    }
  }

  // 2. Build-time define (esbuild replaces __SUPABASE_URL__ / __SUPABASE_ANON_KEY__
  //    with the literal string values from the SUPABASE_URL / SUPABASE_ANON_KEY env vars)
  const buildUrl =
    typeof __SUPABASE_URL__ !== 'undefined' ? __SUPABASE_URL__ : undefined;
  const buildKey =
    typeof __SUPABASE_ANON_KEY__ !== 'undefined' ? __SUPABASE_ANON_KEY__ : undefined;
  if (buildUrl && buildKey) {
    return { supabaseUrl: buildUrl, supabaseAnonKey: buildKey };
  }

  // 3. Local development fallback
  return {
    supabaseUrl: 'http://127.0.0.1:54321',
    supabaseAnonKey: 'sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH',
  };
}

const { supabaseUrl, supabaseAnonKey } = resolveConfig();

export { supabaseUrl, supabaseAnonKey };
