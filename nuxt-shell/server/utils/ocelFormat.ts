/**
 * server/utils/ocelFormat.ts
 *
 * Pure OCEL 2.0 format builder — extracted from ocel-export.get.ts.
 * Testable without Nitro or Supabase.
 *
 * Spec: https://www.ocel-standard.org/
 */

export interface OcelEventRow {
  id?: string
  session_id?: string
  activity: string
  timestamp_ms: number
  object_refs: string[]
  attributes: Record<string, unknown>
  prev_hash: string | null
  event_hash: string
  seq: number
}

export interface Ocel2ObjectType {
  name: string
  attributes: Array<{ name: string; type: string }>
}

export interface Ocel2EventType {
  name: string
  attributes: Array<{ name: string; type: string }>
}

export interface Ocel2Object {
  id: string
  type: string
  attributes: Array<{ name: string; time: string; value: unknown }>
}

export interface Ocel2Event {
  id: string
  type: string
  time: string
  attributes: Array<{ name: string; value: unknown }>
  relationships: Array<{ objectId: string; qualifier: string }>
}

export interface Ocel2Log {
  objectTypes: Ocel2ObjectType[]
  eventTypes: Ocel2EventType[]
  objects: Ocel2Object[]
  events: Ocel2Event[]
}

export type WasmCrossCheckVerdict = 'MATCH' | 'MISMATCH' | 'COOK_ONLY' | 'GAME_ONLY' | 'NO_DATA'

export interface CrossCheckReceipt {
  id: string
  milestone: string
  verdict: string
  engine_source: string
  output_hash: string
  proven_at: string
  session_id: string | null
}

export function inferObjectType(objectId: string): string {
  if (objectId.startsWith('session')) return 'GameSession'
  if (objectId.startsWith('intent')) return 'Intent'
  if (objectId.startsWith('frame')) return 'Frame'
  if (objectId.startsWith('cook')) return 'CookArtifact'
  return 'GameSession'
}

export function toOcel2(rows: OcelEventRow[], sessionId: string): Ocel2Log {
  const objectMap = new Map<string, string>()
  objectMap.set(sessionId, 'GameSession')

  for (const row of rows) {
    for (const ref of row.object_refs) {
      if (!objectMap.has(ref)) {
        objectMap.set(ref, inferObjectType(ref))
      }
    }
  }

  const activitySet = new Set(rows.map(r => r.activity))

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
    relationships: [
      { objectId: sessionId, qualifier: 'session' },
      ...row.object_refs
        .filter(ref => ref !== sessionId)
        .map(ref => ({ objectId: ref, qualifier: 'rel' })),
    ],
  }))

  const objects: Ocel2Object[] = Array.from(objectMap.entries()).map(([id, type]) => ({
    id,
    type,
    attributes: [],
  }))

  const objectTypeNames = [...new Set(objectMap.values())]
  const objectTypes: Ocel2ObjectType[] = objectTypeNames.map(name => ({
    name,
    attributes: [],
  }))

  const eventTypes: Ocel2EventType[] = [...activitySet].map(name => ({
    name,
    attributes: [
      { name: 'seq', type: 'integer' },
      { name: 'event_hash', type: 'string' },
    ],
  }))

  return { objectTypes, eventTypes, objects, events }
}

/** Classify receipts sharing an output_hash into a cross-check verdict. */
export function classifyWasmCrossCheck(receipts: CrossCheckReceipt[]): {
  cook_receipts: number
  game_receipts: number
  total: number
  verdict: WasmCrossCheckVerdict
} {
  const isCook = (r: CrossCheckReceipt) => /cook|html5|verify|package/i.test(r.milestone)
  const isGame = (r: CrossCheckReceipt) => /game|session|proof|tps|dflss/i.test(r.milestone)

  const cookCount = receipts.filter(isCook).length
  const gameCount = receipts.filter(isGame).length

  let verdict: WasmCrossCheckVerdict
  if (receipts.length === 0) {
    verdict = 'NO_DATA'
  } else if (cookCount > 0 && gameCount > 0) {
    verdict = 'MATCH'
  } else if (cookCount > 0) {
    verdict = 'COOK_ONLY'
  } else if (gameCount > 0) {
    verdict = 'GAME_ONLY'
  } else {
    verdict = 'MATCH'
  }

  return { cook_receipts: cookCount, game_receipts: gameCount, total: receipts.length, verdict }
}
