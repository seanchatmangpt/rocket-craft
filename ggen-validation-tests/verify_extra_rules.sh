#!/bin/bash

# Exit on command failure unless handled
set -eo pipefail

MANIFEST_PATH="/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
CORE_TTL_PATH="/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH="/tmp/core_extra.ttl.bak"
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

TOTAL_TESTS=5
PASSED_TESTS=0

run_test_case() {
    local name="$1"
    local expected_error="$2"
    
    # Run the validation command
    set +e
    OUTPUT=$("$GGEN_BIN" sync --manifest "$MANIFEST_PATH" --validate-only true 2>&1)
    EXIT_CODE=$?
    set -e
    
    # Check if the expected error string is present in the output
    if echo "$OUTPUT" | grep -q "$expected_error"; then
        echo "PASS: $name (Validation failed with expected error: '$expected_error')"
        ((PASSED_TESTS++))
        return 0
    else
        echo "FAIL: $name"
        echo "Expected error pattern: $expected_error"
        echo "Exit Code: $EXIT_CODE"
        echo "Output was:"
        echo "$OUTPUT"
        echo "=== DEBUG INFO ==="
        grep -n pinCategory "$CORE_TTL_PATH"
        tail -n 20 "$CORE_TTL_PATH"
        return 1
    fi
}

# 1. Stack size larger than initial heap
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadMemoryLayoutStackSize a ue4:WasmMemoryLayout ;
    rdfs:label "BadMemoryLayoutStackSize" ;
    ue4:wasmStackSize 131072 ;
    ue4:wasmInitialMemory 65536 ; # Stack larger than Initial Memory
    ue4:wasmMaximumMemory 131072 ;
    ue4:wasmAllowMemoryGrowth true ;
    ue4:wasmExportedSymbol "_main" .
EOF
run_test_case "Stack size larger than initial heap" "WASM Memory boundary mismatch"

# 2. Shipping config using unoptimized build levels (-O0)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamMemoryLayout a ue4:WasmMemoryLayout ;
    rdfs:label "GundamMemoryLayout" ;
    ue4:wasmStackSize 65536 ;
    ue4:wasmInitialMemory 131072 ;
    ue4:wasmMaximumMemory 131072 ;
    ue4:wasmAllowMemoryGrowth false ;
    ue4:wasmExportedSymbol "_main" .

gundam:UnoptimizedOptLevel a ue4:CompilerOptimizationLevel ;
    rdfs:label "UnoptimizedOptLevel" ;
    ue4:optFlag "-O0" .

gundam:BadLinkingConfigShipping a ue4:LinkingConfiguration ;
    rdfs:label "BadLinkingConfigShipping" ;
    ue4:hasMemoryLayout gundam:GundamMemoryLayout ;
    ue4:hasOptimizationLevel gundam:UnoptimizedOptLevel ;
    ue4:buildMode "Shipping" .
EOF
run_test_case "Shipping config using unoptimized build levels (-O0)" "Shipping build optimization violation"

# 3. Shipping config with bOptimize false
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadShippingBuildConfiguration a ue4:BuildConfiguration ;
    rdfs:label "Shipping" ;
    ue4:bOptimize false ;
    ue4:bEnableSymbols false ;
    ue4:bDisableLogging true .
EOF
run_test_case "Shipping config with bOptimize false" "Shipping configuration violation"

# 4. Static baking missing mandated output paths
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadStaticBakeConfig a ue4:StaticBakingConfiguration ;
    rdfs:label "BadStaticBakeConfig" ;
    ue4:isStaticallyBaked true ;
    ue4:headerOutputPath "Source/Generated/Headers" . # Missing other paths
EOF
run_test_case "Static baking missing mandated output paths" "Projection Law violation"

# 5. VaRest dynamic API usage in static configurations
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:StaticBakeConfigVaRest a ue4:StaticBakingConfiguration ;
    rdfs:label "StaticBakeConfigVaRest" ;
    ue4:isStaticallyBaked true ;
    ue4:headerOutputPath "Source/Headers" ;
    ue4:dataTableOutputPath "Content/DataTables" ;
    ue4:bomOutputPath "Build/BOM" ;
    ue4:walkthroughOutputPath "Tests/walkthrough.json" ;
    ue4:byteClassMatrixOutputPath "Build/Matrices" ;
    ue4:receiptOutputPath "Build/receipt.json" .

gundam:StaticBakingTargetVaRest a ue4:PackagingTarget ;
    rdfs:label "StaticBakingTargetVaRest" ;
    ue4:targetWorld gundam:GundamWorld ;
    ue4:buildConfiguration ue4:Config_Development ;
    ue4:targetRHIProfile ue4:WebGL2_RHI_Profile ;
    ue4:targetPlatformName "HTML5" ;
    ue4:hasStaticBaking gundam:StaticBakeConfigVaRest .

gundam:VaRestCallNodeTemp a ue4:UEdGraphNode ;
    rdfs:label "VaRestCallNodeTemp" ;
    ue4:nodeOf gundam:GundamInputGraph ;
    ue4:callsFunction <https://rocket-craft.io/ontology/ue4/VaRest_Call_Function> .
EOF
run_test_case "VaRest dynamic API usage in static configurations" "Statically baked target worlds must not use dynamic VaRest calls"

cleanup
echo "EXTRA VERIFICATIONS COMPLETED: $PASSED_TESTS / $TOTAL_TESTS passed."
if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    exit 0
else
    exit 1
fi
