import { writeFileSync } from 'node:fs'
import type { VerifierReport, GateResult, RocketResidual, JidokaEvent, RocketStatus } from './types.js'

export function createVerifierReport(milestone: string, status: RocketStatus): VerifierReport {
  return {
    milestone,
    status,
    scoped_status: 'UNKNOWN',
    gates: [],
    residuals: [],
    jidoka_events: [],
    emitted_at: new Date().toISOString(),
  }
}

export function appendGateResult(report: VerifierReport, gate: GateResult): void {
  report.gates.push(gate)
}

export function appendResidual(report: VerifierReport, residual: RocketResidual): void {
  report.residuals.push(residual)
}

export function appendJidokaEvent(report: VerifierReport, event: JidokaEvent): void {
  report.jidoka_events.push(event)
}

export function finalizeReport(report: VerifierReport, scoped_status: string): VerifierReport {
  return { ...report, scoped_status, emitted_at: new Date().toISOString() }
}

export function writeJsonReport(report: VerifierReport, path: string): void {
  writeFileSync(path, JSON.stringify(report, null, 2), 'utf8')
}

export function writeMarkdownReport(report: VerifierReport, path: string): void {
  const lines: string[] = [
    `# Verifier Report — ${report.milestone}`,
    '',
    `**Status:** ${report.status}  `,
    `**Scoped status:** ${report.scoped_status}  `,
    `**Emitted:** ${report.emitted_at}`,
    '',
    '## Gates',
    '',
    ...report.gates.map(g =>
      `- **${g.name}**: ${g.status}${g.detail ? ' — ' + g.detail : ''}`
    ),
    '',
    '## Residuals',
    '',
    report.residuals.length === 0
      ? '_None_'
      : report.residuals.map(r =>
          `- \`${r.code}\` [${r.severity}] ${r.message}${r.repair_candidate ? ' → ' + r.repair_candidate : ''}`
        ).join('\n'),
    '',
    '## Jidoka Events',
    '',
    report.jidoka_events.length === 0
      ? '_None_'
      : report.jidoka_events.map(j =>
          `- **${j.defect_class}** on \`${j.surface}\`: ${j.observed_failure}`
        ).join('\n'),
  ]
  writeFileSync(path, lines.join('\n'), 'utf8')
}
