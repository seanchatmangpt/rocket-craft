/**
 * useRocketSupabase — typed Supabase client for rocket-craft game sessions.
 *
 * Pattern ported from ~/dashboard.bak/app/composables/useSupabaseClient.ts
 * Extended with game-specific tables: game_sessions, ocel_events, game_receipts.
 *
 * Uses createSharedComposable (VueUse) so a single client instance is shared
 * across all components — no reconnection churn.
 */

import { createClient } from '@supabase/supabase-js';
import { createSharedComposable } from '@vueuse/core';

// ── Database types ────────────────────────────────────────────────────────────

export interface GameSession {
  id: string;
  player_id: string | null;
  session_started_at: string;
  session_ended_at: string | null;
  engine_source: 'real_ue4' | 'headless' | 'unknown';
  is_alive: boolean;
  ocel_event_count: number;
  receipt_hash: string | null;
  metadata: Record<string, unknown>;
  created_at: string;
}

/** DB row shape for ocel_events table (flat string[] refs, no qualifier). */
export interface OcelEventRow {
  id: string;
  session_id: string;
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  prev_hash: string | null;
  event_hash: string;
  seq: number;
  created_at: string;
}

export interface GameReceipt {
  id: string;
  session_id: string;
  verdict: 'PASS' | 'FAIL' | 'PENDING';
  milestone: string;
  ocel_event_count: number;
  ocel_lifecycle: string[];
  engine_source: string;
  receipt_hash: string;
  proven_at: string;
  payload: Record<string, unknown>;
  created_at: string;
}

type GameSessionInsert = {
  player_id: string | null;
  session_started_at: string;
  session_ended_at?: string | null;
  engine_source: 'real_ue4' | 'headless' | 'unknown';
  is_alive: boolean;
  ocel_event_count: number;
  receipt_hash?: string | null;
  metadata: Record<string, unknown>;
};

type GameSessionUpdate = Partial<Pick<GameSession, 'session_ended_at' | 'engine_source' | 'is_alive' | 'ocel_event_count' | 'receipt_hash'>>;

type OcelEventInsert = {
  session_id: string;
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  prev_hash: string | null;
  event_hash: string;
  seq: number;
};

type GameReceiptInsert = {
  session_id: string;
  verdict: 'PASS' | 'FAIL' | 'PENDING';
  milestone: string;
  ocel_event_count: number;
  ocel_lifecycle: string[];
  engine_source: string;
  receipt_hash: string;
  proven_at: string;
  payload: Record<string, unknown>;
};

export interface Database {
  public: {
    Tables: {
      game_sessions: {
        Row: GameSession;
        Insert: GameSessionInsert;
        Update: GameSessionUpdate;
      };
      ocel_events: {
        Row: OcelEventRow;
        Insert: OcelEventInsert;
        Update: Partial<OcelEventRow>;
      };
      game_receipts: {
        Row: GameReceipt;
        Insert: GameReceiptInsert;
        Update: Partial<GameReceipt>;
      };
    };
  };
}

// ── Shared client ─────────────────────────────────────────────────────────────

const _useRocketSupabase = () => {
  const config = useRuntimeConfig();

  // Use untyped client — typed wrappers live in useGameSessionPersistence
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const client = createClient(
    (config.public.supabaseUrl as string) || 'http://localhost:54321',
    (config.public.supabaseAnonKey as string) || '',
    {
      auth: { persistSession: true, autoRefreshToken: true },
      realtime: { params: { eventsPerSecond: 10 } },
    }
  );

  const user = ref<{ id: string; email?: string } | null>(null);

  // Hydrate user from existing session on mount
  if (import.meta.client) {
    client.auth.getSession().then(({ data }) => {
      user.value = data.session?.user ?? null;
    });
    client.auth.onAuthStateChange((_event, session) => {
      user.value = session?.user ?? null;
    });
  }

  return { client, user };
};

export const useRocketSupabase = createSharedComposable(_useRocketSupabase);
