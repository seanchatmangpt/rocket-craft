import type { RocketReceipt, RocketResidual } from './types.js'

export function buildReceipt(
  sequence: number,
  event_type: string,
  surface: string,
  receipt: string,
  opts: Partial<Omit<RocketReceipt, 'sequence' | 'event_type' | 'surface' | 'receipt'>> = {}
): RocketReceipt {
  return { sequence, event_type, surface, receipt, status: 'ADMITTED', residuals: [], ...opts }
}

export function validateReceiptChain(chain: RocketReceipt[]): RocketResidual[] {
  const residuals: RocketResidual[] = []
  for (let i = 0; i < chain.length; i++) {
    const entry = chain[i]
    if (entry.sequence !== i + 1) {
      residuals.push({
        code: 'RECEIPT-SEQ-GAP',
        surface: 'receipt-chain',
        message: `Entry ${i}: expected sequence ${i + 1}, got ${entry.sequence}`,
        severity: 'blocker',
        repair_candidate: 'Ensure receipts are numbered 1-based and contiguous',
      })
    }
    if (i > 0) {
      const prev = chain[i - 1]
      if (entry.prev_hash !== prev.receipt) {
        residuals.push({
          code: 'RECEIPT-PREV-HASH-MISMATCH',
          surface: 'receipt-chain',
          message: `Entry ${i} prev_hash "${entry.prev_hash}" does not match entry ${i - 1} receipt "${prev.receipt}"`,
          severity: 'blocker',
          repair_candidate: 'Re-derive prev_hash from the preceding receipt field',
        })
      }
    } else if (entry.prev_hash !== undefined && entry.prev_hash !== null) {
      residuals.push({
        code: 'RECEIPT-GENESIS-HAS-PREV',
        surface: 'receipt-chain',
        message: 'First receipt must not have a prev_hash',
        severity: 'error',
      })
    }
  }
  return residuals
}

export function detectMutation(original: RocketReceipt, candidate: RocketReceipt): boolean {
  return (
    original.sequence !== candidate.sequence ||
    original.event_type !== candidate.event_type ||
    original.receipt !== candidate.receipt ||
    original.prev_hash !== candidate.prev_hash
  )
}

export function detectBrokenPrevHash(chain: RocketReceipt[]): boolean {
  return validateReceiptChain(chain).some(r => r.code === 'RECEIPT-PREV-HASH-MISMATCH')
}

export function admitReceipt(receipt: RocketReceipt): RocketReceipt {
  return { ...receipt, status: 'ADMITTED' }
}

export function refuseReceipt(receipt: RocketReceipt, reason: string): RocketReceipt {
  return {
    ...receipt,
    status: 'REFUSED',
    residuals: [
      ...receipt.residuals,
      { code: 'RECEIPT-REFUSED', surface: receipt.surface, message: reason, severity: 'blocker' },
    ],
  }
}
