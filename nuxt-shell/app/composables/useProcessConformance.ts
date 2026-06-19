/**
 * useProcessConformance — Van der Aalst process conformance for the game session pipeline.
 *
 * Declared process model (required activity sequence):
 *   GameSessionStarted → FrameRendered → InputAdmitted*
 *
 * This composable queries session_lifecycle_summary (migration 000004) and checks
 * whether each session's distinct_activities set conforms to the declared model.
 *
 * Conformance metrics (from pm4py's four quality dimensions):
 *   fitness    — fraction of sessions that include all required activities
 *   precision  — fraction of sessions with no unexpected activity types
 *   simplicity — 1.0 (the declared model has no branches — single trace)
 *   generalization — fraction of required activities seen across all sessions
 *
 * The Van der Aalst doctrine: these numbers must be derivable from the event log.
 * They are not flags. A pipeline is conformant only when fitness >= threshold.
 */

export interface SessionLifecycleSummary {
  session_id: string;
  event_count: number;
  distinct_activities: string[];
  duration_ms: number | null;
  latest_verdict: string | null;
}

export interface ConformanceResult {
  fitness: number;
  precision: number;
  simplicity: number;
  generalization: number;
  conformant_sessions: number;
  total_sessions: number;
  non_conformant: Array<{ session_id: string; missing: string[]; extra: string[] }>;
  all_activities_seen: string[];
}

// The lawful process model — exactly these activities must appear, in any order,
// but GameSessionStarted must come before FrameRendered (enforced by verify_event_chain)
const REQUIRED_ACTIVITIES = ['GameSessionStarted', 'FrameRendered'];
const EXPECTED_ACTIVITIES = new Set(['GameSessionStarted', 'FrameRendered', 'InputAdmitted']);

function conformanceOf(sessions: SessionLifecycleSummary[]): ConformanceResult {
  if (sessions.length === 0) {
    return { fitness: 0, precision: 0, simplicity: 1, generalization: 0, conformant_sessions: 0, total_sessions: 0, non_conformant: [], all_activities_seen: [] };
  }

  const allActivities = new Set<string>();
  let conformantCount = 0;
  const nonConformant: ConformanceResult['non_conformant'] = [];

  for (const s of sessions) {
    const seen = new Set(s.distinct_activities ?? []);
    for (const a of seen) allActivities.add(a);

    const missing = REQUIRED_ACTIVITIES.filter(r => !seen.has(r));
    const extra = [...seen].filter(a => !EXPECTED_ACTIVITIES.has(a));

    if (missing.length === 0 && extra.length === 0) {
      conformantCount++;
    } else {
      nonConformant.push({ session_id: s.session_id, missing, extra });
    }
  }

  const fitness = conformantCount / sessions.length;
  // Precision: sessions with no extra/unexpected activities
  const precisionNumerator = sessions.filter(s => {
    const seen = new Set(s.distinct_activities ?? []);
    return [...seen].every(a => EXPECTED_ACTIVITIES.has(a));
  }).length;
  const precision = precisionNumerator / sessions.length;
  // Generalization: how many of the required activities appear at least once across all sessions
  const generalization = REQUIRED_ACTIVITIES.filter(r => allActivities.has(r)).length / REQUIRED_ACTIVITIES.length;

  return {
    fitness,
    precision,
    simplicity: 1.0, // single-trace declared model has simplicity = 1
    generalization,
    conformant_sessions: conformantCount,
    total_sessions: sessions.length,
    non_conformant: nonConformant,
    all_activities_seen: [...allActivities].sort(),
  };
}

export function useProcessConformance() {
  const { client } = useRocketSupabase();

  const sessions = ref<SessionLifecycleSummary[]>([]);
  const conformance = ref<ConformanceResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const fitnessLabel = computed(() => {
    const f = conformance.value?.fitness ?? null;
    if (f === null) return '—';
    if (f >= 0.95) return 'CONFORMANT';
    if (f >= 0.75) return 'DEGRADED';
    if (f >= 0.5) return 'PARTIAL';
    return 'NON-CONFORMANT';
  });

  const fitnessColor = computed(() => {
    const f = conformance.value?.fitness ?? null;
    if (f === null) return '#64748b';
    if (f >= 0.95) return '#22c55e';
    if (f >= 0.75) return '#f59e0b';
    if (f >= 0.5) return '#f97316';
    return '#ef4444';
  });

  async function load(limit = 100) {
    loading.value = true;
    error.value = null;
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const { data, error: err } = await (client as any)
        .from('session_lifecycle_summary')
        .select('session_id, event_count, distinct_activities, duration_ms, latest_verdict')
        .order('session_id', { ascending: false })
        .limit(limit);
      if (err) throw new Error(err.message);
      sessions.value = data ?? [];
      conformance.value = conformanceOf(sessions.value);
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load conformance data';
    } finally {
      loading.value = false;
    }
  }

  return {
    sessions: readonly(sessions),
    conformance: readonly(conformance),
    loading: readonly(loading),
    error: readonly(error),
    fitnessLabel,
    fitnessColor,
    load,
    REQUIRED_ACTIVITIES,
    EXPECTED_ACTIVITIES: [...EXPECTED_ACTIVITIES],
  };
}
