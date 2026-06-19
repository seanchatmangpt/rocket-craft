/**
 * GET /api/game/ocel-export?session_id=<uuid>
 *
 * Exports the ocel_events for a session as OCEL 2.0 JSON — the standard format
 * accepted by pm4py for process discovery and conformance checking.
 *
 * Van der Aalst doctrine: this is the falsifier. Drop the JSON output into pm4py:
 *   import pm4py
 *   log = pm4py.read_ocel2_json('session-ocel.json')
 *   process_tree = pm4py.discover_process_tree_inductive(log)
 *   pm4py.view_process_tree(process_tree)
 *
 * OCEL 2.0 spec: https://www.ocel-standard.org/
 */
import { createClient } from '@supabase/supabase-js';

interface OcelEventRow {
  id: string;
  session_id: string;
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  prev_hash: string | null;
  event_hash: string;
  seq: number;
}

// OCEL 2.0 types ─────────────────────────────────────────────────────────────

interface Ocel2ObjectType {
  name: string;
  attributes: Array<{ name: string; type: string }>;
}

interface Ocel2EventType {
  name: string;
  attributes: Array<{ name: string; type: string }>;
}

interface Ocel2Object {
  id: string;
  type: string;
  attributes: Array<{ name: string; time: string; value: unknown }>;
}

interface Ocel2Event {
  id: string;
  type: string;
  time: string;
  attributes: Array<{ name: string; value: unknown }>;
  relationships: Array<{ objectId: string; qualifier: string }>;
}

interface Ocel2Log {
  objectTypes: Ocel2ObjectType[];
  eventTypes: Ocel2EventType[];
  objects: Ocel2Object[];
  events: Ocel2Event[];
}

function inferObjectType(objectId: string): string {
  if (objectId.startsWith('session')) return 'GameSession';
  if (objectId.startsWith('intent')) return 'Intent';
  if (objectId.startsWith('frame')) return 'Frame';
  return 'GameSession';
}

function toOcel2(rows: OcelEventRow[], sessionId: string): Ocel2Log {
  // Collect unique object IDs encountered across all events
  const objectMap = new Map<string, string>(); // id → type

  // Always include the session itself as a GameSession object
  objectMap.set(sessionId, 'GameSession');

  for (const row of rows) {
    for (const ref of row.object_refs) {
      if (!objectMap.has(ref)) {
        objectMap.set(ref, inferObjectType(ref));
      }
    }
  }

  // Collect unique activity names (→ eventTypes)
  const activitySet = new Set(rows.map(r => r.activity));

  // Build events
  const events: Ocel2Event[] = rows.map((row) => ({
    id: `ev-${row.seq}`,
    type: row.activity,
    time: new Date(row.timestamp_ms).toISOString(),
    attributes: [
      { name: 'seq', value: row.seq },
      { name: 'event_hash', value: row.event_hash },
      ...(row.prev_hash ? [{ name: 'prev_hash', value: row.prev_hash }] : []),
      ...Object.entries(row.attributes).map(([k, v]) => ({ name: k, value: v })),
    ],
    // object_refs: each ref with "session" qualifier for session refs, "object" for others
    relationships: [
      // Always relate the event to its session
      { objectId: sessionId, qualifier: 'session' },
      // Add other refs that are not already the session
      ...row.object_refs
        .filter(ref => ref !== sessionId)
        .map(ref => ({ objectId: ref, qualifier: 'rel' })),
    ],
  }));

  // Build object entries
  const objects: Ocel2Object[] = Array.from(objectMap.entries()).map(([id, type]) => ({
    id,
    type,
    attributes: [],  // No attribute change tracking in current schema
  }));

  // Build objectTypes and eventTypes
  const objectTypeNames = [...new Set(objectMap.values())];
  const objectTypes: Ocel2ObjectType[] = objectTypeNames.map(name => ({
    name,
    attributes: [],
  }));

  const eventTypes: Ocel2EventType[] = [...activitySet].map(name => ({
    name,
    attributes: [
      { name: 'seq', type: 'integer' },
      { name: 'event_hash', type: 'string' },
    ],
  }));

  return { objectTypes, eventTypes, objects, events };
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const sessionId = typeof query.session_id === 'string' ? query.session_id : null;

  if (!sessionId) {
    throw createError({ statusCode: 400, message: 'session_id query param required' });
  }

  const config = useRuntimeConfig(event);
  const supabaseUrl = config.public.supabaseUrl as string;
  const serviceKey = config.supabaseServiceRoleKey as string;

  if (!supabaseUrl || !serviceKey) {
    throw createError({ statusCode: 503, message: 'Supabase not configured' });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const supabase = createClient<any>(supabaseUrl, serviceKey);

  const { data: rows, error } = await supabase
    .from('ocel_events')
    .select('id, session_id, activity, timestamp_ms, object_refs, attributes, prev_hash, event_hash, seq')
    .eq('session_id', sessionId)
    .order('seq', { ascending: true });

  if (error) throw createError({ statusCode: 500, message: error.message });
  if (!rows || rows.length === 0) {
    throw createError({ statusCode: 404, message: `No events found for session ${sessionId}` });
  }

  const ocel2 = toOcel2(rows as OcelEventRow[], sessionId);

  // Serve as downloadable JSON file
  setHeader(event, 'Content-Type', 'application/json');
  setHeader(event, 'Content-Disposition', `attachment; filename="ocel2-${sessionId.slice(0, 8)}.json"`);

  return ocel2;
});
