/**
 * server/utils/otlp-emitter.ts
 *
 * Reusable server-side utility for emitting OTel spans to the configured OTLP
 * HTTP collector. Extracted from ocel-ingest.post.ts so every server route can
 * emit spans without duplicating the OTLP HTTP logic.
 *
 * Key design constraints:
 * - NEVER throws — OTLP failures are logged but must not block receipt creation
 *   or any other server route that calls these utilities.
 * - Reads otlpCollectorUrl from useRuntimeConfig() at call time (not module init)
 *   so hot-reloads and test overrides work correctly.
 * - Uses the native fetch with AbortSignal.timeout(2000) to avoid hanging Nitro
 *   worker threads if the collector is unreachable.
 *
 * Pattern: ~/dashboard.bak/server/api/otel/logs.post.js (OTLP proxy) +
 *          ~/truex/packages/observability/src/fields.ts (attribute shape)
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** OTLP attribute value — only the scalar forms we need. */
export interface OtlpAttributeValue {
  stringValue?: string;
  intValue?: string;
}

/** Single OTLP key-value attribute. */
export interface OtlpAttribute {
  key: string;
  value: OtlpAttributeValue;
}

/** Minimal OTLP span shape accepted by the /v1/traces endpoint. */
export interface OtlpSpan {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  name: string;
  startTimeUnixNano: string;
  endTimeUnixNano: string;
  attributes: OtlpAttribute[];
  status: { code: number };
}

/**
 * Caller-facing span descriptor — a simplified view that the utility converts
 * to the OTLP wire format.
 *
 * The OCEL linkage fields (receipt_id, receipt_hash, ocel_event_count, chain_tip)
 * close the correlation gap between OTel traces and OCEL sessions: a process mining
 * tool can join on receipt_id to link every OTel trace back to its OCEL event log.
 */
export interface SpanDescriptor {
  /** Human-readable activity name — becomes "ocel.<activity>" as the span name. */
  activity: string;
  /** Unix milliseconds for the start of the span. */
  timestamp_ms: number;
  /** Session identifier emitted as game.session_id attribute. */
  session_id: string;
  /** Additional key-value attributes merged into the OTLP span. */
  attributes?: Record<string, string | number | boolean>;
  // ── OCEL linkage (gap-12) ──────────────────────────────────────────────────
  /** Receipt UUID that finalised this session — enables OTel ↔ OCEL join. */
  receipt_id?: string;
  /** BLAKE3 hash of the receipt payload — tamper detection for cross-stream joins. */
  receipt_hash?: string;
  /** Total OCEL events in this session at the time the span was emitted. */
  ocel_event_count?: number;
  /** BLAKE3 hash of the last OCEL event — proves chain tip at this point in time. */
  chain_tip?: string;
}

/** Return shape from both emit variants. */
export interface EmitResult {
  /** The trace ID generated for this batch, or null if the collector was unreachable. */
  traceId: string | null;
  /** True when the collector accepted the payload (2xx response). */
  accepted: boolean;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/** Generate a cryptographically random hex string of `bytes` length. */
function hexId(bytes: number): string {
  const arr = new Uint8Array(bytes);
  crypto.getRandomValues(arr);
  return Array.from(arr)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

/** Convert a scalar value to an OTLP attribute object. */
function toOtlpAttr(key: string, value: string | number | boolean): OtlpAttribute {
  return { key, value: { stringValue: String(value) } };
}

/** Build the OTLP ExportTraceServiceRequest envelope from a span array. */
function buildOtlpPayload(spans: OtlpSpan[]): Record<string, unknown> {
  return {
    resourceSpans: [
      {
        resource: {
          attributes: [
            toOtlpAttr('service.name', 'rocket-craft-nuxt-shell'),
            toOtlpAttr('service.version', '1.0.0'),
          ],
        },
        scopeSpans: [
          {
            scope: { name: 'rocket-craft.ocel', version: '1.0.0' },
            spans,
          },
        ],
      },
    ],
  };
}

/** Resolve the OTLP collector URL from runtimeConfig, falling back to localhost. */
function resolveCollectorUrl(): string {
  try {
    const config = useRuntimeConfig();
    return (config.otlpCollectorUrl as string | undefined) ?? 'http://localhost:4318';
  } catch {
    // useRuntimeConfig() can throw outside a Nitro request context (e.g. unit tests)
    return 'http://localhost:4318';
  }
}

/** POST the OTLP payload to the collector; returns traceId on success, null on any failure. */
async function postToCollector(
  traceId: string,
  spans: OtlpSpan[],
  collectorUrl: string,
): Promise<EmitResult> {
  const payload = buildOtlpPayload(spans);
  try {
    const res = await fetch(`${collectorUrl}/v1/traces`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(2000),
    });
    if (!res.ok) {
      console.warn(
        `[otlp-emitter] collector returned ${res.status} for trace ${traceId} — continuing`,
      );
      return { traceId, accepted: false };
    }
    return { traceId, accepted: true };
  } catch (err) {
    // Collector unreachable or timed out — log and continue; OCEL in Supabase is
    // the source of truth; OTel is supplemental observability only.
    const message = err instanceof Error ? err.message : String(err);
    console.warn(`[otlp-emitter] OTLP emit failed (non-fatal): ${message}`);
    return { traceId: null, accepted: false };
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Emit a single OTel span for one game session activity.
 *
 * @param sessionId  - Game session identifier (game.session_id attribute).
 * @param activity   - Activity name; the span name will be `ocel.<activity>`.
 * @param attributes - Additional attributes merged into the span.
 * @returns EmitResult with the generated traceId (null if collector unreachable).
 *
 * @example
 * const { traceId } = await emitOtelSpan('sess-abc', 'ReceiptCreated', { verdict: 'PASS' });
 */
export async function emitOtelSpan(
  sessionId: string,
  activity: string,
  attributes: Record<string, string | number | boolean> = {},
): Promise<EmitResult> {
  const collectorUrl = resolveCollectorUrl();
  const traceId = hexId(16);
  const spanId = hexId(8);
  const startNs = BigInt(Date.now()) * 1_000_000n;

  const builtInAttrs: OtlpAttribute[] = [
    toOtlpAttr('game.session_id', sessionId),
    toOtlpAttr('game.activity', activity),
    toOtlpAttr('game.timestamp_ms', Date.now()),
  ];

  const extraAttrs: OtlpAttribute[] = Object.entries(attributes).map(([k, v]) =>
    toOtlpAttr(k, v),
  );

  const span: OtlpSpan = {
    traceId,
    spanId,
    name: `ocel.${activity}`,
    startTimeUnixNano: startNs.toString(),
    endTimeUnixNano: (startNs + 1_000_000n).toString(),
    attributes: [...builtInAttrs, ...extraAttrs],
    status: { code: 1 }, // STATUS_CODE_OK
  };

  return postToCollector(traceId, [span], collectorUrl);
}

/**
 * Emit multiple OTel spans in a single OTLP ExportTraceServiceRequest envelope.
 * All spans share the same traceId; each gets its own unique spanId with the
 * first span acting as parent for the rest.
 *
 * This is the preferred path for the ocel-ingest endpoint which processes a
 * batch of OCEL events at once.
 *
 * @param descriptors - Array of span descriptors to emit as a batch.
 * @returns EmitResult with the shared traceId (null if collector unreachable).
 *
 * @example
 * const result = await emitOtelSpans([
 *   { session_id: 'sess-abc', activity: 'GameSessionStarted', timestamp_ms: Date.now() },
 *   { session_id: 'sess-abc', activity: 'FrameRendered',       timestamp_ms: Date.now() + 16 },
 * ]);
 */
export async function emitOtelSpans(descriptors: SpanDescriptor[]): Promise<EmitResult> {
  if (descriptors.length === 0) {
    return { traceId: null, accepted: false };
  }

  const collectorUrl = resolveCollectorUrl();
  const traceId = hexId(16);
  const parentSpanId = hexId(8);

  const spans: OtlpSpan[] = descriptors.map(desc => {
    const startNs = BigInt(desc.timestamp_ms) * 1_000_000n;
    const builtIn: OtlpAttribute[] = [
      toOtlpAttr('game.session_id', desc.session_id),
      toOtlpAttr('game.activity', desc.activity),
      toOtlpAttr('game.timestamp_ms', desc.timestamp_ms),
    ];
    // OCEL linkage attributes — present only when the span is tied to a receipt/chain
    const ocelLink: OtlpAttribute[] = [];
    if (desc.receipt_id)        ocelLink.push(toOtlpAttr('ocel.receipt_id', desc.receipt_id));
    if (desc.receipt_hash)      ocelLink.push(toOtlpAttr('ocel.receipt_hash', desc.receipt_hash));
    if (desc.ocel_event_count != null) ocelLink.push(toOtlpAttr('ocel.event_count', desc.ocel_event_count));
    if (desc.chain_tip)         ocelLink.push(toOtlpAttr('ocel.chain_tip', desc.chain_tip));
    const extra: OtlpAttribute[] = Object.entries(desc.attributes ?? {}).map(([k, v]) =>
      toOtlpAttr(k, v),
    );
    return {
      traceId,
      spanId: hexId(8),
      parentSpanId,
      name: `ocel.${desc.activity}`,
      startTimeUnixNano: startNs.toString(),
      endTimeUnixNano: (startNs + 1_000_000n).toString(),
      attributes: [...builtIn, ...ocelLink, ...extra],
      status: { code: 1 }, // STATUS_CODE_OK
    };
  });

  return postToCollector(traceId, spans, collectorUrl);
}
