# Rocket OCEL8/OTEL8 Telemetry Codec Law

## Core Architecture
UE4 should emit byte-coded telemetry frames.
Rust expands, verifies, receipts, and exports canonical OCEL/OTEL.
UE4 must not emit giant JSON strings on the hot path.

## The Projection
- `ggen` → generated telemetry dictionary → generated UE4 C++ uint8 enums / structs
- UE4 emits compact byte frames (`RocketTelemetryFrame8`)
- Rust decoder verifies frames
- Rust maps byte codes to OCEL + OTEL
- Supabase / files / collector stores canonical evidence

## The Boundary Rule
UE4 emits bytes.
Rust restores meaning.
OCEL records object truth.
OTEL records execution truth.
Receipts decide standing.

Do not stream strings out of the game. Stream admitted byte facts, then expand them at the authority boundary.
