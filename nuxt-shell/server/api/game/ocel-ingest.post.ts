/**
 * POST /api/game/ocel-ingest
 *
 * Server-side OCEL event ingest endpoint.
 * Pattern: ~/dashboard.bak/server/api/otel/logs.post.js (OTLP proxy)
 * Extended: each OCEL batch also emits an OTLP span to the local collector.
 *
 * Body: { session_id: string, events: OcelEventBatch[] }
 * Returns: { ingested: number, otel_span_id: string | null }
 *
 * OTel attributes per event (from ~/truex/packages/observability/src/fields.ts pattern):
 *   game.session_id, game.activity, game.timestamp_ms, game.engine_source,
 *   ocel.event_hash, ocel.seq, ocel.object_refs[]
 */

interface OcelEventBatch {
  activity: string;
  timestamp_ms: number;
  object_refs: string[];
  attributes: Record<string, unknown>;
  event_hash: string;
  seq: number;
  session_id: string;
}

interface OtelSpan {
  traceId: string;
  spanId: string;
  name: string;
  startTimeUnixNano: string;
  endTimeUnixNano: string;
  attributes: Array<{ key: string; value: { stringValue?: string; intValue?: string } }>;
  status: { code: number };
}

function hexId(bytes: number): string {
  const arr = new Uint8Array(bytes);
  crypto.getRandomValues(arr);
  return Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
}

function toOtelAttr(key: string, value: string | number): OtelSpan['attributes'][0] {
  return typeof value === 'number'
    ? { key, value: { stringValue: String(value) } }
    : { key, value: { stringValue: value } };
}

async function emitOtelSpans(events: OcelEventBatch[], collectorUrl: string): Promise<string | null> {
  const traceId = hexId(16);
  const parentSpanId = hexId(8);

  const spans: OtelSpan[] = events.map(evt => {
    const startNs = BigInt(evt.timestamp_ms) * 1_000_000n;
    return {
      traceId,
      spanId: hexId(8),
      parentSpanId,
      name: `ocel.${evt.activity}`,
      startTimeUnixNano: startNs.toString(),
      endTimeUnixNano: (startNs + 1_000_000n).toString(),
      attributes: [
        toOtelAttr('game.session_id', evt.session_id),
        toOtelAttr('game.activity', evt.activity),
        toOtelAttr('game.timestamp_ms', evt.timestamp_ms),
        toOtelAttr('ocel.seq', evt.seq),
        toOtelAttr('ocel.event_hash', evt.event_hash),
        toOtelAttr('ocel.object_refs', evt.object_refs.join(',')),
      ],
      status: { code: 1 }, // OK
    };
  });

  const otlpPayload = {
    resourceSpans: [{
      resource: {
        attributes: [
          toOtelAttr('service.name', 'rocket-craft-nuxt-shell'),
          toOtelAttr('service.version', '1.0.0'),
        ],
      },
      scopeSpans: [{
        scope: { name: 'rocket-craft.ocel', version: '1.0.0' },
        spans,
      }],
    }],
  };

  try {
    const res = await fetch(`${collectorUrl}/v1/traces`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(otlpPayload),
      signal: AbortSignal.timeout(2000),
    });
    return res.ok ? traceId : null;
  } catch {
    return null; // collector unreachable — non-fatal, OCEL in Supabase is the source of truth
  }
}

export default defineEventHandler(async (event) => {
  const body = await readBody<{ session_id: string; events: OcelEventBatch[] }>(event);

  if (!body?.session_id || !Array.isArray(body.events) || body.events.length === 0) {
    throw createError({ statusCode: 400, statusMessage: 'session_id and non-empty events[] required' });
  }

  const config = useRuntimeConfig();
  const otlpUrl = (config.otlpCollectorUrl as string | undefined) ?? 'http://localhost:4318';

  const traceId = await emitOtelSpans(body.events, otlpUrl);

  return {
    ingested: body.events.length,
    trace_id: traceId,
    session_id: body.session_id,
  };
});
