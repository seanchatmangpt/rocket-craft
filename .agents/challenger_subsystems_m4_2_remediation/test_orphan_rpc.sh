#!/bin/bash

MANIFEST_PATH="/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
CORE_TTL_PATH="/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH="/tmp/core_orphan_rpc.ttl.bak"
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

# 1. Orphaned RPC (no hasFunction relationship to any class)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
mecha:AMyNPC a owl:Class ; rdfs:label "AMyNPC" ; rdfs:subClassOf ue4:AActor .

mecha:MechaServerRPCOrphaned a ue4:UServerRPC ;
    rdfs:label "MechaServerRPCOrphaned" ;
    ue4:bWithValidation true ;
    ue4:validationFunction mecha:MechaValidationFuncWrongScope .

mecha:MechaValidationFuncWrongScope a ue4:UFunction ;
    rdfs:label "MechaValidationFuncWrongScope" ;
    ue4:returnProperty mecha:MechaValidationFuncWrongScopeRet .

mecha:MechaValidationFuncWrongScopeRet a ue4:UBoolProperty ;
    rdfs:label "MechaValidationFuncWrongScopeRet" .

# We intentionally do NOT define AMechaCharacter hasFunction MechaServerRPCOrphaned
mecha:AMyNPC ue4:hasFunction mecha:MechaValidationFuncWrongScope .
EOF

echo "Running validation with orphaned RPC..."
set +e
OUTPUT=$("$GGEN_BIN" sync --manifest "$MANIFEST_PATH" --validate-only true 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -eq 0 ]; then
    echo "GAP DETECTED: Orphaned RPC with mismatched validation scope successfully bypassed all validation checks!"
else
    echo "Validation failed as expected (or failed due to another rule). Output:"
    echo "$OUTPUT"
fi

cleanup
