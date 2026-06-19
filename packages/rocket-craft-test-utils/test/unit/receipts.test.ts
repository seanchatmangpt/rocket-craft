import { describe, it, expect } from 'vitest'
import { loadFixture } from '../../src/fixtures.js'
import {
  validateReceiptChain, detectMutation, detectBrokenPrevHash, buildReceipt,
} from '../../src/receipts.js'
import type { RocketReceipt } from '../../src/types.js'

describe('receipt chain — valid fixture', () => {
  it('admits a well-formed chain', () => {
    const chain = loadFixture<RocketReceipt[]>('receipt-chain.valid.json')
    const residuals = validateReceiptChain(chain)
    expect(residuals).toHaveLength(0)
  })

  it('detects broken prev_hash', () => {
    const chain = loadFixture<RocketReceipt[]>('receipt-chain.broken-prev-hash.json')
    expect(detectBrokenPrevHash(chain)).toBe(true)
  })

  it('detects missing sequence', () => {
    const chain = loadFixture<RocketReceipt[]>('receipt-chain.missing-sequence.json')
    const residuals = validateReceiptChain(chain)
    expect(residuals.some(r => r.code === 'RECEIPT-SEQ-GAP')).toBe(true)
  })

  it('detects mutation between original and tampered receipt', () => {
    const original = buildReceipt(1, 'Interact', 'game-shell', 'sha256:abc')
    const mutated: RocketReceipt = { ...original, receipt: 'sha256:TAMPERED' }
    expect(detectMutation(original, mutated)).toBe(true)
  })

  it('does NOT flag identical receipts as mutated', () => {
    const r = buildReceipt(1, 'Interact', 'game-shell', 'sha256:abc')
    expect(detectMutation(r, { ...r })).toBe(false)
  })

  it('genesis receipt must not have prev_hash', () => {
    const chain: RocketReceipt[] = [
      buildReceipt(1, 'Interact', 'shell', 'sha256:aaa', { prev_hash: 'sha256:SHOULDNOTEXIST' }),
    ]
    const residuals = validateReceiptChain(chain)
    expect(residuals.some(r => r.code === 'RECEIPT-GENESIS-HAS-PREV')).toBe(true)
  })
})
