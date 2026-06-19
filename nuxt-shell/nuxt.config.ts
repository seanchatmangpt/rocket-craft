// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },

  // COOP/COEP required for SharedArrayBuffer (UE4 wasm-threads).
  // In dev: Vite proxy forwards /manufactured/** to the UE4 asset server (port 8080).
  vite: {
    server: {
      headers: {
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Cross-Origin-Embedder-Policy': 'require-corp',
      },
      proxy: {
        '/manufactured': {
          target: 'http://localhost:8080',
          changeOrigin: true,
          // Strip /manufactured prefix so Brm.wasm is fetched as /Brm.wasm on the asset server
          rewrite: (path: string) => path.replace(/^\/manufactured/, ''),
        },
      },
    },
  },

  // COOP/COEP for production/SSR (nitro route rules)
  nitro: {
    routeRules: {
      '/**': {
        headers: {
          'Cross-Origin-Opener-Policy': 'same-origin',
          'Cross-Origin-Embedder-Policy': 'require-corp',
        },
      },
    },
  },

  modules: [
    '@nuxt/ui',       // 125+ accessible components: forms, dashboard, drawer, toast, command palette, chat
    '@vueuse/nuxt',   // auto-imports all VueUse composables (useEventListener, useFullscreen, etc.)
    '@vite-pwa/nuxt', // PWA: manifest, service worker, offline shell, update lifecycle
  ],

  pwa: {
    registerType: 'prompt', // show update prompt instead of auto-reloading
    manifest: {
      name: 'Rocket-Craft Mission Control',
      short_name: 'Rocket-Craft',
      description: 'Browser-native mission-control shell with UE4/WASM projected world-body.',
      theme_color: '#0b0f19',
      background_color: '#0b0f19',
      display: 'standalone',
      orientation: 'landscape',
      scope: '/',
      start_url: '/',
      icons: [
        { src: '/pwa/icon-192.png', sizes: '192x192', type: 'image/png' },
        { src: '/pwa/icon-512.png', sizes: '512x512', type: 'image/png' },
      ],
    },
    workbox: {
      navigateFallback: '/',
      // Cache the Nuxt shell and control-plane assets only.
      // UE4/WASM packages are NOT blindly precached — they can be hundreds of MB
      // and must be versioned explicitly with a cache budget if caching is needed.
      globPatterns: ['**/*.{js,css,html,png,svg,ico,json,woff2}'],
      globIgnores: ['**/Brm.*', '**/*.wasm', '**/*.data'],
    },
    client: {
      installPrompt: true,
      periodicSyncForUpdates: 3600,
    },
    devOptions: {
      enabled: true,
      type: 'module',
    },
  },

  runtimeConfig: {
    public: {
      supabaseUrl: process.env.SUPABASE_URL ?? '',
      supabaseAnonKey: process.env.SUPABASE_ANON_KEY ?? '',
    },
  },
})
