import { defineStore } from 'pinia'
import { createClient } from '@supabase/supabase-js'
import type { Session, User, SupabaseClient } from '@supabase/supabase-js'

// ── Types ──────────────────────────────────────────────────────────────────────

export type UserRole = 'admin' | 'moderator' | 'player' | null

export interface AuthState {
  session: Session | null
  user: User | null
  role: UserRole
  initialized: boolean
}

// ── Store ──────────────────────────────────────────────────────────────────────

export const useAuthStore = defineStore('auth', () => {
  // State
  const session = ref<Session | null>(null)
  const user = ref<User | null>(null)
  const role = ref<UserRole>(null)
  const initialized = ref(false)

  // Supabase client — lazily created from runtime config so SSR doesn't blow up
  let _client: SupabaseClient | null = null

  function getClient(): SupabaseClient {
    if (_client) return _client
    const config = useRuntimeConfig()
    const url = config.public.supabaseUrl || 'http://localhost:54321'
    const anonKey = config.public.supabaseAnonKey || 'placeholder'
    _client = createClient(url, anonKey)
    return _client
  }

  // ── Computed getters ─────────────────────────────────────────────────────────

  const isAuthenticated = computed(() => !!session.value)

  const isAdmin = computed(() => role.value === 'admin')

  const isModerator = computed(() => role.value === 'moderator' || role.value === 'admin')

  const userRole = computed((): UserRole => role.value)

  const displayName = computed(() => {
    if (!user.value) return null
    return (
      (user.value.user_metadata?.['full_name'] as string | undefined) ??
      user.value.email?.split('@')[0] ??
      'Operator'
    )
  })

  // ── Private helpers ──────────────────────────────────────────────────────────

  function _syncFromSession(incoming: Session | null): void {
    session.value = incoming
    user.value = incoming?.user ?? null

    // Role is stored in app_metadata.role (set server-side via service role key).
    // Fall back to user_metadata.role for dev convenience.
    const rawRole =
      (incoming?.user?.app_metadata?.['role'] as string | undefined) ??
      (incoming?.user?.user_metadata?.['role'] as string | undefined) ??
      null

    if (rawRole === 'admin') {
      role.value = 'admin'
    } else if (rawRole === 'moderator') {
      role.value = 'moderator'
    } else if (incoming?.user) {
      role.value = 'player'
    } else {
      role.value = null
    }
  }

  // ── Public actions ───────────────────────────────────────────────────────────

  /**
   * Set up the Supabase onAuthStateChange listener exactly once.
   * Call this from the Nuxt plugin so it runs before any route renders.
   */
  async function initialize(): Promise<void> {
    if (initialized.value) return

    const client = getClient()

    // Hydrate from existing session (important for SSR/hot-reload recovery)
    const { data } = await client.auth.getSession()
    _syncFromSession(data.session)

    // Single persistent listener — drives all subsequent state changes
    client.auth.onAuthStateChange((_event, incoming) => {
      _syncFromSession(incoming)
    })

    initialized.value = true
  }

  async function signIn(email: string, password: string): Promise<void> {
    const { error } = await getClient().auth.signInWithPassword({ email, password })
    if (error) throw error
  }

  async function signUp(email: string, password: string): Promise<void> {
    const { error } = await getClient().auth.signUp({ email, password })
    if (error) throw error
  }

  async function signOut(): Promise<void> {
    const { error } = await getClient().auth.signOut()
    if (error) throw error
    _syncFromSession(null)
  }

  return {
    // State (read-only surfaces)
    session: readonly(session),
    user: readonly(user),
    role: readonly(role),
    initialized: readonly(initialized),

    // Computed
    isAuthenticated,
    isAdmin,
    isModerator,
    userRole,
    displayName,

    // Actions
    initialize,
    signIn,
    signUp,
    signOut,
  }
})
