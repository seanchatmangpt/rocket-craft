# Barbarian Road Mashines (Brm)

> **Note:** This project is built and managed using the `./rocket` CLI tool.

## Purpose
A vehicle-based action game titled **Barbarian Road Mashines**, built upon the Advanced Vehicle Template. It emphasizes high-speed, chaotic road gameplay ("No roles on this roads").

## Target Platforms
- **Mobile/Android**: Extensive configuration for mobile deployment.
- **HTML5/Web**: Integrated with WebSocket networking for web-based multiplayer.

## Unique Config Settings
- **Networking**: Uses `WebSocketNetDriver` (Port 8889) for HTML5 and web compatibility.
- **Packaging & Deployment**:
  - `UsePakFile` is set to **False**, which is unique for this project.
  - Utilizes **Early Downloader** configuration for specific asset types (icu, brk, res, etc.).
- **Android Specifics**: 
  - Targeted for SDK 26.
  - Configured for multiple texture formats (ETC1, ETC2, DXT, PVRTC, ATC, ASTC) with specific priorities.
  - Supports Daydream VR.
- **Physics**: **Substepping** is enabled to ensure stable vehicle simulation at high speeds.
- **Maps**: 
  - Startup: `menuMap`
  - Server Default: `barbarian-1`
- **Game Instance**: Uses a custom Blueprint-based game instance: `main-config_C`.
