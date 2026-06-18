# TPS/DfLSS Playwright Manufacturing Strategy
**DO NOT TREAT THIS AS DOCUMENTATION. TREAT IT AS LAW.**

Refactor Rocket-Craft so the final authority is Playwright visual verification of a browser-native Unreal 4 HTML5/WASM world built with the SpeculativeCoder UE4.27 HTML5 ES3 fork.

The pipeline must not accept:
- Rust-only simulation
- CLI emulation
- mocked worlds
- unit-test-only success
- package-only success
- compile-only success
- screenshots without input actuation
- input actuation without visual delta
- visual delta without receipt

The accepted crown path is:

Prompt
→ Rocket-Craft Contract
→ Unreal 4 world artifact
→ HTML5/WASM package
→ local browser launch
→ Playwright waits for engine readiness
→ Playwright captures baseline screenshot
→ Playwright sends movement input
→ Playwright captures after screenshot
→ visual delta is computed
→ browser console logs are captured
→ cryptographic receipt is produced

If visual motion delta is below threshold, mark DEFECT and route repair by failure taxonomy.

Victory requires a replayable receipt proving the generated world was visible, loaded, responsive, and moved under input.

## Acceptance Matrix

GATE 0 — Source Admission
PASS only if Rocket-Craft has a declared world contract for the prompt.

GATE 1 — Unreal Artifact Admission
PASS only if Rocket-Craft emits Unreal 4-consumable world artifacts.

GATE 2 — HTML5/WASM Package Admission
PASS only if the SpeculativeCoder UE4.27 HTML5 ES3 build produces browser-deployable output.

GATE 3 — Browser Load Admission
PASS only if Playwright opens the packaged world and detects engine readiness.

GATE 4 — Visual World Admission
PASS only if screenshot shows a non-error WebGL/Unreal scene.

GATE 5 — Actuation Admission
PASS only if keyboard input is injected.

GATE 6 — Motion Admission
PASS only if after-screenshot differs from before-screenshot above threshold.

GATE 7 — Receipt Admission
PASS only if prompt, contract hash, build log, package path, screenshots, console logs, input trace, visual delta, and final verdict are recorded.

## The Repair Routing Law

No generic repair loop.

Every failure must route to a cell:
- UE4 fork/build cell
- HTML5 packaging cell
- Rocket-Craft contract cell
- Unreal artifact generation cell
- local serving cell
- Playwright browser-load cell
- WebGL/runtime cell
- input-binding cell
- visual-delta cell
- receipt/audit cell

## The Command to Agents

Stop proving that code exists.

Prove that the world drives.
