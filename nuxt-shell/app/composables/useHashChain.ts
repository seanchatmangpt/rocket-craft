/**
 * Hash Chain Composable for Tamper Evidence
 * Ported from dashboard.bak/app/composables/useHashChain.js to TypeScript (Nuxt 4).
 *
 * Van der Aalst doctrine: if the event log cannot prove a lawful process happened,
 * then it did not work.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/** Canonical shape hashed by computeEventHash. Must stay in this exact key order. */
export interface HashChainEvent {
  id: string
  timestamp: string
  type: string
  data: Record<string, unknown>
  prev_hash: string | null
}

/** A linked event — carries the hash of itself plus the prev_hash reference. */
export interface LinkedEvent extends HashChainEvent {
  hash: string
}

/** One entry in a break list produced by verifyChain / findBreaks. */
export interface ChainBreak {
  index: number
  eventId: string
  type: 'hash_mismatch' | 'chain_break' | 'genesis_error'
  expected: string | null
  actual: string | null
  message: string
}

/** Rich break returned by findBreaks — includes severity and the offending event. */
export interface EnrichedChainBreak extends ChainBreak {
  severity: 'critical' | 'high'
  /** ISO timestamp at which findBreaks was called — supplied by caller, never Date.now(). */
  detectedAt: string
  event: LinkedEvent
}

/** Result of verifyChain. */
export interface ChainVerifyResult {
  valid: boolean
  breaks: ChainBreak[]
  totalEvents: number
  validEvents: number
  /** Ratio validEvents / totalEvents; NaN when totalEvents === 0. */
  integrity: number
}

// ── Implementation ────────────────────────────────────────────────────────────

/**
 * Convert a 32-byte ArrayBuffer to a lowercase hex string.
 * Shared by computeEventHash and computeMerkleRoot.
 */
function bufferToHex(buffer: ArrayBuffer): string {
  return Array.from(new Uint8Array(buffer))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

/**
 * useHashChain — cryptographic hash-chain utilities for tamper-evident event logging.
 *
 * Named function export (not const arrow) for Nuxt 4 auto-import compatibility.
 */
export function useHashChain() {
  /**
   * Compute SHA-256 hash of an event.
   *
   * Canonical JSON is built from exactly these five keys in this order:
   *   { id, timestamp, type, data, prev_hash }
   * Changing key order would break all existing chain hashes — do not reorder.
   *
   * @param event - Event to hash; prev_hash defaults to null when absent.
   * @returns Lowercase hex SHA-256 string.
   */
  const computeEventHash = async (event: HashChainEvent): Promise<string> => {
    const canonical = JSON.stringify({
      id: event.id,
      timestamp: event.timestamp,
      type: event.type,
      data: event.data,
      prev_hash: event.prev_hash ?? null,
    })

    const encoded = new TextEncoder().encode(canonical)
    const hashBuffer = await crypto.subtle.digest('SHA-256', encoded)
    return bufferToHex(hashBuffer)
  }

  /**
   * Link current event to previous event in the chain.
   *
   * Sets current.prev_hash to the hash of `previous` (or null for genesis),
   * then computes and attaches current.hash.
   *
   * @param current  - Current event (without hash fields).
   * @param previous - Previous linked event, or null if this is the genesis event.
   * @returns A new LinkedEvent with both prev_hash and hash populated.
   */
  const linkEvents = async (
    current: HashChainEvent,
    previous: LinkedEvent | null,
  ): Promise<LinkedEvent> => {
    const linkedEvent: HashChainEvent = {
      ...current,
      prev_hash: previous ? await computeEventHash(previous) : null,
    }
    const hash = await computeEventHash(linkedEvent)
    return { ...linkedEvent, hash }
  }

  /**
   * Verify the integrity of an entire event chain.
   *
   * For each event:
   *  1. Re-computes the expected hash from its canonical fields.
   *  2. Compares against the stored event.hash.
   *  3. For non-genesis events, verifies event.prev_hash matches the prior event's hash.
   *  4. For the genesis event (index 0), verifies prev_hash is null.
   *
   * @param events - Ordered array of LinkedEvents (genesis first).
   * @returns ChainVerifyResult — valid iff breaks is empty.
   */
  const verifyChain = async (events: LinkedEvent[]): Promise<ChainVerifyResult> => {
    if (!events || events.length === 0) {
      return { valid: true, breaks: [], totalEvents: 0, validEvents: 0, integrity: NaN }
    }

    const breaks: ChainBreak[] = []
    let validEvents = 0

    for (let i = 0; i < events.length; i++) {
      const event = events[i]!

      // 1. Verify own hash
      const expectedHash = await computeEventHash({
        id: event.id,
        timestamp: event.timestamp,
        type: event.type,
        data: event.data,
        prev_hash: event.prev_hash,
      })

      if (event.hash !== expectedHash) {
        breaks.push({
          index: i,
          eventId: event.id,
          type: 'hash_mismatch',
          expected: expectedHash,
          actual: event.hash,
          message: `Event ${event.id} hash mismatch`,
        })
        continue
      }

      // 2. Verify chain linkage
      if (i === 0) {
        // Genesis must have no predecessor
        if (event.prev_hash !== null) {
          breaks.push({
            index: i,
            eventId: event.id,
            type: 'genesis_error',
            expected: null,
            actual: event.prev_hash,
            message: `Genesis event ${event.id} should have null prev_hash`,
          })
          continue
        }
      } else {
        const previousEvent = events[i - 1]!
        const expectedPrevHash = await computeEventHash({
          id: previousEvent.id,
          timestamp: previousEvent.timestamp,
          type: previousEvent.type,
          data: previousEvent.data,
          prev_hash: previousEvent.prev_hash,
        })

        if (event.prev_hash !== expectedPrevHash) {
          breaks.push({
            index: i,
            eventId: event.id,
            type: 'chain_break',
            expected: expectedPrevHash,
            actual: event.prev_hash,
            message: `Chain break at event ${event.id}`,
          })
          continue
        }
      }

      validEvents++
    }

    return {
      valid: breaks.length === 0,
      breaks,
      totalEvents: events.length,
      validEvents,
      integrity: validEvents / events.length,
    }
  }

  /**
   * Find and enrich breaks in the chain.
   *
   * Delegates to verifyChain, then annotates each break with severity and the
   * offending event object. Callers must supply `detectedAt` (an ISO timestamp)
   * so this function remains deterministic.
   *
   * @param chain      - Ordered array of LinkedEvents.
   * @param detectedAt - ISO timestamp to stamp on each break (caller supplies, no Date.now()).
   * @returns Array of EnrichedChainBreak entries.
   */
  const findBreaks = async (
    chain: LinkedEvent[],
    detectedAt: string,
  ): Promise<EnrichedChainBreak[]> => {
    const result = await verifyChain(chain)
    return result.breaks.map((b) => ({
      ...b,
      severity: (b.type === 'hash_mismatch' ? 'critical' : 'high') as 'critical' | 'high',
      detectedAt,
      event: chain[b.index]!,
    }))
  }

  /**
   * Compute a Merkle root over an array of hex hash strings.
   *
   * Uses standard binary tree reduction: pairs are SHA-256(left + right),
   * odd nodes duplicate themselves.
   *
   * @param hashes - Array of lowercase hex SHA-256 strings.
   * @returns Merkle root hex string, or null for an empty input.
   */
  const computeMerkleRoot = async (hashes: string[]): Promise<string | null> => {
    if (!hashes || hashes.length === 0) return null
    if (hashes.length === 1) return hashes[0]!

    let currentLevel = [...hashes]

    while (currentLevel.length > 1) {
      const nextLevel: string[] = []

      for (let i = 0; i < currentLevel.length; i += 2) {
        const left = currentLevel[i]!
        const right = currentLevel[i + 1] ?? left // duplicate odd leaf

        const combined = left + right
        const encoded = new TextEncoder().encode(combined)
        const hashBuffer = await crypto.subtle.digest('SHA-256', encoded)
        nextLevel.push(bufferToHex(hashBuffer))
      }

      currentLevel = nextLevel
    }

    return currentLevel[0]!
  }

  /**
   * Create a genesis (first) event for a new hash chain.
   *
   * Callers must supply both `id` and `timestamp` to keep this function
   * deterministic (no Date.now() or crypto.randomUUID() inside).
   *
   * @param id          - Unique identifier for the genesis event.
   * @param timestamp   - ISO timestamp string (caller-supplied).
   * @param initialData - Arbitrary payload for the genesis event.
   * @returns A fully-hashed LinkedEvent with prev_hash === null.
   */
  const createGenesisEvent = async (
    id: string,
    timestamp: string,
    initialData: Record<string, unknown> = { message: 'Chain genesis' },
  ): Promise<LinkedEvent> => {
    const genesisEvent: HashChainEvent = {
      id,
      timestamp,
      type: 'genesis',
      data: initialData,
      prev_hash: null,
    }
    const hash = await computeEventHash(genesisEvent)
    return { ...genesisEvent, hash }
  }

  /**
   * Validate that an event object has all required fields before adding to chain.
   *
   * Does NOT compute any hashes — purely structural / synchronous.
   *
   * @param event - Candidate event object (typed loosely to allow pre-validation).
   * @returns { valid, errors } — errors is empty when valid.
   */
  const validateEventStructure = (
    event: Partial<HashChainEvent>,
  ): { valid: boolean; errors: string[] } => {
    const required: Array<keyof HashChainEvent> = ['id', 'timestamp', 'type', 'data']
    const missing = required.filter((field) => !(field in event) || event[field] === undefined)

    if (missing.length > 0) {
      return { valid: false, errors: [`Missing required fields: ${missing.join(', ')}`] }
    }

    if (isNaN(Date.parse(event.timestamp!))) {
      return { valid: false, errors: ['Invalid timestamp format'] }
    }

    return { valid: true, errors: [] }
  }

  return {
    computeEventHash,
    linkEvents,
    verifyChain,
    findBreaks,
    computeMerkleRoot,
    createGenesisEvent,
    validateEventStructure,
  }
}
