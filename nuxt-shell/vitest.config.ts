import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'nuxt-shell-unit',
    include: ['test/unit/**/*.test.ts'],
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./test/setup.ts'],
  },
})
