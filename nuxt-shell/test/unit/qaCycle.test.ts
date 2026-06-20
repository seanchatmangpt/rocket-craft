// @vitest-environment happy-dom
/**
 * qaCycle.test.ts
 *
 * Tests for the autonomous QA cycle pure functions:
 *   classifyOverall — maps check results to HEALTHY | DEGRADED | CRITICAL
 *   buildCycleResult — maps raw query data to QaCheckResult[]
 *
 * Van der Aalst doctrine: QA logic that cannot be tested without a live DB
 * cannot be trusted. These tests verify the invariant-checking layer itself.
 */

import { describe, it, expect } from 'vitest';
import { classifyOverall, buildCycleResult } from '../../server/utils/qaCycle';

// ── classifyOverall ──────────────────────────────────────────────────────────

describe('classifyOverall — all pass', () => {
  it('all checks pass → HEALTHY', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: true },
      { check: 'LIFECYCLE_COMPLETE', passed: true },
      { check: 'NO_SYNTHETIC_SOURCE', passed: true },
      { check: 'MERKLE_ROOT_PRESENT', passed: true },
    ];
    expect(classifyOverall(results)).toBe('HEALTHY');
  });

  it('single passing check → HEALTHY', () => {
    expect(classifyOverall([{ check: 'RECEIPT_CHAIN_INTACT', passed: true }])).toBe('HEALTHY');
  });

  it('empty results → HEALTHY (no failures)', () => {
    expect(classifyOverall([])).toBe('HEALTHY');
  });
});

describe('classifyOverall — DEGRADED', () => {
  it('LIFECYCLE_COMPLETE fails → DEGRADED', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: true },
      { check: 'LIFECYCLE_COMPLETE', passed: false },
      { check: 'NO_SYNTHETIC_SOURCE', passed: true },
      { check: 'MERKLE_ROOT_PRESENT', passed: true },
    ];
    expect(classifyOverall(results)).toBe('DEGRADED');
  });

  it('NO_SYNTHETIC_SOURCE fails → DEGRADED', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: true },
      { check: 'NO_SYNTHETIC_SOURCE', passed: false },
    ];
    expect(classifyOverall(results)).toBe('DEGRADED');
  });

  it('MERKLE_ROOT_PRESENT fails → DEGRADED', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: true },
      { check: 'MERKLE_ROOT_PRESENT', passed: false },
    ];
    expect(classifyOverall(results)).toBe('DEGRADED');
  });

  it('multiple non-chain failures → DEGRADED (not CRITICAL)', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: true },
      { check: 'LIFECYCLE_COMPLETE', passed: false },
      { check: 'MERKLE_ROOT_PRESENT', passed: false },
    ];
    expect(classifyOverall(results)).toBe('DEGRADED');
  });
});

describe('classifyOverall — CRITICAL', () => {
  it('RECEIPT_CHAIN_INTACT fails → CRITICAL', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: false },
      { check: 'LIFECYCLE_COMPLETE', passed: true },
      { check: 'NO_SYNTHETIC_SOURCE', passed: true },
      { check: 'MERKLE_ROOT_PRESENT', passed: true },
    ];
    expect(classifyOverall(results)).toBe('CRITICAL');
  });

  it('chain fail + other fail → CRITICAL (chain beats other failures)', () => {
    const results = [
      { check: 'RECEIPT_CHAIN_INTACT', passed: false },
      { check: 'LIFECYCLE_COMPLETE', passed: false },
      { check: 'NO_SYNTHETIC_SOURCE', passed: false },
    ];
    expect(classifyOverall(results)).toBe('CRITICAL');
  });

  it('chain fail alone → CRITICAL', () => {
    expect(classifyOverall([{ check: 'RECEIPT_CHAIN_INTACT', passed: false }])).toBe('CRITICAL');
  });
});

// ── buildCycleResult ─────────────────────────────────────────────────────────

describe('buildCycleResult — HEALTHY inputs', () => {
  const VALID_HASH = 'a'.repeat(64);

  it('all valid inputs → all checks pass', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      engineSource: 'rocket_cli',
      eventHashes: [VALID_HASH],
      merkleRoot: VALID_HASH,
    });
    expect(results).toHaveLength(4);
    expect(results.every(r => r.passed)).toBe(true);
  });

  it('extra activities beyond required → still LIFECYCLE_COMPLETE passes', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted', 'ExtraActivity'],
      engineSource: 'rocket_cli',
      eventHashes: [VALID_HASH],
      merkleRoot: VALID_HASH,
    });
    const lc = results.find(r => r.check === 'LIFECYCLE_COMPLETE');
    expect(lc?.passed).toBe(true);
  });
});

describe('buildCycleResult — failing inputs', () => {
  const VALID_HASH = 'b'.repeat(64);

  it('chainOk=false → RECEIPT_CHAIN_INTACT fails', () => {
    const results = buildCycleResult({
      chainOk: false,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      engineSource: 'rocket_cli',
      eventHashes: [VALID_HASH],
      merkleRoot: VALID_HASH,
    });
    const check = results.find(r => r.check === 'RECEIPT_CHAIN_INTACT');
    expect(check?.passed).toBe(false);
  });

  it('missing FrameRendered → LIFECYCLE_COMPLETE fails with missing listed', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'InputAdmitted'],
      engineSource: 'rocket_cli',
      eventHashes: [VALID_HASH],
      merkleRoot: VALID_HASH,
    });
    const lc = results.find(r => r.check === 'LIFECYCLE_COMPLETE');
    expect(lc?.passed).toBe(false);
    expect((lc?.evidence as { missing: string[] }).missing).toContain('FrameRendered');
  });

  it('engine_source=synthetic → NO_SYNTHETIC_SOURCE fails', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      engineSource: 'synthetic',
      eventHashes: [VALID_HASH],
      merkleRoot: VALID_HASH,
    });
    const ns = results.find(r => r.check === 'NO_SYNTHETIC_SOURCE');
    expect(ns?.passed).toBe(false);
  });

  it('empty eventHashes → MERKLE_ROOT_PRESENT fails', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      engineSource: 'rocket_cli',
      eventHashes: [],
      merkleRoot: null,
    });
    const mp = results.find(r => r.check === 'MERKLE_ROOT_PRESENT');
    expect(mp?.passed).toBe(false);
  });

  it('null merkleRoot → MERKLE_ROOT_PRESENT fails', () => {
    const results = buildCycleResult({
      chainOk: true,
      activities: ['GameSessionStarted', 'FrameRendered', 'InputAdmitted'],
      engineSource: 'rocket_cli',
      eventHashes: [VALID_HASH],
      merkleRoot: null,
    });
    const mp = results.find(r => r.check === 'MERKLE_ROOT_PRESENT');
    expect(mp?.passed).toBe(false);
  });
});
