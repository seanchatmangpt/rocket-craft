import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const FIXTURE_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..', 'fixtures')

export { FIXTURE_DIR }

export function loadFixture<T = unknown>(name: string): T {
  const p = resolve(FIXTURE_DIR, name.endsWith('.json') ? name : `${name}.json`)
  const raw = readFileSync(p, 'utf8')
  return JSON.parse(raw) as T
}
