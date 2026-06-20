# Supabase Code Inventory (Global Search)

## Projects Found with Nuxt + Supabase
1. **`~/rocket-craft`**
   - `nuxt-shell/package.json`: `@supabase/supabase-js`
   - `pwa-staff/package.json`: `@supabase/supabase-js`
   - Configs: `nuxt.config.ts` exposes public env vars for Supabase.
   - Usage: Native `@supabase/supabase-js` used for auth, leaderboards, telemetry.
2. **`~/dashboard.bak`**
   - `package.json`: `@supabase/supabase-js`, `@nuxtjs/supabase`
   - Configs: `nuxt.config.ts` configures `@nuxtjs/supabase` with `redirectOptions`.
   - Usage: Utilizes the `@nuxtjs/supabase` Nuxt module integration.
3. **`~/cns` (CNS Forge)**
   - `package.json`: `nuxt`, `@supabase/supabase-js`
4. **`~/dogturk` (remo-dash)**
   - `package.json`: `nuxt`, `@supabase/supabase-js`
5. **`~/neako`**
   - Configs: `nuxt.config.ts` configures `@nuxtjs/supabase` with redirect configs.

*Note: No Next.js surfaces with Supabase were found. Other projects found with Supabase were Expo/React Native (`expo-supabase-ai-template`, `pcp`, `zoeapp`, `zoela`).*

## Catalogued Supabase Patterns Found

### Configuration & Module Initialization
- **`~/dashboard.bak/nuxt.config.ts` & `~/neako/nuxt.config.ts`**: Both utilize the `@nuxtjs/supabase` module, configuring `redirectOptions` (e.g. login `/auth/login`, callback `/auth/callback`).

### Client Initialization (Vanilla JS/TS)
- **`~/rocket-craft/pwa-staff/src/lib/supabaseClient.ts`**: Singleton initialization using `createClient(supabaseUrl, supabaseAnonKey)`.

### Authentication Logic
- **`~/rocket-craft/pwa-staff/src/auth.ts`**: Uses `getSession()`, `onAuthStateChange()`, `signInWithPassword()`, and `signOut()`.

### Database Queries & Subscriptions
- **`~/rocket-craft/pwa-staff/src/leaderboard.ts`**: `from('leaderboard').select(...)` and `channel('public:leaderboard').on('postgres_changes', ...).subscribe()`.
- **`~/rocket-craft/pwa-staff/src/admin.ts`**: `from('players').update(...)`.
- **`~/rocket-craft/pwa-staff/src/profile.ts`**: `from('world_specs').upsert(...)`.
