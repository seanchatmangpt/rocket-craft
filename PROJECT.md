# Project: Genie 26 World Manufacturing Platform

## Architecture
The platform is implemented as a new member crate `genie-core` in the `unify-rs` workspace. It bridges natural language intent, structured specifications, and Unreal Engine 4 T3D level layouts.

```
                  ┌────────────────────────────────────────┐
                  │          Natural Language Intent       │
                  └───────────────────┬────────────────────┘
                                      │
                                      ▼ (Parser)
                  ┌────────────────────────────────────────┐
                  │      Structured World Specification    │
                  │ (Places, Actors, Objects, Relations,   │
                  │    Rules, Processes, History, Receipts)│
                  └───────────────────┬────────────────────┘
                                      │
                   ┌──────────────────┴──────────────────┐
                   ▼ (Validation Gates)                  ▼ (Evolution)
        ┌─────────────────────┐               ┌─────────────────────┐
        │   Static Laws &     │               │   Evolver Engine    │
        │   SHACL Validation  │               │ (Incremental Update)│
        └──────────┬──────────┘               └──────────┬──────────┘
                   │                                     │
                   ▼ (Validated O*)                      ▼ (Evolved Spec)
        ┌─────────────────────┐               ┌─────────────────────┐
        │  T3D Scene layout   │◄──────────────┤   State Continuity  │
        │  World Generator    │               │    Preservation     │
        └──────────┬──────────┘               └─────────────────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Playable UE4 Map    │
        │ Cryptographic Proof │
        │ R ⊢ A = μ(O*)       │
        └─────────────────────┘
```

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Crate Setup & Data Model | Scaffold `genie-core` crate under `unify-rs/`, implement AST `WorldSpec`, register in cargo workspace. | None | PLANNED |
| 2 | Intent & Validation Gates | Implement intent parser/loader, semantic coherence static laws, SHACL engine validation, and BLAKE3 receipts. | M1 | PLANNED |
| 3 | World Manufacturing Engine | Implement `LayoutCompiler` to generate valid Unreal Engine 4 compatible `.t3d` level maps from specification. | M2 | PLANNED |
| 4 | Web Runtime & Deployment | Implement a web visualizer dashboard (HTML/JS) rendering the 3D layout, allowing player interaction, local server hosting, and telemetry logging. | M3 | PLANNED |
| 5 | Evolution & E2E Verification | Implement the evolution engine (incremental specification updates) and the `verify_genie.sh` script that compiles the workspace, launches the web runtime, and validates browser access. | M4 | PLANNED |

## Interface Contracts

### 1. `WorldSpec` Parsing & Serialization
```rust
pub fn parse_intent(intent: &str) -> Result<WorldSpec, GenieError>;
pub fn load_spec(path: &Path) -> Result<WorldSpec, GenieError>;
pub fn save_spec(spec: &WorldSpec, path: &Path) -> Result<(), GenieError>;
```

### 2. Validation & Admission Gates
```rust
pub struct WorldCoherenceLaw;
pub struct WorldCoherenceGate;

impl StaticLaw for WorldCoherenceLaw {
    const NAME: &'static str = "WorldCoherence";
    const DESCRIPTION: &'static str = "Enforce entity referential integrity and spatial bounds validation";
}

impl Admit<WorldCoherenceLaw> for WorldCoherenceGate {
    type Artifact = WorldSpec;
    type Refusal = Refusal<WorldCoherenceLaw>;

    fn admit(&self, spec: &WorldSpec) -> Result<(), Self::Refusal>;
}
```

### 3. World Generation & Layout Compiler
```rust
pub struct LayoutCompiler;

impl LayoutCompiler {
    /// Compiles the validated WorldSpec into a copy-pasteable UE4 T3D map string
    pub fn compile(spec: &WorldSpec) -> String;
}
```

### 4. Evolution & State Continuity
```rust
pub struct WorldEvolver;

impl WorldEvolver {
    /// Evolve an existing WorldSpec with a new modification intent, preserving existing structures
    pub fn evolve(spec: &WorldSpec, modification_intent: &str) -> Result<WorldSpec, GenieError>;
}
```

### 5. Deployment & Operation logs
```rust
pub struct DeploymentManager;

impl DeploymentManager {
    /// Launches the manufactured world (simulation or process launch) and registers entry logs
    pub fn deploy(spec: &WorldSpec, log_path: &Path) -> Result<(), GenieError>;
}
```

## Code Layout
- `unify-rs/Cargo.toml` (Register member `genie-core`)
- `unify-rs/genie-core/Cargo.toml`
- `unify-rs/genie-core/src/lib.rs`
- `unify-rs/genie-core/src/spec.rs` (Data models)
- `unify-rs/genie-core/src/laws.rs` (Coherence validation and typestate gates)
- `unify-rs/genie-core/src/layout.rs` (T3D compiler)
- `unify-rs/genie-core/src/evolution.rs` (Evolution engine)
- `unify-rs/genie-core/src/deployment.rs` (Operation, launch simulation & logs)
- `unify-rs/unify/src/app.rs` (CLI command options)
- `unify-rs/unify/src/commands.rs` (CLI execution handlers)
- `verify_genie.sh` (E2E validation script)
