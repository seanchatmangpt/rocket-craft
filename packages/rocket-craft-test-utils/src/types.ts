export type RocketStatus =
  | 'UNKNOWN'
  | 'PARTIAL'
  | 'PARTIAL_ALIVE'
  | 'PARTIAL_ALIVE_CANDIDATE'
  | 'ALIVE_UNDER_SCOPE'
  | 'VERIFIED_UNDER_SCOPE'
  | 'REFUSED'
  | 'RESIDUAL'
  | 'BLOCKED'

export type AdmissionStatus = 'ADMITTED' | 'REFUSED' | 'RESIDUAL' | 'UNKNOWN'

export interface RocketReceipt {
  sequence: number
  event_type: string
  surface: string
  input_hash?: string
  output_hash?: string
  prev_hash?: string
  receipt: string
  status: AdmissionStatus
  residuals: RocketResidual[]
}

export interface RocketResidual {
  code: string
  surface: string
  message: string
  severity: 'info' | 'warn' | 'error' | 'blocker'
  repair_candidate?: string
}

export interface GameIntent {
  seq: number
  type: string
  source: string
  payload?: Record<string, unknown>
  status?: AdmissionStatus
}

export interface VisualDeltaResult {
  baseline_hash: string
  after_hash: string
  changed_pixels?: number
  delta_ratio?: number
  admitted: boolean
  residuals: RocketResidual[]
}

export interface CommandReceipt {
  command: string
  cwd: string
  exit_code: number
  stdout_hash: string
  stderr_hash: string
  duration_ms: number
  status: AdmissionStatus
  residuals: RocketResidual[]
}

export interface UE4BridgeEvent {
  seq: number
  type: string
  source: 'nuxt-shell' | 'ue4-canvas' | 'playwright' | 'supabase-realtime'
  status?: AdmissionStatus
  payload?: Record<string, unknown>
  receipt?: string
}

export interface VerifierReport {
  milestone: string
  status: RocketStatus
  scoped_status: string
  gates: GateResult[]
  residuals: RocketResidual[]
  jidoka_events: JidokaEvent[]
  emitted_at: string
}

export interface GateResult {
  name: string
  status: AdmissionStatus
  detail?: string
  residuals: RocketResidual[]
}

export interface JidokaEvent {
  defect_class: string
  surface: string
  expected_law: string
  observed_failure: string
  residual: RocketResidual
  repair_candidate?: string
  repair_applied: boolean
  receipt?: string
}
