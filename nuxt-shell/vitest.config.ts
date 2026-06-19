import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'nuxt-shell-unit',
    // test/unit/** = unit tests (happy-dom, no server)
    // tests/**     = contract/API tests (MOCK_API=1 or live Nitro)
    include: ['test/unit/**/*.test.ts', 'tests/**/*.test.ts'],
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./test/setup.ts'],
  },
})
