# Nuxt & Next Supabase Reuse Plan

*Note: Global search across `~/` confirms no Next.js surfaces are actively using Supabase. The plan will focus on Nuxt surfaces, leveraging existing module integrations found in the user's workspace.*

| Source Project | File / Pattern | Classification | Strategy |
| --- | --- | --- | --- |
| `dashboard.bak` | `@nuxtjs/supabase` setup | **REUSE_AS_IS** | Extract the module configuration from `dashboard.bak/nuxt.config.ts` to implement first-class Nuxt-native auth in `rocket-craft/nuxt-shell` instead of manual client setup. |
| `rocket-craft` | `pwa-staff/src/lib/supabaseClient.ts` | **PORT_TO_NUXT** | Deprecate the manual vanilla TS initialization in favor of the auto-imported `useSupabaseClient()` provided by `@nuxtjs/supabase`. |
| `rocket-craft` | `pwa-staff/src/auth.ts` | **PORT_TO_NUXT** | Replace manual `onAuthStateChange` listeners with Nuxt middleware and `useSupabaseUser()`. |
| `rocket-craft` | `pwa-staff/src/leaderboard.ts` | **PORT_TO_NUXT** | Convert to a Nuxt Vue component using `onMounted` with `useSupabaseClient().channel(...)` for realtime postgres subscriptions. |
| `neako` | Route Redirection logic | **KEEP_AS_REFERENCE** | Use the `redirectOptions` configuration found in `neako` to protect authenticated paths in the Nuxt app. |
| `cns`, `dogturk` | Supabase Test Mocks | **KEEP_AS_REFERENCE** | Use testing architectures found in `cns` for validating Supabase integrations in Nuxt without touching prod logic. |
| All | Vanilla Supabase Auth UI | **REFUSE** | Refuse manual vanilla DOM manipulations (e.g. `document.getElementById`) for login. Move entirely to Vue `<template>` forms. |

## Next Steps
1. Add `@nuxtjs/supabase` to `rocket-craft/nuxt-shell/package.json` to leverage the official Nuxt integrations we catalogued from `dashboard.bak` and `neako`.
2. Port the query logic (`.from()`, `.channel()`) out of `pwa-staff` and into Vue components in `nuxt-shell`.
3. Do not modify the backend or write any SQL migrations; the tables are already functional.
