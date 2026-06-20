/**
 * Nuxt test helpers — thin wrappers over @nuxt/test-utils.
 * Use only in test/nuxt/ files, never in e2e/ or unit/.
 *
 * Law: mixing @nuxt/test-utils helpers with Playwright calls in the same file
 * causes environment pollution. Keep them separated.
 */

// Dynamic import guards: if @nuxt/test-utils is not installed, exports are
// marked ADMIT_LATER via residuals rather than throwing at import time.

let _nuxtTestUtils: typeof import('@nuxt/test-utils/e2e') | null = null

async function getNuxtTestUtils() {
  if (!_nuxtTestUtils) {
    try {
      _nuxtTestUtils = await import('@nuxt/test-utils/e2e')
    } catch {
      throw new Error(
        '[rocket-craft-test-utils] @nuxt/test-utils is not installed. ' +
        'Add it as a dev dependency: pnpm add -D @nuxt/test-utils'
      )
    }
  }
  return _nuxtTestUtils
}

export async function setupRocketNuxtE2E(opts: Record<string, unknown> = {}) {
  const { setup } = await getNuxtTestUtils()
  return setup({ ...opts })
}

export async function createRocketPage(path = '/') {
  const { createPage } = await getNuxtTestUtils()
  return createPage(path)
}

// These are re-exported for convenience from @nuxt/test-utils/runtime
// They only work inside a Nuxt test environment (test/nuxt/)
export async function mountRocketSuspended<T>(
  component: T,
  opts?: Record<string, unknown>
) {
  const { mountSuspended } = await import('@nuxt/test-utils/runtime' as string)
  return (mountSuspended as Function)(component, opts)
}

export async function renderRocketSuspended<T>(
  component: T,
  opts?: Record<string, unknown>
) {
  const { renderSuspended } = await import('@nuxt/test-utils/runtime' as string)
  return (renderSuspended as Function)(component, opts)
}

export async function mockRocketNuxtImport(name: string, factory: () => unknown) {
  const { mockNuxtImport } = await import('@nuxt/test-utils/runtime' as string)
  return (mockNuxtImport as Function)(name, factory)
}
