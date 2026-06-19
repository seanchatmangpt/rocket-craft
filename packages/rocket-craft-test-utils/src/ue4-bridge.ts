import type { UE4BridgeEvent, GameIntent, AdmissionStatus } from './types.js'

export interface UE4BridgeSpy {
  received: UE4BridgeEvent[]
  rawBrowserEvents: UE4BridgeEvent[]
  clear(): void
}

const RAW_BROWSER_TYPES = new Set([
  'click', 'mousedown', 'mouseup', 'mousemove', 'keydown', 'keyup',
  'touchstart', 'touchend', 'pointerdown', 'pointerup',
])

export function createUE4BridgeMock(): UE4BridgeSpy {
  const received: UE4BridgeEvent[] = []
  const rawBrowserEvents: UE4BridgeEvent[] = []
  return {
    received,
    rawBrowserEvents,
    clear() { received.length = 0; rawBrowserEvents.length = 0 },
  }
}

export function installUE4BridgeSpy(spy: UE4BridgeSpy): (event: UE4BridgeEvent) => void {
  return (event: UE4BridgeEvent) => {
    spy.received.push(event)
    if (RAW_BROWSER_TYPES.has(event.type)) {
      spy.rawBrowserEvents.push(event)
    }
  }
}

export function sendAdmittedIntentToUE4(
  intent: GameIntent,
  handler: (event: UE4BridgeEvent) => void
): UE4BridgeEvent {
  const event: UE4BridgeEvent = {
    seq: intent.seq,
    type: intent.type,
    source: 'nuxt-shell',
    status: 'ADMITTED' as AdmissionStatus,
    payload: intent.payload,
  }
  handler(event)
  return event
}

export function expectUE4ReceivedIntent(spy: UE4BridgeSpy, type: string): void {
  const found = spy.received.some(e => e.type === type)
  if (!found)
    throw new Error(`UE4 bridge never received intent "${type}". Got: ${spy.received.map(e => e.type).join(', ')}`)
}

export function expectUE4DidNotReceiveRawBrowserEvent(spy: UE4BridgeSpy): void {
  if (spy.rawBrowserEvents.length > 0)
    throw new Error(
      `UE4 received raw browser events (law violation): ${spy.rawBrowserEvents.map(e => e.type).join(', ')}`
    )
}

export function expectUE4EmittedProjectionEvent(spy: UE4BridgeSpy, type: string): void {
  const found = spy.received.some(e => e.type === type && e.source === 'ue4-canvas')
  if (!found)
    throw new Error(`UE4 canvas never emitted projection event "${type}"`)
}
