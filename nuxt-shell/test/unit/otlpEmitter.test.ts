// @vitest-environment node
// Real Node fetch + http server — happy-dom's Window.fetch does not deliver the
// local collector's response, aborting the round-trip.
import { describe, it, expect } from 'vitest';
import { createServer, type Server } from 'node:http';
import { buildOtlpPayload, toOtlpAttr, emitOtelSpan, type OtlpSpan } from '../../server/utils/otlp-emitter';

/**
 * OTLP emitter contract tests.
 *
 * A malformed OTLP ExportTraceServiceRequest envelope is silently rejected by a
 * collector — traces vanish with no other signal. These tests pin the envelope
 * structure AND prove (via a mock collector) that emitOtelSpan POSTs a well-formed
 * payload carrying the session linkage, with the returned traceId matching.
 */

describe('otlp-emitter payload contract', () => {
  it('toOtlpAttr wraps scalars as OTLP AnyValue stringValue', () => {
    expect(toOtlpAttr('k', 'v')).toEqual({ key: 'k', value: { stringValue: 'v' } });
    expect(toOtlpAttr('n', 42)).toEqual({ key: 'n', value: { stringValue: '42' } });
    expect(toOtlpAttr('b', true)).toEqual({ key: 'b', value: { stringValue: 'true' } });
  });

  it('buildOtlpPayload produces a spec-shaped ExportTraceServiceRequest', () => {
    const span: OtlpSpan = {
      traceId: 'a'.repeat(32),
      spanId: 'b'.repeat(16),
      name: 'ocel.GameSessionStarted',
      startTimeUnixNano: '1',
      endTimeUnixNano: '2',
      attributes: [toOtlpAttr('game.session_id', 'sess-1')],
      status: { code: 1 },
    };
    const payload = buildOtlpPayload([span]) as any;

    expect(Array.isArray(payload.resourceSpans)).toBe(true);
    const rs = payload.resourceSpans[0];
    // resource must carry service.name (collectors group/route on this)
    const svcName = rs.resource.attributes.find((a: any) => a.key === 'service.name');
    expect(svcName?.value.stringValue).toBe('rocket-craft-nuxt-shell');
    // scopeSpans → spans must contain our span unchanged
    expect(rs.scopeSpans[0].spans).toHaveLength(1);
    expect(rs.scopeSpans[0].spans[0].name).toBe('ocel.GameSessionStarted');
    expect(rs.scopeSpans[0].spans[0].traceId).toBe('a'.repeat(32));
  });
});

describe('emitOtelSpan → mock collector', () => {
  it('POSTs a well-formed payload with session linkage; returns matching traceId', async () => {
    let captured: any = null;
    const server: Server = createServer((req, res) => {
      let raw = '';
      req.on('data', (c) => { raw += c; });
      req.on('end', () => {
        captured = { url: req.url, body: JSON.parse(raw) };
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
      });
    });
    await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
    const port = (server.address() as { port: number }).port;
    const prev = process.env.OTLP_COLLECTOR_URL;
    process.env.OTLP_COLLECTOR_URL = `http://127.0.0.1:${port}`;

    try {
      const { traceId, accepted } = await emitOtelSpan('sess-xyz', 'ReceiptCreated', { verdict: 'PASS' });
      expect(accepted).toBe(true);
      expect(traceId).toMatch(/^[0-9a-f]{32}$/);

      // Collector received it at the OTLP traces path
      expect(captured.url).toBe('/v1/traces');
      const span = captured.body.resourceSpans[0].scopeSpans[0].spans[0];
      expect(span.traceId).toBe(traceId);
      expect(span.name).toBe('ocel.ReceiptCreated');
      const attrs = Object.fromEntries(span.attributes.map((a: any) => [a.key, a.value.stringValue]));
      expect(attrs['game.session_id']).toBe('sess-xyz');
      expect(attrs['verdict']).toBe('PASS');
    } finally {
      if (prev === undefined) delete process.env.OTLP_COLLECTOR_URL;
      else process.env.OTLP_COLLECTOR_URL = prev;
      await new Promise<void>((resolve) => server.close(() => resolve()));
    }
  });

  it('returns traceId=null, accepted=false when the collector is unreachable (non-fatal)', async () => {
    const prev = process.env.OTLP_COLLECTOR_URL;
    // Unused port — connection refused.
    process.env.OTLP_COLLECTOR_URL = 'http://127.0.0.1:1';
    try {
      const { traceId, accepted } = await emitOtelSpan('sess-down', 'X');
      expect(accepted).toBe(false);
      expect(traceId).toBeNull();
    } finally {
      if (prev === undefined) delete process.env.OTLP_COLLECTOR_URL;
      else process.env.OTLP_COLLECTOR_URL = prev;
    }
  });
});
