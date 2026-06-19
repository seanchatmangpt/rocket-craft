#!/bin/bash

MANIFEST_PATH="/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
CORE_TTL_PATH="/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH="/tmp/core_simulate_no_body.ttl.bak"
GGEN_BIN="/Users/sac/.local/bin/ggen"

setup() {
    if [ -f "$BACKUP_PATH" ]; then
        rm -f "$BACKUP_PATH"
    fi
    cp "$CORE_TTL_PATH" "$BACKUP_PATH"
}

restore() {
    if [ -f "$BACKUP_PATH" ]; then
        cp "$BACKUP_PATH" "$CORE_TTL_PATH"
    fi
}

cleanup() {
    restore
    if [ -f "$BACKUP_PATH" ]; then
        rm -f "$BACKUP_PATH"
    fi
}

setup

# 2. bSimulatePhysics is true but has NO hasRigidBody relation
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamComponentNoBody a ue4:UBoxComponent ;
    rdfs:label "GundamComponentNoBody" ;
    ue4:bSimulatePhysics true .
    # No hasRigidBody relation defined!
EOF

echo "Running validation with simulate physics and no rigid body..."
set +e
OUTPUT=$("$GGEN_BIN" sync --manifest "$MANIFEST_PATH" --validate-only true 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -eq 0 ]; then
    echo "GAP DETECTED: Component with bSimulatePhysics true and no rigid body successfully bypassed all validation checks!"
else
    echo "Validation failed as expected (or failed due to another rule). Output:"
    echo "$OUTPUT"
fi

cleanup
