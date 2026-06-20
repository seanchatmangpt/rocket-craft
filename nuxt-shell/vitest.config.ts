import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: {
    // @noble/hashes sub-path exports use .js extensions for Node ESM.
    // Map the bare specifier used in composables to the .js path.
    alias: {
      '@noble/hashes/blake3': '@noble/hashes/blake3.js',
      '@noble/hashes/utils': '@noble/hashes/utils.js',
    },
  },
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
