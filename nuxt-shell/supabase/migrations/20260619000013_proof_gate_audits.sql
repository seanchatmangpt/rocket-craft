-- proof_gate_audits: event log for every proof gate evaluation
-- Van der Aalst doctrine: if the gate ran but the log cannot prove it, it did not run.
-- These rows are the process evidence for the proof-gate sub-process.

create table if not exists proof_gate_audits (
  id           uuid primary key default gen_random_uuid(),
  session_id   text not null,
  gate_name    text not null,
  outcome      text not null check (outcome in ('pass', 'fail')),
  input_summary text,
  reason       text,
  evaluated_at timestamptz not null default now()
);

-- Fast lookup per session (e.g. "did all gates pass for session X?")
create index if not exists idx_proof_gate_audits_session
  on proof_gate_audits (session_id, evaluated_at desc);

-- Fast lookup for all failures (health monitoring)
create index if not exists idx_proof_gate_audits_fail
  on proof_gate_audits (outcome, evaluated_at desc)
  where outcome = 'fail';

-- Service role only — gate audit log must not be readable by anonymous clients
alter table proof_gate_audits enable row level security;

create policy "service role full access"
  on proof_gate_audits
  for all
  using (auth.role() = 'service_role');
