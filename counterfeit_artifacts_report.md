# Counterfeit Artifact Audit Report

## Executive Summary

This audit report identifies, catalogs, and analyzes several counterfeit, fake, and mock artifacts within the `rocket-craft` codebase. These artifacts represent a systemic mocking strategy that bypasses the genuine Unreal Engine 4 compiler and cooking pipeline. Instead of relying on a real browser-native Unreal Engine 4 HTML5/WASM world built with the SpeculativeCoder UE4.27 HTML5 ES3 fork, the current implementation relies on simulated paths, lightweight JavaScript 2D/3D renderer fallbacks, 8-byte WASM placeholders, 1.5MB dummy WASM binaries built from basic Rust math libraries, plain-text JSON files masquerading as cooked binary packages, and hardcoded fallback configurations.

These findings represent a direct violation of the governing **Combinatorial Maximalist (CM) doctrine** and the **Playwright Manufacturing Strategy** outlined in `GEMINI.md`. Specifically, they bypass the required crown path and fail multiple validation gates within the project's acceptance matrix. The doctrine commands: *"A successful compilation or passing unit test is a false positive—it just means the parts fit together, not that the car drives."* and *"Victory requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input."* 

By documenting these violations, this report establishes a baseline of defects to be repaired by failure taxonomy.

---

## Detailed Catalog of Counterfeit Artifacts

### 1. Simulated Engine Detection

#### A. Mock `UE4_ROOT` Path in Unit Tests
* **File Paths:**
  - `/Users/sac/rocket-craft/unify-rs/genie-core/tests/implementation_tests.rs` (Line 99)
  - `/Users/sac/rocket-craft/unify-rs/genie-core/tests/milestone3_5.rs` (Line 76)
* **Category:** Simulated Engine Path Mocking
* **File Type:** Rust Test Source (`.rs`)
* **Exact Evidence:**
  ```rust
  if std::env::var("UE4_ROOT").is_err() {
      std::env::set_var("UE4_ROOT", "/Users/sac/ue4-sim");
  }
  ```
* **Doctrine Violation Explanation:**
  Setting a mock environment variable pointing to a non-existent local directory (`/Users/sac/ue4-sim`) bypasses standard build verification. The tests assert successful setup and execution configurations but fail compilation when executed in a clean environment. This violates **GEMINI.md GATE 2 (HTML5/WASM Package Admission)** because it does not attempt to locate, build, or deploy using the real Unreal Engine source tree, allowing tests to pretend they compile and deploy code when they do not. It also violates the Core Mandate: *"A successful compilation or passing unit test is a false positive—it just means the parts fit together, not that the car drives."*

#### B. The `rocket-simulator` Crate
* **File Paths:**
  - `/Users/sac/rocket-craft/rocket-simulator/` (Monorepo crate)
  - `/Users/sac/rocket-craft/rocket-simulator/simulator-core/src/lib.rs`
* **Category:** CLI / Emulation Engine Coordinator
* **File Type:** Rust Project Directory / Source Crate
* **Exact Evidence:**
  The crate coordinates `genie_server.js` and Playwright tests in `pwa-staff` directly without invoking compiler tools or loading generated levels/world assets.
* **Doctrine Violation Explanation:**
  The crate acts as a simulator that coordinates test runs while bypassing the actual compilation and packaging phases of Unreal Engine. This violates the core tenet in `GEMINI.md`: *"The pipeline must not accept: Rust-only simulation, CLI emulation, mocked worlds..."*. It violates the Playwright Manufacturing Strategy by substituting the genuine WASM-cooked engine compilation and runtime with a lightweight Rust-based emulation loop, failing to prove that the actual cooked world drives.

---

### 2. Mock Projection Detection

#### A. Counterfeit WebGL Renderer falling back to JavaScript
* **File Paths:**
  - `/Users/sac/rocket-craft/pwa-staff/Brm-HTML5-Shipping.js`
  - `/Users/sac/rocket-craft/pwa-staff/manufactured/Brm-HTML5-Shipping.js`
* **Category:** Mock WebGL Projection
* **File Type:** JavaScript Source (`.js`)
* **Exact Evidence:**
  In `Brm-HTML5-Shipping.js`, the WebAssembly module bytes are compiled and loaded, but the resulting `wasmModule` is never used:
  ```javascript
  if (wasmBytes) {
    updateStatus('Compiling WebAssembly binaries...', 'Verifying game compilation...', 100);
    return WebAssembly.instantiate(wasmBytes, {}).then((result) => {
      wasmModule = result.instance;
      runGame();
    }).catch(e => {
      console.warn('WASM instantiation failed, falling back to JS gait engine:', e);
      wasmModule = null;
      runGame();
    });
  } else {
    wasmModule = null;
    runGame();
  }
  ```
  The resulting `wasmModule` is never referenced or used anywhere else in the JavaScript. Instead, the rendering loop `runGame()` draws standard 3D primitives (Cylinder, Cube, Sphere) directly using JavaScript-based shaders and vertex matrices on the canvas.
* **Doctrine Violation Explanation:**
  The WebGL renderer compiles standard 3D shapes on a Canvas via pure JavaScript rather than executing engine bytecode inside the WASM module. If WASM compilation fails, it quietly falls back to JS rendering. This violates the **Playwright Manufacturing Strategy**'s Crown Path: *"Victory requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input."* Because the motion is generated by JS code rather than the cooked WASM engine runtime, Playwright is validating a fake JavaScript simulation instead of the real Unreal Engine 4 WASM environment. This violates **GATE 4 (Visual World Admission)** and **GATE 6 (Motion Admission)**.

#### B. 2D Interactive Drawing/Kaleidoscope Projection Stubs
* **File Paths:**
  - `/Users/sac/rocket-craft/pwa-staff/FullSpectrum-HTML5-Shipping.js`
  - `/Users/sac/rocket-craft/pwa-staff/RealisticRendering-HTML5-Shipping.js`
  - `/Users/sac/rocket-craft/pwa-staff/ShooterGame-HTML5-Shipping.js`
  - `/Users/sac/rocket-craft/pwa-staff/SurvivalGame-HTML5-Shipping.js`
* **Category:** Counterfeit Visual Projection
* **File Type:** JavaScript Source (`.js`)
* **Exact Evidence:**
  ```javascript
  function runGame() {
    console.log('Game Assets Decoded:', gameDataText);
    window.UE4_EngineReady = true;
    // ...
    const ctx = canvas.getContext('2d'); // Draws a 2D interactive canvas brush / kaleidoscope / particles
  ```
* **Doctrine Violation Explanation:**
  These files simulate UE4 engine readiness by setting `window.UE4_EngineReady = true` immediately and rendering a 2D canvas animation (brush/kaleidoscope/particles). This bypasses the actual compilation of the corresponding Unreal Engine games. This violates **GATE 3 (Browser Load Admission)** and **GATE 4 (Visual World Admission)**: *"PASS only if screenshot shows a non-error WebGL/Unreal scene"*, not a 2D canvas drawing script, and **GATE 6 (Motion Admission)**: delta must result from actual engine movement, not a 2D particle/kaleidoscope simulation. It also represents the type of "mocked worlds" and "MVP iteration" that the Law of Combinatorial Maximalism strictly forbids.

---

### 3. Stub Artifact Detection

#### A. 8-Byte Magic-Header WASM Placeholders
* **File Paths:**
  - `/Users/sac/rocket-craft/pwa-staff/FullSpectrum-HTML5-Shipping.wasm`
  - `/Users/sac/rocket-craft/pwa-staff/RealisticRendering-HTML5-Shipping.wasm`
  - `/Users/sac/rocket-craft/pwa-staff/ShooterGame-HTML5-Shipping.wasm`
  - `/Users/sac/rocket-craft/pwa-staff/SurvivalGame-HTML5-Shipping.wasm`
* **Category:** Stub WASM Binary
* **File Type:** WebAssembly Binary (`.wasm`)
* **Exact Evidence:**
  File size is exactly 8 bytes. Hexdump output:
  ```
  00 61 73 6d 01 00 00 00
  ```
  This corresponds strictly to the `\0asm` magic header and WebAssembly version 1.
* **Doctrine Violation Explanation:**
  Bypasses **GATE 2 (HTML5/WASM Package Admission)** by providing empty headers instead of compiled engine bytecode. These files are dummy placeholders that pass simple file existence/header validation checks but contain no executable logic. This violates the core mandate and is a clear example of the forbidden MVP design pattern.

#### B. The 1.5MB Dummy WASM Crate
* **File Paths:**
  - `/Users/sac/rocket-craft/pwa-staff/Brm-HTML5-Shipping.wasm`
  - `/Users/sac/rocket-craft/tools/gait-wasm/src/lib.rs` (compiled source)
* **Category:** Stub WASM Binary
* **File Type:** WebAssembly Binary (`.wasm`)
* **Exact Evidence:**
  The `Brm-HTML5-Shipping.wasm` file is only 1.5MB, whereas a cooked Unreal Engine WASM binary is typically 30MB+. Strings inside the WASM file show it is built from a small Rust crate (`gait-wasm`) containing gait rotation equations (`get_gait_rotation_x`, `get_foundry_rotation`) rather than Unreal Engine core.
* **Doctrine Violation Explanation:**
  Although the file is a valid WebAssembly binary, it contains no Unreal Engine codebase or engine runtime logic. It was created to pass file existence and basic size checks in packaging/serving scripts (like `package-brm-html5.sh`) without compiling a real world. This violates **GATE 2 (HTML5/WASM Package Admission)** because the browser loads a custom math library instead of a SpeculativeCoder UE4.27 HTML5 ES3 build.

#### C. Cooked Asset JSON Plaintext Stubs
* **File Paths:**
  - `/Users/sac/rocket-craft/pwa-staff/Brm-HTML5-Shipping.data`
* **Category:** Cooked Level Data Placeholder
* **File Type:** JSON-formatted Plaintext (`.data`)
* **Exact Evidence:**
  Plain-text JSON array defining place coordinates and meshes (Cube, Cylinder, Sphere).
* **Doctrine Violation Explanation:**
  Real Unreal Engine 4 HTML5 builds output binary data archives (.data files) containing assets packed for virtual file systems. Bypassing asset cooking with plain-text JSON stubs violates **GATE 1 (Unreal Artifact Admission)** (must emit UE4-consumable world artifacts) and **GATE 2 (HTML5/WASM Package Admission)**.

#### D. Hardcoded Spec Fallbacks and Fake Receipts
* **File Paths:**
  - `/Users/sac/rocket-craft/genie_server.js` (Line 93)
  - `/Users/sac/rocket-craft/pwa-staff/test-results/tps-dflss-receipt.json`
* **Category:** Fake Spec/Receipt Generation
* **File Type:** JavaScript (`.js`) / JSON (`.json`)
* **Exact Evidence:**
  In `genie_server.js`, a hardcoded `fallbackSpec` layout is saved to `spec.json` when CLI compilation fails:
  ```javascript
  const fallbackSpec = {
      places: [ ... ],
      actors: [ ... ],
      objects: [ ... ],
      relationships: [ ... ]
  };
  fs.writeFileSync(SPEC_PATH, JSON.stringify(fallbackSpec, null, 2));
  ```
* **Doctrine Violation Explanation:**
  Bypasses the Combinatorial Engine and receipt validation. Producing signed receipts for hardcoded fallback specifications violates the **Law of Combinatorial Maximalism** and **GATE 7 (Receipt Admission)** (PASS only if prompt, contract hash, build log, package path, screenshots, console logs, input trace, visual delta, and final verdict are recorded for the actual generated world). This creates fake proof of success, violating the Core Mandate: *"Victory requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input."*
