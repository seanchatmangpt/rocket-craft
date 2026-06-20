# Stage 0 Pre-Flight Report

**Date:** 2026-06-18

## Checks

### Disk Space
- **Free:** 268 GB (926 GB total, 70% used)
- **Status:** PASS (≥100 GB required)

### Emscripten (emcc)
- **Found at:** `/opt/homebrew/bin/emcc`
- **Version:** emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 5.0.2-git
- **Status:** PASS

### Fake Engine ~/ue4-sim
- **Status:** EXISTS — contains `Engine/` and `Saved/` directories
- **Action required:** DELETE after Stage 5 (replaced by real engine clone)

### WASM Files

#### pwa-staff/ (top-level stubs)
| File | Size | Notes |
|------|------|-------|
| Brm-HTML5-Shipping.wasm | 1,614,152 bytes | Real-ish (has valid WASM magic header `\0asm`) |
| FullSpectrum-HTML5-Shipping.wasm | 8 bytes | STUB (magic header only) |
| RealisticRendering-HTML5-Shipping.wasm | 8 bytes | STUB (magic header only) |
| ShooterGame-HTML5-Shipping.wasm | 8 bytes | STUB (magic header only) |
| SurvivalGame-HTML5-Shipping.wasm | 8 bytes | STUB (magic header only) |

#### pwa-staff/manufactured/ (Brm build artifacts)
| File | Size |
|------|------|
| Brm-HTML5-Shipping.wasm | 7,011 bytes (stub — will be replaced by real build) |
| Brm-HTML5-Shipping.js | 34,849 bytes |
| Brm-HTML5-Shipping.data | 2,836 bytes |
| Brm-HTML5-Shipping.html | 2,975 bytes |
| receipt.json | 179 bytes |
| spec.json | 4,613 bytes |

All WASM files are stubs. The manufactured/Brm-HTML5-Shipping.wasm (7 KB) confirms a stub build was manufactured previously — it will be replaced by a real UE4 HTML5 compile.

### Engine Clone Status
- **~/ue-4.27-html5-es3/:** NOT STARTED — directory does not exist
- **Action required:** Clone required before Stage 1 UE4 builds can proceed

---

## READY FOR STAGE 1: NO

**Reason:** The real UE4 HTML5 engine (`~/ue-4.27-html5-es3/`) has not been cloned yet. All other pre-conditions pass:
- Disk space is sufficient (268 GB free)
- Emscripten 5.0.2 is installed
- Fake engine stub at `~/ue4-sim/` exists and can be used for dry-run testing until the real engine is available

**Next step:** Clone `ue-4.27-html5-es3` to `~/ue-4.27-html5-es3/` before proceeding to Stage 1.
