<script setup lang="ts">
useHead({ title: 'Rocket-Craft — Pilot Profile' });

const { client, user } = useRocketSupabase();

interface PlayerRow {
  id: string;
  username: string | null;
  high_score: number;
  created_at: string;
}

interface SessionSummary {
  id: string;
  is_alive: boolean;
  ocel_event_count: number;
  engine_source: string;
  session_started_at: string;
  session_ended_at: string | null;
}

const player = ref<PlayerRow | null>(null);
const sessions = ref<SessionSummary[]>([]);
const leaderboardRank = ref<number | null>(null);
const loading = ref(true);

async function loadProfile() {
  if (!user.value) { loading.value = false; return; }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const sb = client as any;

  // Upsert player profile row on first login (idempotent via ON CONFLICT)
  await sb.rpc('upsert_player_profile', { p_username: null }).catch(() => null);

  const [playerRes, sessionsRes, rankRes] = await Promise.all([
    // players.auth_user_id FK = Supabase auth user UUID; not players.id PK
    sb.from('players').select('id, username, high_score, created_at').eq('auth_user_id', user.value.id).single(),
    sb.from('game_sessions')
      .select('id, is_alive, ocel_event_count, engine_source, session_started_at, session_ended_at')
      .eq('player_id', user.value.id)
      .order('session_started_at', { ascending: false })
      .limit(10),
    sb.from('leaderboard').select('rank').eq('player_id', user.value.id).single(),
  ]);

  player.value = playerRes.data;
  sessions.value = sessionsRes.data ?? [];
  leaderboardRank.value = rankRes.data?.rank ?? null;
  loading.value = false;
}

async function signOut() {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await (client as any).auth.signOut();
  navigateTo('/');
}

onMounted(loadProfile);
watch(user, loadProfile);

const totalEvents = computed(() => sessions.value.reduce((sum, s) => sum + (s.ocel_event_count ?? 0), 0));
const passedSessions = computed(() => sessions.value.filter(s => s.ocel_event_count > 0).length);
const memberSince = computed(() => player.value ? new Date(player.value.created_at).toLocaleDateString('en-US', { year: 'numeric', month: 'long' }) : '');
</script>

<template>
  <main class="profile-shell">
    <header class="profile-header">
      <NuxtLink to="/game" class="back">← Mission Control</NuxtLink>
      <h1>Pilot Profile</h1>
    </header>

    <div v-if="!user" class="status">
      <p>Not authenticated.</p>
      <NuxtLink to="/" class="cta">Sign in →</NuxtLink>
    </div>

    <div v-else-if="loading" class="status">Loading pilot data…</div>

    <div v-else class="profile-grid">
      <!-- Pilot card -->
      <section class="pilot-card card">
        <div class="avatar">{{ (player?.username ?? user.email ?? 'P')[0]!.toUpperCase() }}</div>
        <h2 class="callsign">{{ player?.username ?? user.email ?? 'Anonymous Pilot' }}</h2>
        <p class="email">{{ user.email }}</p>
        <div class="stats-row">
          <div class="stat">
            <span class="stat-val">{{ leaderboardRank !== null ? `#${leaderboardRank}` : '—' }}</span>
            <span class="stat-label">Global Rank</span>
          </div>
          <div class="stat">
            <span class="stat-val">{{ player?.high_score?.toLocaleString() ?? 0 }}</span>
            <span class="stat-label">High Score</span>
          </div>
          <div class="stat">
            <span class="stat-val">{{ sessions.length }}</span>
            <span class="stat-label">Sessions</span>
          </div>
        </div>
        <p class="member-since">Pilot since {{ memberSince }}</p>
        <button class="sign-out-btn" @click="signOut">Sign Out</button>
      </section>

      <!-- Session history -->
      <section class="sessions-card card">
        <h3>Recent Sessions</h3>
        <div class="session-stats">
          <span>{{ totalEvents.toLocaleString() }} total OCEL events</span>
          <span>{{ passedSessions }} sessions with proof</span>
        </div>
        <ul class="session-list">
          <li v-for="s in sessions" :key="s.id" class="session-row">
            <span class="session-engine" :class="s.engine_source">{{ s.engine_source }}</span>
            <span class="session-events">{{ s.ocel_event_count }} events</span>
            <span class="session-status" :class="{ alive: s.is_alive }">{{ s.is_alive ? '● LIVE' : s.session_ended_at ? 'ended' : 'open' }}</span>
            <span class="session-date">{{ new Date(s.session_started_at).toLocaleDateString() }}</span>
          </li>
        </ul>
        <div v-if="sessions.length === 0" class="no-sessions">No sessions yet — launch a game.</div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.profile-shell {
  min-height: 100dvh; background: #0b0f19; color: #e0e0e0;
  font-family: 'Courier New', monospace; padding: 1rem;
}
.profile-header {
  display: flex; align-items: center; gap: 1rem;
  border-bottom: 1px solid #1e3a5f; padding-bottom: 0.75rem; margin-bottom: 1.5rem;
}
.profile-header h1 { font-size: 1rem; color: #00f0ff; margin: 0; }
.back { color: #00f0ff; text-decoration: none; font-size: 0.85rem; }
.status { text-align: center; padding: 3rem; color: #666; }
.cta { color: #00f0ff; text-decoration: none; display: block; margin-top: 0.5rem; }
.profile-grid { display: grid; grid-template-columns: 280px 1fr; gap: 1rem; }
@media (max-width: 640px) { .profile-grid { grid-template-columns: 1fr; } }
.card { background: #0d1117; border: 1px solid #1e3a5f; padding: 1.25rem; border-radius: 4px; }
.avatar {
  width: 56px; height: 56px; background: #1e3a5f; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 1.5rem; color: #00f0ff; font-weight: bold; margin: 0 auto 0.75rem;
}
.callsign { text-align: center; font-size: 0.95rem; color: #e0e0e0; margin: 0 0 0.25rem; }
.email { text-align: center; font-size: 0.7rem; color: #555; margin: 0 0 1rem; }
.stats-row { display: flex; gap: 0.5rem; justify-content: center; margin: 0.75rem 0; }
.stat { text-align: center; flex: 1; }
.stat-val { display: block; font-size: 1.2rem; font-weight: bold; color: #00f0ff; }
.stat-label { display: block; font-size: 0.65rem; color: #555; }
.member-since { text-align: center; font-size: 0.7rem; color: #555; margin: 0.5rem 0; }
.sign-out-btn {
  display: block; width: 100%; margin-top: 1rem;
  background: none; border: 1px solid #1e3a5f; color: #666;
  padding: 0.4rem; cursor: pointer; font-family: inherit; font-size: 0.75rem;
}
.sign-out-btn:hover { border-color: #ff4444; color: #ff4444; }
.sessions-card h3 { font-size: 0.85rem; color: #888; margin: 0 0 0.5rem; font-weight: normal; letter-spacing: 0.05em; }
.session-stats { font-size: 0.7rem; color: #555; margin-bottom: 0.75rem; display: flex; gap: 1rem; }
.session-list { list-style: none; margin: 0; padding: 0; }
.session-row {
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.4rem 0; border-bottom: 1px solid #0b0f19; font-size: 0.75rem;
}
.session-engine { color: #888; min-width: 5rem; }
.session-engine.real_ue4 { color: #00c853; }
.session-events { color: #00f0ff; flex: 1; }
.session-status { color: #555; min-width: 3.5rem; }
.session-status.alive { color: #00c853; animation: pulse 2s infinite; }
.session-date { color: #555; font-size: 0.7rem; }
.no-sessions { color: #555; font-size: 0.8rem; padding: 1rem 0; }
@keyframes pulse { 0%,100% { opacity:1 } 50% { opacity:0.5 } }
</style>
