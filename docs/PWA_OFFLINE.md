# Progressive Web App (PWA) Offline & Caching Architecture

This document describes the Progressive Web App (PWA) architecture, the offline caching strategies, and the asset synchronization pipeline for the Rocket Craft web interface (`pwa-staff`).

---

## 1. Overview

The Rocket Craft frontend portal is designed as a Progress Web App that can run fully offline, enabling developers and players to launch and interact with simulated Unreal Engine HTML5 games even without active internet connectivity.

Offline functionality is driven by three components:
1.  **Service Worker (`worker.js`):** Intercepts and caches HTTP requests.
2.  **Offline Fallback UI (`offline.html`):** Displays a user-friendly reconnect screen when network navigation fails.
3.  **PWA Asset Manifest (`manifest.json`):** Tracks and lists all deployable and local PWA assets.

---

## 2. PWA Build & Asset Sync Pipeline

The orchestration of PWA assets is integrated into the Rust-based `./rocket` CLI.

### 2.1 Synchronization Command
To sync the local PWA files and generate/update the asset manifest, execute:
```bash
./rocket pwa sync
```
Internally, this traverses the `pwa-staff/` directory (excluding `node_modules`, hidden files, and `manifest.json` itself) and outputs a pretty-printed `manifest.json` cataloging all assets with a generated version tag.

### 2.2 Compilation Command
To compile the TypeScript files (including the service worker `worker.ts` and `cache.ts`) and bundle them using esbuild, run:
```bash
cd pwa-staff
npm run build
```
This performs:
- **CSS build:** `postcss css/style.css -o dist/style.css`
- **TypeScript build:** Bundles all page-specific scripts to `dist/` and transpiles `worker.ts` to `worker.js`.

---

## 3. Caching Strategy & Logic

The Service Worker (`pwa-staff/worker.ts`) implements a hybrid caching strategy optimized for both administrative portal availability and local game loading.

### 3.1 Pre-caching on Install
During the Service Worker `install` event, all core portal files and HTML5 shipping builds are fetched and cached into `static-assets-v2`.

Pre-cached assets include:
-   **Core UI Pages:** `index.html`, `admin.html`, `leaderboard.html`, `login.html`, `signup.html`, `profile.html`
-   **Bundled Scripts & Styles:** `cache.js`, `lucide.min.js`, `dist/style.css`, `dist/*.js`
-   **HTML5 Shipping Game Packages:** HTML, JS loader, WebAssembly, and `.data` files for:
    -   **Barbarian Road Mashines:** `Brm-HTML5-Shipping.*`
    -   **ShootTheZombie2:** `SurvivalGame-HTML5-Shipping.*`
    -   **Realistic Rendering:** `RealisticRendering-HTML5-Shipping.*`
    -   **FullSpectrum Template:** `FullSpectrum-HTML5-Shipping.*`
    -   **Hang3d Nightmare:** `ShooterGame-HTML5-Shipping.*`

#### Critical Asset Fallback
The `install` event uses `Promise.allSettled` to avoid blocking installation if non-critical assets (such as development test specs) fail. However, `offline.html` is marked as **critical**. If `offline.html` fails to cache, the service worker installation fails immediately.

### 3.2 Network First (Navigation Requests)
For HTML navigation (page transitions), a **Network First** approach is used:
1.  Attempt to fetch the page from the network.
2.  If successful, update the `dynamic-content-v2` cache.
3.  If offline or network fails, fall back to `offline.html`.

### 3.3 Cache First (Static Assets)
For assets (scripts, styles, images, WASM binaries, and data files), a **Cache First** approach is used:
1.  Check the cache (`static-assets-v2` or `dynamic-content-v2`).
2.  If cached, serve immediately (eliminating latency and network dependency).
3.  If not cached, fetch from network, and dynamically cache same-origin (`basic` type) responses.

---

## 4. Offline Fallback UI (`offline.html`)

When a user is offline and attempts to navigate to a page that isn't cached, they are redirected to `offline.html`.

### 4.1 Features
-   **Auto-reconnect:** Listens to the window `online` event and automatically reloads the page once connectivity is restored.
-   **Manual Retry:** Provides a "Retry Connection" button that fetches a lightweight asset (`favicon.ico`) with no-cache/no-cors constraints to verify internet connectivity.
-   **Lucide Icons:** Uses SVG-based Lucide icons (cached locally via `lucide.min.js`) to ensure correct rendering offline.
