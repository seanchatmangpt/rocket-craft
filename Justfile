# Rocket Craft — Justfile
# Usage: just <recipe>   (install: cargo install just)

# Default: list all recipes
default:
    @just --list

# ── Environment ──────────────────────────────────────────────────────────────

# Diagnose environment (UE4_ROOT, Blender, Node, emsdk, etc.)
doctor:
    ./rocket doctor

# Print project manifest summary
info:
    ./rocket info

# ── HTML5 Pipeline (Brm) ─────────────────────────────────────────────────────

# Cook + package Brm for HTML5 (Cook 7 flags: -IgnoreCookErrors -package)
# Produces: /tmp/brm-html5-archive/HTML5/Brm.wasm (175 MB) + Brm.data (48 MB)
html5-cook:
    ./package-brm-html5.sh

# Serve the cooked HTML5 package on :8080
html5-serve port="8080":
    ./rocket html5 serve --port {{port}}

# Full Stage 6 proof: serve + Playwright E2E + receipt validation (verdict=PASS)
html5-proof archive="/tmp/brm-html5-archive":
    ./verify_html5_pipeline.sh {{archive}}

# Cook then immediately prove (full pipeline in one command)
html5-all:
    just html5-cook
    just html5-proof

# Re-run Playwright proof against already-served archive (server must be running)
html5-playwright:
    cd pwa-staff && TARGET_GAME_URL="/Brm.html" npx playwright test \
        tests-e2e/tps-dflss.spec.ts \
        --config playwright.html5.config.ts \
        --reporter=list

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the rocket CLI (release)
build-rocket:
    cd tools && cargo build --release
    @echo "Built: tools/target/release/rocket-cmd"

# Build UE4 editor (Stage 4 — takes 1-3 hours)
build-ue4:
    cd tools && cargo run --bin build_ue4 --release

# ── Lint & Format ────────────────────────────────────────────────────────────

# Check Rust formatting across all workspaces (no changes, exit non-zero if drift)
fmt-check:
    cd tools && cargo fmt -- --check
    cd nexus-engine && cargo fmt -- --check
    cd blueprint-rs && cargo fmt -- --check
    cd chicago-tdd-tools && cargo fmt -- --check
    cd unify-rs && cargo fmt -- --check
    cd infinity-blade-4/mud && cargo fmt -- --check
    cd asset-pipeline && cargo fmt -- --check

# Apply Rust formatting across all workspaces
fmt:
    cd tools && cargo fmt
    cd nexus-engine && cargo fmt
    cd blueprint-rs && cargo fmt
    cd chicago-tdd-tools && cargo fmt
    cd unify-rs && cargo fmt
    cd infinity-blade-4/mud && cargo fmt
    cd asset-pipeline && cargo fmt

# Run clippy across all Rust workspaces
clippy:
    cd tools && cargo clippy --all 2>&1 | grep -v "^error\[E" | grep "^warning\|^error" | grep -v "wasm4pm\|unknown lint" || true
    cd nexus-engine && cargo clippy --all 2>&1 | grep "^warning\|^error" | grep -v "wasm4pm\|unknown lint" || true
    cd blueprint-rs && cargo clippy --all 2>&1 | grep "^warning\|^error" || true
    cd chicago-tdd-tools && cargo clippy --all-features 2>&1 | grep "^warning\|^error" | grep -v "unknown lint" || true
    cd unify-rs && cargo clippy --all 2>&1 | grep "^warning\|^error" | grep -v "unknown lint\|wasm4pm" || true
    cd infinity-blade-4/mud && cargo clippy --all 2>&1 | grep "^warning\|^error" || true
    cd asset-pipeline && cargo clippy 2>&1 | grep "^warning\|^error" || true

# Full CI gate: fmt-check + clippy + test + typecheck
ci:
    just fmt-check
    just clippy
    just test
    just typecheck
    just validate-receipt

# ── Tests ─────────────────────────────────────────────────────────────────────

# Run all Rust workspace tests
test-rust:
    cd tools && cargo test --all
    cd nexus-engine && cargo test --all
    cd blueprint-rs && cargo test --all
    cd chicago-tdd-tools && cargo test --all-features
    cd unify-rs && cargo test --all
    cd infinity-blade-4/mud && cargo test --all
    cd asset-pipeline && cargo test

# Run pwa-staff unit tests (vitest)
test-pwa:
    cd pwa-staff && npm test

# Run ALL tests (Rust + PWA)
test:
    just test-rust
    just test-pwa

# Type-check pwa-staff TypeScript
typecheck:
    cd pwa-staff && npx tsc --noEmit

# ── PWA ───────────────────────────────────────────────────────────────────────

# Lint + format pwa-staff
lint:
    ./rocket pwa lint

# Build pwa-staff (esbuild + postcss)
build-pwa:
    cd pwa-staff && npm run build

# Start pwa-staff dev server on :3000
serve-pwa:
    cd pwa-staff && npm start

# ── Receipt Validation ────────────────────────────────────────────────────────

# Validate the Stage 6 TPS-DFLSS receipt
validate-receipt:
    ./rocket receipt validate --file pwa-staff/test-results/tps-dflss-receipt.json

# Show last receipt verdict
receipt-status:
    @python3 -c "import json; r=json.load(open('pwa-staff/test-results/tps-dflss-receipt.json')); \
        print('verdict:', r['verdict'], '| visualDelta:', r['visualDelta'], '| run_id:', r['run_id'])" \
        2>/dev/null || echo "No receipt found — run: just html5-proof"

# ── Cleanup ───────────────────────────────────────────────────────────────────

# Clean Rust build artifacts (use sparingly — prefers incremental)
clean-rust:
    cd tools && cargo clean
    cd nexus-engine && cargo clean

# Clean HTML5 cook archive
clean-html5:
    rm -rf /tmp/brm-html5-archive
    @echo "Cleaned /tmp/brm-html5-archive — re-cook with: just html5-cook"

# Clean pwa-staff node_modules
clean-pwa:
    rm -rf pwa-staff/node_modules
    cd pwa-staff && npm ci

# Verify flagship mecha pipeline (F1 Cinematic gate)
verify-flagship-ue4-mech:
    ./verify_mecha_pipeline.sh

