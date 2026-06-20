/**
 * POST /api/otel/spans
 *
 * Accepts OTLP-formatted trace spans from the UE4 cook pipeline (or any emitter)
 * and converts them to ocel_events rows in Supabase — closing the loop between
 * OpenTelemetry observability and OCEL 2.0 process mining.
 *
 * Pattern: ~/dashboard.bak/server/api/otel/logs.post.js
 * Extended: maps OTLP resourceSpans → OcelEventRow via activity heuristic.
 *
 * Body (OTLP JSON): { resourceSpans: [{ scopeSpans: [{ spans: [...] }] }] }
 * Query params:
 *   session_id  — associate spans with an existing game session (optional)
 *   object_ref  — OCEL object ref string (default: "cook:<session_id|unknown>")
 *
 * Each span becomes one ocel_events row:
 *   activity   = span.name (canonicalized to UpperCamelCase activity)
 *   ts_ms      = span.startTimeUnixNano / 1_000_000
 *   attributes = span.attributes as JSON
 *
 * The BLAKE3 hash chain is re-computed server-side using ChainedOcelEmitter
 * logic (same formula used by rocket-sdk push_to_supabase).
 *
 * Returns: { ingested: number, session_id: string, chain_tip: string | null }
 */

import { createClient } from '@supabase/supabase-js'
import { blake3 } from '@noble/hashes/blake3.js'
import { bytesToHex } from '@noble/hashes/utils'

// OTLP span structure (only the fields we use)
interface OtlpSpan {
  name: string
  startTimeUnixNano: string // uint64 as string in OTLP JSON
  attributes?: Array<{ key: string; value: { stringValue?: string; intValue?: string; boolValue?: boolean } }>
  traceId?: string
  spanId?: string
}

interface OtlpScopeSpans { spans: OtlpSpan[] }
interface OtlpResourceSpans { scopeSpans: OtlpScopeSpans[] }
interface OtlpBody { resourceSpans: OtlpResourceSpans[] }

/** Flatten OTLP body to a list of spans ordered by startTimeUnixNano */
function flattenSpans(body: OtlpBody): OtlpSpan[] {
  const spans: OtlpSpan[] = []
  for (const rs of body.resourceSpans ?? []) {
    for (const ss of rs.scopeSpans ?? []) {
      spans.push(...(ss.spans ?? []))
    }
  }
  spans.sort((a, b) => {
    const ta = BigInt(a.startTimeUnixNano ?? '0')
    const tb = BigInt(b.startTimeUnixNano ?? '0')
    return ta < tb ? -1 : ta > tb ? 1 : 0
  })
  return spans
}

/** Convert OTLP span attributes array to a plain object */
function attrsToObject(attrs: OtlpSpan['attributes'] = []): Record<string, unknown> {
  const out: Record<string, unknown> = {}
  for (const { key, value } of attrs) {
    if (value.stringValue !== undefined) out[key] = value.stringValue
    else if (value.intValue !== undefined) out[key] = Number(value.intValue)
    else if (value.boolValue !== undefined) out[key] = value.boolValue
  }
  return out
}

/** Compute a BLAKE3 event hash matching ChainedOcelEmitter formula */
function computeEventHash(params: {
  session_id: string | null
  object_ref: string
  activity: string
  ts_ms: number
  seq: number
  prev_hash: string | null
  attributes: Record<string, unknown>
}): string {
  const payload = {
    activity: params.activity,
    attributes: params.attributes,
    object_ref: params.object_ref,
    prev_hash: params.prev_hash,
    seq: params.seq,
    session_id: params.session_id,
    ts_ms: params.ts_ms,
  }
  // canonical JSON: keys sorted alphabetically (same as Rust canonical_json)
  const canonical = JSON.stringify(payload, Object.keys(payload).sort())
  return bytesToHex(blake3(new TextEncoder().encode(canonical)))
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const sessionId = (query.session_id as string) || null
  const body = (await readBody(event)) as OtlpBody

  if (!body?.resourceSpans?.length) {
    throw createError({ statusCode: 400, message: 'resourceSpans array is required' })
  }

  const supabaseUrl = process.env.SUPABASE_URL ?? 'http://localhost:54321'
  const serviceKey = process.env.SUPABASE_SERVICE_ROLE_KEY
  if (!serviceKey) {
    throw createError({ statusCode: 503, message: 'SUPABASE_SERVICE_ROLE_KEY not configured' })
  }

  const objectRef = (query.object_ref as string) || `cook:${sessionId ?? 'unknown'}`
  const spans = flattenSpans(body)

  if (spans.length === 0) {
    return { ingested: 0, session_id: sessionId, chain_tip: null }
  }

  // Fetch the current chain tip for this session (to continue an existing chain)
  const supabase = createClient(supabaseUrl, serviceKey)
  let prevHash: string | null = null
  let startSeq = 0

  if (sessionId) {
    const { data: lastEvent } = await supabase
      .from('ocel_events')
      .select('event_hash, seq')
      .eq('session_id', sessionId)
      .order('seq', { ascending: false })
      .limit(1)
      .maybeSingle()

    if (lastEvent) {
      prevHash = lastEvent.event_hash as string
      startSeq = (lastEvent.seq as number) + 1
    }
  }

  // Build rows
  const rows = spans.map((span, i) => {
    const tsMs = Number(BigInt(span.startTimeUnixNano ?? '0') / 1_000_000n)
    const activity = span.name
    const attributes = attrsToObject(span.attributes)
    const seq = startSeq + i

    const hash = computeEventHash({
      session_id: sessionId,
      object_ref: objectRef,
      activity,
      ts_ms: tsMs,
      seq,
      prev_hash: prevHash,
      attributes,
    })

    const row = {
      session_id: sessionId,
      object_ref: objectRef,
      activity,
      ts_ms: tsMs,
      seq,
      prev_hash: prevHash,
      event_hash: hash,
      attributes,
    }
    prevHash = hash
    return row
  })

  const { error: insertErr } = await supabase.from('ocel_events').insert(rows)
  if (insertErr) {
    throw createError({ statusCode: 500, message: `Failed to insert ocel_events: ${insertErr.message}` })
  }

  return {
    ingested: rows.length,
    session_id: sessionId,
    object_ref: objectRef,
    chain_tip: prevHash,
    first_seq: startSeq,
    last_seq: startSeq + rows.length - 1,
  }
})
