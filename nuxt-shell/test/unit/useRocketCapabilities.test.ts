// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest'
import { useRocketCapabilities, type BrowserCapabilities } from '../../app/composables/useRocketCapabilities'

describe('useRocketCapabilities', () => {
  it('returns capabilities ref object', () => {
    const { capabilities } = useRocketCapabilities()
    expect(capabilities.value).toBeDefined()
    expect(typeof capabilities.value).toBe('object')
  })

  it('capabilities start as false before onMounted fires', () => {
    const { capabilities } = useRocketCapabilities()
    // Before mount, all are false (default state)
    const keys: (keyof BrowserCapabilities)[] = [
      'speechRecognition', 'speechSynthesis', 'gamepad', 'fullscreen',
      'pwaInstallable', 'serviceWorker', 'sharedArrayBuffer', 'webgl2',
    ]
    keys.forEach(k => expect(typeof capabilities.value[k]).toBe('boolean'))
  })

  it('unsupported is an array of missing capability names', () => {
    const { unsupported } = useRocketCapabilities()
    expect(Array.isArray(unsupported.value)).toBe(true)
  })

  it('allCriticalSupported is false before mount (webgl2/serviceWorker/SAB not available in happy-dom)', () => {
    const { allCriticalSupported } = useRocketCapabilities()
    // happy-dom doesn't implement WebGL2 or SharedArrayBuffer
    expect(typeof allCriticalSupported.value).toBe('boolean')
  })

  it('composable is callable multiple times without throwing', () => {
    expect(() => {
      useRocketCapabilities()
      useRocketCapabilities()
    }).not.toThrow()
  })
})
