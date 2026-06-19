<script setup lang="ts">
import { createClient } from '@supabase/supabase-js';

definePageMeta({ layout: false });
useHead({ title: 'Rocket-Craft — Sign In' });

const config = useRuntimeConfig();
const router = useRouter();
const toast = useToast();

const email = ref('');
const password = ref('');
const loading = ref(false);
const mode = ref<'login' | 'signup'>('login');

const supabase = computed(() =>
  createClient(config.public.supabaseUrl || 'http://localhost:54321', config.public.supabaseAnonKey || 'placeholder')
);

async function submit() {
  if (!email.value || !password.value) return;
  loading.value = true;
  try {
    const fn = mode.value === 'login'
      ? supabase.value.auth.signInWithPassword({ email: email.value, password: password.value })
      : supabase.value.auth.signUp({ email: email.value, password: password.value });
    const { error } = await fn;
    if (error) {
      toast.add({ title: error.message, color: 'error', duration: 4000 });
    } else {
      toast.add({ title: mode.value === 'login' ? 'Signed in' : 'Account created', color: 'success', duration: 2000 });
      router.push('/game');
    }
  } finally {
    loading.value = false;
  }
}

function toggleMode() {
  mode.value = mode.value === 'login' ? 'signup' : 'login';
}

// Skip login if already authenticated
onMounted(async () => {
  if (!config.public.supabaseUrl) return;
  const sb = supabase.value;
  const { data } = await sb.auth.getSession();
  if (data.session) router.replace('/game');
});
</script>

<template>
  <div class="login-shell">
    <div class="login-card" role="main" aria-label="Sign in to Rocket-Craft">
      <h1 class="brand">ROCKET-CRAFT</h1>
      <p class="subtitle">Mission Control Access</p>

      <UForm class="auth-form" @submit="submit">
        <UFormField label="Email" name="email">
          <UInput
            v-model="email"
            type="email"
            placeholder="operator@rocket-craft.io"
            autocomplete="email"
            autofocus
            data-testid="input-email"
            size="lg"
          />
        </UFormField>

        <UFormField label="Password" name="password">
          <UInput
            v-model="password"
            type="password"
            placeholder="••••••••"
            autocomplete="current-password"
            data-testid="input-password"
            size="lg"
          />
        </UFormField>

        <UButton
          type="submit"
          color="primary"
          size="lg"
          :loading="loading"
          :disabled="!email || !password"
          block
          data-testid="btn-auth-submit"
        >
          {{ mode === 'login' ? 'Sign In' : 'Create Account' }}
        </UButton>
      </UForm>

      <div class="toggle-row">
        <span>{{ mode === 'login' ? "Don't have an account?" : 'Already have an account?' }}</span>
        <UButton variant="link" size="sm" data-testid="btn-toggle-mode" @click="toggleMode">
          {{ mode === 'login' ? 'Sign up' : 'Sign in' }}
        </UButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-shell {
  min-height: 100dvh;
  background: #0b0f19;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Courier New', monospace;
}
.login-card {
  width: 100%;
  max-width: 400px;
  padding: 2rem;
  background: #0d1117;
  border: 1px solid #1e3a5f;
  border-radius: 8px;
}
.brand {
  font-size: 1.5rem;
  font-weight: bold;
  color: #00f0ff;
  letter-spacing: 3px;
  margin: 0 0 0.25rem;
}
.subtitle {
  color: #666;
  font-size: 0.8rem;
  margin: 0 0 1.5rem;
}
.auth-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.toggle-row {
  margin-top: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: #888;
}
</style>
