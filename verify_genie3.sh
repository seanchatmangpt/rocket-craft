#!/bin/bash
set -e

# Genie 3 Verification Script
# This script builds the genie3-rs crate, runs the integration tests to verify the scenario
# (creating initial state, applying movement actions, applying promptable events, and
# validating state transitions and consistency), and exits with 0 on success.

CWD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "=========================================================="
echo "Starting Genie 3 World Model Verification Scenario..."
echo "Working directory: $CWD"
echo "=========================================================="

# 1. Build the crate
echo "[1/3] Building genie3-rs crate..."
cd "$CWD/genie3-rs"
cargo build

# 2. Describe the verification scenario
echo ""
echo "[2/3] Preparing to run verification scenario..."
echo "Scenario Details:"
echo "  - Create Initial State:"
echo "    * Place: room_1 (Control Room, hard containment enabled)"
echo "    * Actor: bot_1 (Welder Bot, starting at origin)"
echo "    * Object: cnc_1 (CNC Machine at (10, 10, 0))"
echo "  - Apply Movement Action:"
echo "    * Move bot_1 by (+5, +5, 0)"
echo "    * Assert state transition and placement updates"
echo "  - Apply Promptable Events:"
echo "    * Event A: Change weather to Stormy, time of day to 20:00 (evening)"
echo "    * Event B: Spawn new barrier object 'barrier_1' at (5, 10, 0)"
echo "  - Validate Consistency & Safety Constraints:"
echo "    * Assert that bot_1 cannot move into the spawned barrier (collision avoidance)"
echo "    * Assert that bot_1 cannot teleport (speed limits)"
echo "    * Assert that bot_1 cannot exit room_1 (hard containment)"
echo "    * Validate referential coherence across all entities"
echo ""

# 3. Run the scenario tests
echo "[3/3] Running integration test suite..."
cargo test --test integration_tests -- --nocapture

echo ""
echo "=========================================================="
echo "SUCCESS: Genie 3 World Model scenario executed successfully!"
echo "All state transitions and consistency assertions passed."
echo "=========================================================="
exit 0
