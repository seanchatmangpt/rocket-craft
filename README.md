# Rocket Craft

Rocket Craft is a multiplatform gaming ecosystem built on the battle-tested **Unreal Engine 4.24** architecture. Orchestrated by a modern **Rust SDK** (`rocket-sdk` and the `./rocket` CLI), Rocket Craft provides a unified developer experience (DX) for managing, building, and deploying highly optimized Unreal Engine projects across Web (HTML5/WebGL), Windows, Linux, macOS, Android, and iOS.

## Architecture & Tech Stack

Rocket Craft uses a hybrid architecture, combining the power of Unreal Engine with a modern web stack:

- **Core Engine:** Unreal Engine 4.24 (Custom source build with advanced HTML5/WebGL support)
- **Backend Services:** **Supabase** (PostgreSQL) powers real-time multiplayer authentication, global leaderboards, and administrative systems.
- **Frontend / PWA:** Written in **TypeScript**, the Progressive Web App (located in `pwa-staff/`) integrates seamlessly with the backend and provides a rich web presence.
- **Orchestration & DX:** A unified **Rust SDK** (`rocket-sdk`) and command-line wrapper (`./rocket`) drive the entire development lifecycle. *Raw bash scripts are no longer the primary entry point.*

## Core Projects

Rocket Craft hosts several sub-projects demonstrating our multiplatform capabilities:

- **Shooter FPS (Hang3d Nightmare):** [C++] Multiplayer active, targeting OpenGL ES 3.0.
- **Barbarian Road Mashines:** [Blueprint] High-speed racing and zombie survival.
- **ShootTheZombie2:** [C++] Based on a classic survival zombie shooter template.
- **Realistic Rendering:** [Blueprint] Pushing UE4 HTML5 capabilities to the maximum visual fidelity.
- **FullSpectrum:** [Blueprint] A clean, empty template for starting new experiences.

## Unified Developer Experience (DX)

Rocket Craft has moved away from manual, fragmented shell scripts. All project management, building, packaging, and deployment are now handled by the **Rocket CLI**.

### Getting Started

To manage the project, use the centralized Rust-based CLI:

```bash
# Example usage of the new unified CLI
./rocket --help
```

*Note: Legacy raw bash scripts (such as `setup.sh`, `generate-keystores.sh`, or manual UE setup scripts) have been deprecated and are wrapped internally by the Rust SDK.*

## Supabase Backend & TypeScript PWA

The ecosystem features a modern, unified web layer:
- **Authentication & Admin:** Managed securely via Supabase.
- **Global Leaderboards:** Competitive tracking natively integrated.
- **PWA Staff:** A fully TypeScript-based frontend (`pwa-staff/`) that serves as the portal for game launches, user profiles, administration, and community features.
- **Offline PWA & HTML5 Caching:** Fully offline support via a service worker caching mechanism, backing up games for client-side play. For detailed offline architecture, see [docs/PWA_OFFLINE.md](docs/PWA_OFFLINE.md).

## Multiplatform Unreal Engine 4.24

We deliberately utilize UE 4.24 because it represents the ultimate checkpoint for robust **HTML5/WebGL** support alongside stable OpenGL ES 2.0/3.1 compatibility for mobile devices. This allows a true **"Write Once, Run Anywhere"** pipeline:

- **Web (HTML5):** Seamless browser playability.
- **Desktop:** Standalone clients and dedicated server support for Windows, Linux, and macOS.
- **Mobile:** Android and iOS builds using optimized rendering paths.

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on the workspace structure, our Rust orchestration, and contribution workflows.
