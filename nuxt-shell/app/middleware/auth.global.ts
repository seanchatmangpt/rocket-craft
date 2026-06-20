/**
 * Auth middleware — redirects unauthenticated users away from protected routes.
 * Pattern: ~/seth/neako-web/middleware/test-pages.global.ts (global route guard)
 *
 * Protected routes: /profile, /receipts, /leaderboard, /game
 * Public routes: / (login), /test/*, anything else
 *
 * The check runs client-side only — Supabase session is a browser-side JWT.
 * On SSR the route is allowed through; client hydration enforces the redirect.
 */

const PROTECTED = ['/profile', '/receipts', '/leaderboard', '/game'];

export default defineNuxtRouteMiddleware(async (to) => {
  // SSR: skip — session is browser-side only
  if (import.meta.server) return;

  if (!PROTECTED.some(p => to.path.startsWith(p))) return;

  // Skip auth guard when no Supabase URL is configured (local dev, E2E without .env).
  // The real auth check only runs when a Supabase project is wired up.
  const config = useRuntimeConfig();
  if (!config.public.supabaseAnonKey) return;

  // Dev/E2E bypass: when ALLOW_ANON_GAME=1, protected routes are open without
  // login so the game + telemetry can be exercised locally and in Playwright.
  // NEVER set this in production — auth is enforced whenever the flag is off.
  if (config.public.allowAnonGame) return;

  // Lazy-import to avoid bundling Supabase in the auth guard itself
  const { user } = useRocketSupabase();

  // Give the composable one tick to hydrate from localStorage
  if (!user.value) {
    await nextTick();
  }

  if (!user.value) {
    return navigateTo(`/?redirect=${encodeURIComponent(to.fullPath)}`);
  }
});
