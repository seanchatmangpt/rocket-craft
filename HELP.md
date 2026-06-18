# Rocket Craft Help & Troubleshooting

Welcome to the Rocket Craft troubleshooting and help guide. With our migration to a **Rust SDK (`rocket-sdk`)** and the unified `./rocket` CLI wrapper, many of the legacy build and deployment issues have been streamlined.

## Unified Developer Experience (DX)

If you are having trouble building or managing the projects, **always use the `./rocket` CLI** rather than trying to invoke legacy bash scripts directly. The Rust orchestrator automatically manages dependencies, environment paths, and engine build steps for you.

```bash
# Access the built-in help manual for the CLI
./rocket help
```

*Note: Do not run legacy scripts like `setup.sh` or `HTML5Setup.sh` manually. These are now orchestrated by the Rust SDK.*

## Supabase & Web Platform (PWA)

If you are experiencing issues with authentication, leaderboards, or the TypeScript PWA:

- **TypeScript PWA (`pwa-staff/`):** Ensure your Node.js environment is configured correctly. The PWA communicates directly with the Supabase backend.
- **Supabase Connectivity:** Check your local environment variables and Supabase project settings if the game client fails to register sessions, authenticate users, or upload leaderboard scores.

## Unreal Engine C++ Template Projects

If you are building dedicated servers or modifying the C++ source projects (e.g., Shooter Game, Survival Game) and encounter build cache issues, the Rust SDK should handle clean rebuilds.

### Cleaning C++ Projects

If you need to manually perform a deep clean because of build system issues without triggering a full Engine rebuild:
1. Navigate to the specific project folder.
2. Delete the `Binaries`, `Build`, `DerivedDataCache`, and `Intermediate` folders.
3. Use the `./rocket` CLI to regenerate project files and trigger a clean build.

## Known Engine Quirks (UE 4.24 HTML5)

### Apex Destruction on WebGL
Apex destructible meshes can sometimes fail to initialize properly in packaged HTML5 builds. If you encounter assets that won't respond to damage or simulate physics in the browser:

**Workaround:** Replace hard references to the destructible mesh with **Soft Object References**. This ensures the destructible mesh asset is loaded *after* the Apex Destruction module during engine startup, allowing it to properly initialize when needed at runtime.

### Procedural Mesh Alternatives
For web targets where Apex destruction completely fails, consider using the `ProceduralMeshComponent` to perform runtime slice operations as a lightweight alternative to full destructibles.

## Android Keystores & Cryptographic Signing

Rocket Craft projects require cryptographic keystores for building Android targets. To secure the repository, actual `.keystore` files are excluded from git via `.gitignore`, and only `.placeholder` files are committed.

### Automated Management

The Rust SDK provides commands to check and guide keystore status:
```bash
# Check the status of project keystores
./rocket crypto status

# Print instructions for generating missing keystores and creating placeholders
./rocket crypto
```

### Keystore Generation Script

You can run the provided `generate-keystores.sh` script to generate fresh, valid keystores and place them in the correct directories for each project:
```bash
./generate-keystores.sh
```

This script generates and configures the following keystores:

1. **Barbarian Road Machines (BRM)**
   - Keystore: `barbarian-road-mashines-key.keystore`
   - Alias: `barbarian-road-mashines`
   - Password: `barbar12` (Configured in `versions/4.24.0/Config/DefaultEngine.ini`)
   - Target Path: `versions/4.24.0/Build/Android/barbarian-road-mashines-key.keystore`

2. **Epic Survival Game Series (SurvivalGame)**
   - Keystore: `zombie-key.keystore`
   - Alias: `zombie`
   - Password: `123456654321` (Configured in `versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Config/DefaultEngine.ini`)
   - Target Path: `versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Build/Android/zombie-key.keystore`

3. **Shooter Game**
   - Keystore: `hang3d-nightmare-keystore.keystore`
   - Alias: `NIGHTMARE` (listed as `nightmare` in keytool list output)
   - Password: `NIKOLALUKIC` (Configured in `versions/4.24-Shooter/ShooterGame/Config/DefaultEngine.ini`)
   - Target Path: `versions/4.24-Shooter/ShooterGame/Build/Android/hang3d-nightmare-keystore.keystore` (Also copies `zombie-key.keystore` to this directory)

## Nexus Engine & Typestate Ergonomics

All stateful modules in the `nexus-engine` workspace (combat, session, connection, auction, assembly) are governed by the **Typestate Pattern**, ensuring that invalid transitions are rejected at compile time.

### Builder Patterns
To construct these typestate-wrapped machines, use the provided builder patterns which perform runtime checks (like validating non-negative HP or non-empty usernames) before instantiating the compile-time safe state:
- `CombatMachineBuilder` -> Produces `CombatMachine<Idle>`
- `PlayerSessionBuilder` -> Produces `PlayerSession<Connecting>`
- `ConnectionBuilder` -> Produces `Connection<Disconnected>`
- `AuctionBuilder` -> Produces `Auction<OpenForBids>`
- `MechBuilder` -> Produces `Mech<Mob>`
- `CivilizationBuilder` -> Produces `Civilization<Plan>`
- `MechAssemblySpecBuilder` -> Produces `MechAssemblySpec<Unvalidated, P>`

### Dynamic/Runtime State Transition & Rejection Errors
When you need to perform transitions dynamically (e.g. processing a client packet), the workspace exposes runtime counterparts:
- `CombatState` & `CombatTransitionError`
- `SessionState` & `SessionTransitionError`
- `ConnectionState` & `ConnectionTransitionError`
- `AuctionState` & `AuctionTransitionError`

These error types explain exactly why a transition was rejected (e.g., trying to resolve a hit while in `Dodging` state).
