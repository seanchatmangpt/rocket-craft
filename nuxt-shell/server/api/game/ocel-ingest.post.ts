/**
 * POST /api/game/ocel-ingest
 *
 * Server-side OCEL event ingest endpoint.
 * Pattern: ~/dashboard.bak/server/api/otel/logs.post.js (OTLP proxy)
 * Extended: each OCEL batch also emits an OTLP span to the local collector.
 *
 * Body: { session_id: string, events: OcelEventBatch[] }
 * Returns: { ingested: number, trace_id: string | null, session_id: string }
 *
 * OTel attributes per event (from ~/truex/packages/observability/src/fields.ts pattern):
 *   game.session_id, game.activity, game.timestamp_ms,
 *   ocel.event_hash, ocel.seq, ocel.object_refs[]
 *
 * OTLP emission is handled by server/utils/otlp-emitter — failures are logged
 * but never propagate; Supabase OCEL storage is the source of truth.
 */

import { emitOtelSpans, type SpanDescriptor } from '~/server/utils/otlp-emitter';

interface OcelEventBatch {
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  event_hash: string;
  seq: number;
  session_id: string;
}

export default defineEventHandler(async (event) => {
  const body = await readBody<{ session_id: string; events: OcelEventBatch[] }>(event);

  if (!body?.session_id || !Array.isArray(body.events) || body.events.length === 0) {
    throw createError({ statusCode: 400, statusMessage: 'session_id and non-empty events[] required' });
  }

  // Map OCEL events to SpanDescriptors for the batch emitter
  const descriptors: SpanDescriptor[] = body.events.map(evt => ({
    session_id: evt.session_id,
    activity: evt.activity,
    timestamp_ms: evt.timestamp_ms,
    attributes: {
      'ocel.seq': evt.seq,
      'ocel.event_hash': evt.event_hash,
      'ocel.object_refs': evt.object_refs.join(','),
    },
  }));

  const { traceId } = await emitOtelSpans(descriptors);

  return {
    ingested: body.events.length,
    trace_id: traceId,
    session_id: body.session_id,
  };
});
