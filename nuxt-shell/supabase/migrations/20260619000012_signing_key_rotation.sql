-- migration 000012 — Ed25519 signing key rotation table
--
-- Pattern: ~/dashboard.bak/app/composables/useKeyManagement.js
--
-- Tracks the lifecycle of Ed25519 receipt-signing key pairs:
--   active   → the key currently used for signing cook receipts
--   rotating → grace period: old key still accepts verify, new key signs
--   revoked  → no longer valid for verification
--   expired  → past rotation_interval_days without renewal
--
-- When a new key is rotated in:
--   1. INSERT row with status='active', replaces_key_id = prior active key id
--   2. UPDATE prior active → status='rotating'
--   3. After grace_period_days, UPDATE rotating → status='revoked'
--
-- The public key is stored here so the browser can verify historical receipts
-- even after a rotation. The private key is NEVER stored — it lives in ROCKET_SIGNING_KEY.
--
-- Receipt rows in game_receipts.ed25519_sig store which key_id signed them,
-- enabling verification against the correct historical public key.

create table if not exists signing_keys (
  id                   uuid primary key default gen_random_uuid(),
  algorithm            text not null default 'Ed25519',
  public_key_b64       text not null,         -- base64-encoded Ed25519 public key
  status               text not null default 'active'
                         check (status in ('active', 'rotating', 'revoked', 'expired')),
  replaces_key_id      uuid references signing_keys(id) on delete set null,
  rotation_interval_days int not null default 90,
  grace_period_days    int not null default 7,
  created_at           timestamptz not null default now(),
  rotated_at           timestamptz,           -- when this key was superseded
  revoked_at           timestamptz,           -- when this key was revoked
  expires_at           timestamptz generated always as
                         (created_at + (rotation_interval_days || ' days')::interval) stored,
  notes                text
);

comment on table signing_keys is
  'Ed25519 key lifecycle for receipt signing. Private key in ROCKET_SIGNING_KEY env only.';
comment on column signing_keys.public_key_b64 is
  'Base64-encoded Ed25519 public key — safe to store and expose for verification.';
comment on column signing_keys.status is
  'active=current signing key; rotating=grace period; revoked=no longer valid; expired=past rotation window';
comment on column signing_keys.replaces_key_id is
  'Points to the key this row supersedes in the rotation chain.';

-- Index for fast active key lookup (only one active at a time)
create unique index if not exists signing_keys_active_unique
  on signing_keys (status)
  where status = 'active';

-- Index for key chain traversal (audit trail)
create index if not exists signing_keys_replaces_idx
  on signing_keys (replaces_key_id)
  where replaces_key_id is not null;

-- RLS: only authenticated service roles may write; anyone may read public keys
alter table signing_keys enable row level security;

create policy "signing_keys_public_read" on signing_keys
  for select using (true);

-- Helper: get the current active public key
create or replace function get_active_signing_public_key()
returns text
language sql stable security definer
as $$
  select public_key_b64
  from signing_keys
  where status = 'active'
  limit 1;
$$;

comment on function get_active_signing_public_key() is
  'Returns the current active Ed25519 public key for receipt verification.';
