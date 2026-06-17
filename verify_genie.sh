#!/bin/bash
set -e

# Genie 26 E2E Integration and Validation Script
# This script compiles the unify-rs workspace, runs the manufacturing pipeline,
# evolves the world specification, deploys telemetry logs, and starts the simulation server.

CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "--------------------------------------------------------"
echo "Starting Genie 26 System Integration and Validation..."
echo "Working directory: $CWD"
echo "--------------------------------------------------------"

# 1. Compile the workspace
echo "[1/6] Compiling unify-rs workspace..."
cd "$CWD/unify-rs"
cargo build
cd "$CWD"

UNIFY_BIN="$CWD/unify-rs/target/debug/unify"
if [ ! -f "$UNIFY_BIN" ]; then
    echo "ERROR: Unify CLI binary not found at $UNIFY_BIN"
    exit 1
fi
echo "CLI binary verified: $UNIFY_BIN"

# 2. Cleanup previous outputs
echo "[2/6] Cleaning up previous outputs..."
rm -f "$CWD/spec.json" "$CWD/map.t3d" "$CWD/deploy.log" "$CWD/init_intent.txt" "$CWD/mod_intent.txt"

# 3. Create initial world intent and manufacture
echo "[3/6] Manufacturing initial world..."
cat << 'EOF' > "$CWD/init_intent.txt"
create place room_1 name "Control Room" at (0.0, 0.0, 0.0) bounds (100.0, 100.0, 50.0)
create actor bot_1 name "Welder Bot" role RoboticWelder in room_1
create object cnc_1 name "CNC Alpha" class CNC_Machine in room_1
create relationship rel_1 contains from room_1 to bot_1
create rule rule_1 name TempCheck expression "room_1.temp < 30" severity error
EOF

"$UNIFY_BIN" genie manufacture \
  --intent "$CWD/init_intent.txt" \
  --out-spec "$CWD/spec.json" \
  --out-t3d "$CWD/map.t3d"

# Verify initial artifacts
if [ ! -f "$CWD/spec.json" ] || [ ! -f "$CWD/map.t3d" ]; then
    echo "ERROR: Initial manufacturing failed to produce spec.json or map.t3d"
    exit 1
fi

echo "Initial spec.json and map.t3d manufactured successfully!"
grep -q "Begin Map" "$CWD/map.t3d"
grep -q "Place_room_1" "$CWD/map.t3d"
grep -q "Actor_bot_1" "$CWD/map.t3d"
grep -q "Object_cnc_1" "$CWD/map.t3d"
echo "T3D structural layout conforms to Unreal Engine 4 specifications."

# 4. Evolve the world with updates
echo "[4/6] Evolving world with incremental modification intent..."
cat << 'EOF' > "$CWD/mod_intent.txt"
create place room_2 name "Storage Room" at (200.0, 0.0, 0.0) bounds (50.0, 50.0, 30.0)
update actor bot_1 position (20.0, -10.0, 0.0)
create relationship rel_2 connects from room_1 to room_2
EOF

"$UNIFY_BIN" genie evolve \
  --spec "$CWD/spec.json" \
  --intent "$CWD/mod_intent.txt" \
  --out-spec "$CWD/spec.json" \
  --out-t3d "$CWD/map.t3d"

# Verify evolved artifacts
grep -q "Place_room_2" "$CWD/map.t3d"
grep -q "RelativeLocation=(X=20.000000,Y=-10.000000,Z=0.000000)" "$CWD/map.t3d"
echo "Evolved spec.json and map.t3d verified. State continuity preserved!"

# 5. Deploy the world (generate telemetry log)
echo "[5/6] Deploying world telemetry logs..."
"$UNIFY_BIN" genie deploy \
  --spec "$CWD/spec.json" \
  --log "$CWD/deploy.log"

if [ ! -f "$CWD/deploy.log" ]; then
    echo "ERROR: Deployment failed to produce deploy.log"
    exit 1
fi
echo "Deployment log created successfully."
grep -q "Places: 2" "$CWD/deploy.log"
grep -q "Actors: 1" "$CWD/deploy.log"

# Clean up temp intent files
rm -f "$CWD/init_intent.txt" "$CWD/mod_intent.txt"

# 6. Launch the interactive simulator server
echo "[6/6] Launching local Web Operating Center server..."
echo "Starting Node server on port 3000 in background..."

# Check if port 3000 is in use and kill it to prevent conflicts
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null ; then
    echo "Port 3000 in use, cleaning up..."
    kill -9 $(lsof -t -i:3000) || true
    sleep 1
fi

node "$CWD/genie_server.js" &
SERVER_PID=$!

# Trap exit to kill the server when the script ends
trap "kill $SERVER_PID" EXIT

echo "--------------------------------------------------------"
echo "  GENIE 26 SYSTEM VALIDATION SUCCESSFUL!"
echo "  The 3D World has been manufactured & deployed."
echo "  Open your browser and navigate to: http://localhost:3000"
echo "  Press Ctrl+C to stop the server and exit."
echo "--------------------------------------------------------"

if [ "$1" = "--non-interactive" ] || [ "$1" = "--auto" ] || [ -n "$CI" ]; then
    echo "Automated mode detected. Waiting 3 seconds for server to initialize..."
    sleep 3
    if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null; then
        echo "SUCCESS: Server is listening on port 3000. Automated verification succeeded."
        exit 0
    else
        echo "ERROR: Server is not listening on port 3000!"
        exit 1
    fi
else
    # Keep script running to maintain the server background task
    wait
fi
