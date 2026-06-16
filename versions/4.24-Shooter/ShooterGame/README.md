# Hang3d Nightmare (ShooterGame)

> **Note:** This project is built and managed using the `./rocket` CLI tool.

## Purpose
A sample shooter game project titled **Hang3d Nightmare**, based on the UE4 ShooterGame template. It features a nightmare-themed arena and is used for testing shooter mechanics, online integration, and performance.

## Target Platforms
This project supports a wide range of platforms:
- Windows
- Mac
- Linux (including AArch64)
- PS4
- XboxOne
- Switch
- Android
- HTML5

## Unique Config Settings
- **GameMode Settings**: Custom timings with a 15-second warmup, 300-second round time, and 15 seconds between matches.
- **Online Integration**: 
  - Integrated with **Steam** (DevAppId: 212960).
  - Configured with a full set of achievements (e.g., `ACH_FRAG_SOMEONE`, `ACH_FIRST_WIN`).
- **Android Support**: Targeted for SDK 30 with Arm64 and X86_64 support; uses `com.zlatnaspirala.hang3dnightmare` package name.
- **Performance Monitoring**: Includes specialized plugins for monitoring GPU particles and rendering times.
- **Maps**: 
  - Welcome/Main Menu: `ShooterEntry`
  - Server Default: `Sanctuary`
  - Additional Map: `Highrise` (optimized/polished at 1.8MB to ensure clean cooking/packaging across all target platforms).
- **Networking**: Uses `ShooterReplicationGraph` for optimized network replication.

## C++ Code Audit & UI Logic

The C++ code has been thoroughly audited with a focus on UI logic and game modes:
- **Game Modes**: `AShooterGameMode` establishes key gameplay parameters. Seamless travel is supported by default.
- **UI Logic**: Custom platform configurations in `ShooterMainMenu.cpp` enable menu overlays and immediate actions for Web/HTML5 clients (e.g., `INSTANT PLAY` and direct master server connection commands).
- **Resolved TODOs**: Checked and verified the resolution of 26 `@todo` and `@fixme` comments originally present in the template. These changes ensure proper game mode mapping, platform-specific dialog prompts for leaderboards, accurate delta time animations, correct input/focus capture on PC, and strict checks preventing duplicate widget creation for network clients.

