#!/bin/bash

MANIFEST_PATH="/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
CORE_TTL_PATH="/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH="/tmp/core_varest_fp.ttl.bak"
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

# 3. False Positive: Non-static world using VaRest, alongside a static world
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
# World A: Statically Baked
mecha:WorldA a ue4:UWorld ; rdfs:label "WorldA" .

mecha:StaticBakeConfigA a ue4:StaticBakingConfiguration ;
    rdfs:label "StaticBakeConfigA" ;
    ue4:isStaticallyBaked true ;
    ue4:headerOutputPath "Source/Headers" ;
    ue4:dataTableOutputPath "Content/DataTables" ;
    ue4:bomOutputPath "Build/BOM" ;
    ue4:walkthroughOutputPath "Tests/walkthrough.json" ;
    ue4:byteClassMatrixOutputPath "Build/Matrices" ;
    ue4:receiptOutputPath "Build/receipt.json" .

mecha:TargetA a ue4:PackagingTarget ;
    rdfs:label "TargetA" ;
    ue4:targetWorld mecha:WorldA ;
    ue4:buildConfiguration ue4:Config_Development ;
    ue4:targetRHIProfile ue4:WebGL2_RHI_Profile ;
    ue4:targetPlatformName "HTML5" ;
    ue4:hasStaticBaking mecha:StaticBakeConfigA .

# World B: Dynamic (Not Statically Baked)
mecha:WorldB a ue4:UWorld ; rdfs:label "WorldB" .

mecha:TargetB a ue4:PackagingTarget ;
    rdfs:label "TargetB" ;
    ue4:targetWorld mecha:WorldB ;
    ue4:buildConfiguration ue4:Config_Development ;
    ue4:targetRHIProfile ue4:WebGL2_RHI_Profile ;
    ue4:targetPlatformName "HTML5" .

# A node in World B calling VaRest
mecha:GraphB a ue4:UEdGraph ;
    rdfs:label "GraphB" ;
    ue4:graphOfWorld mecha:WorldB .

mecha:VaRestCallNodeB a ue4:UEdGraphNode ;
    rdfs:label "VaRestCallNodeB" ;
    ue4:nodeOf mecha:GraphB ;
    ue4:callsFunction <https://rocket-craft.io/ontology/ue4/VaRest_Call_Function> .
EOF

echo "Running validation with non-static world calling VaRest alongside static world..."
set +e
OUTPUT=$("$GGEN_BIN" sync --manifest "$MANIFEST_PATH" --validate-only true 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -eq 0 ]; then
    echo "Validation succeeded (No false positive)."
else
    echo "FALSE POSITIVE CONFIRMED: RuleStaticBakingNoVaRest failed validation! Output:"
    echo "$OUTPUT" | grep -A 2 -i "StaticBakingNoVaRest" || echo "$OUTPUT"
fi

cleanup
