#!/bin/bash
# verify_html5_pipeline.sh
# Automated end-to-end HTML5 pipeline and E2E Playwright verification script.

set -e

CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "--------------------------------------------------------"
echo "Starting HTML5 Pipeline Verification..."
echo "Working directory: $CWD"
echo "--------------------------------------------------------"

# 1. Clean previous build outputs
echo "[1/8] Cleaning previous build outputs..."
rm -f "$CWD/spec.json" "$CWD/map.t3d" "$CWD/deploy.log" "$CWD/init_intent.txt" "$CWD/mod_intent.txt"
rm -rf "$CWD/pwa-staff/manufactured"
rm -f "$CWD/pwa-staff/test-results/tps-dflss-receipt.json" "$CWD/pwa-staff/test-results/tps-dflss-diff.png"

# 2. Set UE4_ROOT
echo "[2/8] Setting UE4_ROOT..."
export UE4_ROOT=/Users/sac/ue4-sim
echo "UE4_ROOT is set to: $UE4_ROOT"

# 3. Compile the backend (unify-rs)
echo "[3/8] Compiling backend (unify-rs)..."
cd "$CWD/unify-rs"
cargo build
cd "$CWD"

UNIFY_BIN="$CWD/unify-rs/target/debug/unify"
if [ ! -f "$UNIFY_BIN" ]; then
    echo "ERROR: Unify CLI binary not found at $UNIFY_BIN"
    exit 1
fi

# 4. Trigger world manufacture, evolution, and simulated UE4 packaging pipeline
echo "[4/8] Triggering world manufacture and evolution..."

# Create initial world intent
cat << 'EOF' > "$CWD/init_intent.txt"
create place zone_1 name "Primitive Foundry" at (0.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_2 name "Part Runner Wall" at (400.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_3 name "Assembly Gantry" at (800.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_4 name "Fit + Collision Bay" at (1200.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_5 name "Physics Proving Ground" at (1600.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create place zone_6 name "Final Reveal Platform" at (2000.0, 0.0, 0.0) bounds (150.0, 150.0, 50.0)
create relationship rel_1_2 connects from zone_1 to zone_2
create relationship rel_2_3 connects from zone_2 to zone_3
create relationship rel_3_4 connects from zone_3 to zone_4
create relationship rel_4_5 connects from zone_4 to zone_5
create relationship rel_5_6 connects from zone_5 to zone_6
EOF

# Manufacture
"$UNIFY_BIN" genie manufacture \
  --intent "$CWD/init_intent.txt" \
  --out-spec "$CWD/spec.json" \
  --out-t3d "$CWD/map.t3d"

# Create evolution intent
cat << 'EOF' > "$CWD/mod_intent.txt"
create actor bot_1 name "Welder Bot" role RoboticWelder in zone_1
create place zone_extra name "Extra Lab" at (2400.0, 0.0, 0.0) bounds (100.0, 100.0, 50.0)
create relationship rel_6_extra connects from zone_6 to zone_extra
EOF

# Evolve
"$UNIFY_BIN" genie evolve \
  --spec "$CWD/spec.json" \
  --intent "$CWD/mod_intent.txt" \
  --out-spec "$CWD/spec.json" \
  --out-t3d "$CWD/map.t3d"

# Deploy (triggers UE4 simulation build/package pipeline and copies HTML5 package to served directory)
echo "Deploying world layout and running simulated UE4 pipeline..."
"$UNIFY_BIN" genie deploy \
  --spec "$CWD/spec.json" \
  --log "$CWD/deploy.log"

# Clean up temp intent files
rm -f "$CWD/init_intent.txt" "$CWD/mod_intent.txt"

# 5. Start the local web server on port 3000
echo "[5/8] Starting local web server on port 3000..."
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null ; then
    echo "Port 3000 in use, cleaning up..."
    kill -9 $(lsof -t -i:3000) || true
    sleep 1
fi

node "$CWD/genie_server.js" &
SERVER_PID=$!

# Wait for server to initialize
sleep 3

# 6. Run the Playwright test
echo "[6/8] Running Playwright E2E test..."
cd "$CWD/pwa-staff"
set +e
npx playwright test tests-e2e/tps-dflss.spec.ts
PLAYWRIGHT_EXIT=$?
set -e
cd "$CWD"

# 7. Validate that tps-dflss-receipt.json is generated successfully with verdict PASS and all fields present
echo "[7/8] Validating generated receipt..."
set +e
node -e '
const fs = require("fs");
const path = require("path");
const receiptPath = path.join("pwa-staff", "test-results", "tps-dflss-receipt.json");

if (!fs.existsSync(receiptPath)) {
  console.error("Error: Receipt file tps-dflss-receipt.json was not generated!");
  process.exit(1);
}

const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
const requiredFields = [
  "prompt", "contractHash", "buildLog", "packagePath",
  "browserUrl", "screenshots", "consoleLogs", "inputTrace",
  "visualDelta", "verdict"
];

for (const field of requiredFields) {
  if (receipt[field] === undefined) {
    console.error(`Error: Missing required field "${field}" in receipt!`);
    process.exit(1);
  }
}

if (receipt.verdict !== "PASS") {
  console.error(`Error: Expected verdict "PASS", got "${receipt.verdict}"`);
  process.exit(1);
}

if (!receipt.screenshots || receipt.screenshots.before === undefined || receipt.screenshots.after === undefined) {
  console.error("Error: Screenshots must contain before and after base64 strings");
  process.exit(1);
}

console.log("Success: Receipt validation passed successfully!");
process.exit(0);
'
VALIDATION_EXIT=$?
set -e

# 8. Shut down the local web server
echo "[8/8] Shutting down local web server..."
kill -9 $SERVER_PID || true

if [ $PLAYWRIGHT_EXIT -eq 0 ] && [ $VALIDATION_EXIT -eq 0 ]; then
    echo "--------------------------------------------------------"
    echo "  E2E HTML5 PIPELINE VERIFICATION SUCCESSFUL!"
    echo "--------------------------------------------------------"
    exit 0
else
    echo "--------------------------------------------------------"
    echo "  E2E HTML5 PIPELINE VERIFICATION FAILED!"
    echo "  Playwright Exit Code: $PLAYWRIGHT_EXIT"
    echo "  Validation Exit Code: $VALIDATION_EXIT"
    echo "--------------------------------------------------------"
    exit 1
fi
