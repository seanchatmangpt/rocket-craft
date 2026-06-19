#!/bin/bash

# Exit on command failure unless handled
set -eo pipefail

MANIFEST_PATH="/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"
CORE_TTL_PATH="/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH="/tmp/core.ttl.bak"
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

# Setup initial backup
setup

# trap cleanup EXIT

# Run baseline sync and verify it passes
echo "Running baseline validation..."
if ! "$GGEN_BIN" sync --manifest "$MANIFEST_PATH" --validate-only true > /dev/null 2>&1; then
    echo "FAIL: Baseline validation failed!"
    exit 1
fi
echo "PASS: Baseline validation passed."

TOTAL_TESTS=27
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
        return 1
    fi
}

# 1. RuleA (Pin Connection Direction)
restore
echo "gundam:MoveForwardPinIn ue4:connectedTo gundam:MoveForwardDirPin ." >> "$CORE_TTL_PATH"
run_test_case "RuleA (Pin Connection Direction)" "RuleA"

# 2. RuleB (Graph Isolation Check)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"

gundam:OtherGraph a ue4:UEdGraph ;
    rdfs:label "OtherGraph" ;
    rdfs:comment "A separate unconnected graph." .

gundam:OtherNode a ue4:UEdGraphNode ;
    rdfs:label "OtherNode" ;
    rdfs:comment "A node in the other graph." ;
    ue4:nodeOf gundam:OtherGraph ;
    ue4:hasPin gundam:OtherPin .

gundam:OtherPin a ue4:UEdGraphPin ;
    rdfs:label "OtherPin" ;
    rdfs:comment "Pin in other graph." ;
    ue4:pinOf gundam:OtherNode ;
    ue4:pinDirection ue4:Input ;
    ue4:pinCategory "exec" .

gundam:W_KeyPressedPinOut ue4:connectedTo gundam:OtherPin .
EOF
run_test_case "RuleB (Graph Isolation Check)" "RuleB"

# 3. RuleC (Parameter Mapping Integrity)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"

ue4:SomeOtherFunction a ue4:UFunction ;
    rdfs:label "SomeOtherFunction" ;
    rdfs:comment "Function not called by the node." .

ue4:SomeOtherParam a ue4:UFunctionParameter ;
    rdfs:label "SomeOtherParam" ;
    ue4:parameterOf ue4:SomeOtherFunction ;
    ue4:parameterDirection ue4:Input ;
    ue4:parameterIndex 0 .
EOF
content=$(cat "$CORE_TTL_PATH")
search="ue4:mapsToParameter ue4:WorldDirectionParam"
replace="ue4:mapsToParameter ue4:SomeOtherParam"
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "RuleC (Parameter Mapping Integrity)" "RuleC"

# 4. RuleD (Pin Parameter Direction Match)
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:parameterDirection ue4:Input ;
    ue4:parameterIndex 0 ."
replace="ue4:parameterDirection ue4:Output ;
    ue4:parameterIndex 0 ."
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "RuleD (Pin Parameter Direction Match)" "RuleD"

# 5. RuleE (Exec vs. Data Pin Separation)
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:pinCategory \"exec\" ;
    ue4:connectedTo gundam:W_KeyPressedPinOut"
replace="ue4:pinCategory \"float\" ;
    ue4:connectedTo gundam:W_KeyPressedPinOut"
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "RuleE (Exec vs. Data Pin Separation)" "RuleE"

# 6. RuleF (Character Cooking State)
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:hasCookingState gundam:Cooked"
replace="rdfs:comment \"Removed cooking state\""
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "RuleF (Character Cooking State)" "RuleF"

# 7. RuleG (World Packaging State)
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:hasPackagingState gundam:WasmReady"
replace="rdfs:comment \"Removed packaging state\""
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "RuleG (World Packaging State)" "RuleG"

# 8. RuleH (Dangling Execution Flow)
restore
content=$(cat "$CORE_TTL_PATH")
search1="ue4:connectedTo gundam:MoveForwardPinIn"
replace1="rdfs:comment \"disconnected output exec\""
content="${content/$search1/$replace1}"
search2="ue4:connectedTo gundam:W_KeyPressedPinOut"
replace2="rdfs:comment \"disconnected input exec\""
content="${content/$search2/$replace2}"
echo "$content" > "$CORE_TTL_PATH"
run_test_case "RuleH (Dangling Execution Flow)" "RuleH"

# 9. RuleLabel (Class Label)
restore
echo -e "\ngundam:NoLabelClass a owl:Class .\n" >> "$CORE_TTL_PATH"
run_test_case "RuleLabel (Class Label)" "RuleLabel"

# 10. RuleNamespace (Namespace Sanity)
restore
echo -e "\n<urn:private:bad-namespace> a owl:Class ; rdfs:label \"BadNamespace\" .\n" >> "$CORE_TTL_PATH"
run_test_case "RuleNamespace (Namespace Sanity)" "RuleNamespace"

# 11. SHACL Pin Ownership
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:pinOf gundam:MoveForwardCallNode ;"
replace="# removed pinOf"
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "SHACL Pin Ownership" "A pin must belong to exactly one UEdGraphNode"

# 12. SHACL Input Pin Connection Count Limit
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:OtherKeyPressedPinOut a ue4:UEdGraphPin ;
    rdfs:label "OtherKeyPressedPinOut" ;
    ue4:pinOf gundam:W_KeyPressedNode ;
    ue4:pinDirection ue4:Output ;
    ue4:pinCategory "exec" .

gundam:MoveForwardPinIn ue4:connectedTo gundam:OtherKeyPressedPinOut .
EOF
run_test_case "SHACL Input Pin Connection Count Limit" "Input pin connection count limit"

# 13. SHACL Pin Category Limit
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:pinCategory \"float\" ;"
replace="ue4:pinCategory \"invalid_pin_category_name\" ;"
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "SHACL Pin Category Limit" "limited to standard categories"

# 14. SHACL Variable Node Property Check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadVarNode a ue4:UK2Node_VariableGet ;
    rdfs:label "BadVarNode" ;
    rdfs:comment "Variable node without referencedProperty." ;
    ue4:nodeOf gundam:GundamInputGraph .
EOF
run_test_case "SHACL Variable Node Property Check" "A variable getter or setter node must reference exactly one valid UProperty"

# 15. SHACL UEdGraphNode Parentage Check
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:nodeOf gundam:GundamInputGraph ;"
replace="# removed nodeOf"
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "SHACL UEdGraphNode Parentage Check" "A node must belong to exactly one UEdGraph"

# 16. SHACL Parameter Index check (minInclusive 0)
restore
content=$(cat "$CORE_TTL_PATH")
search="ue4:parameterIndex 0 ."
replace="ue4:parameterIndex -1 ."
echo "${content/$search/$replace}" > "$CORE_TTL_PATH"
run_test_case "SHACL Parameter Index check (minInclusive 0)" "non-negative integer"

# 17. SHACL Active asset without HTML5 representation (AssetHTML5CookingReadyShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamTexture a ue4:UTexture ;
    rdfs:label "GundamTexture" ;
    rdfs:comment "A texture asset in the persistent world." .
EOF
run_test_case "SHACL Active asset cooking ready" "RuleAssetHTML5CookingReady"

# 18. SHACL WebGL Texture Format Compliance (HTML5TextureFormatShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamTexture a ue4:UTexture ;
    rdfs:label "GundamTexture" ;
    rdfs:comment "A texture asset." .

gundam:GundamTextureRep a ue4:AssetPlatformRepresentation ;
    rdfs:label "GundamTextureRep" ;
    ue4:hasAsset gundam:GundamTexture ;
    ue4:targetPlatform ue4:Platform_HTML5 ;
    ue4:hasCookingState ue4:CookState_Cooked ;
    ue4:hasCompressionProfile gundam:GundamTextureProfile .

gundam:GundamTextureProfile a ue4:TextureCompressionProfile ;
    rdfs:label "GundamTextureProfile" ;
    ue4:textureFormat ue4:TexFormat_BC7 .
EOF
run_test_case "SHACL WebGL Texture Format compliance" "RuleHTML5TextureFormat"

# 19. SHACL WASM Memory Initial Memory alignment check (WasmMemoryLayoutShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadMemoryLayout a ue4:WasmMemoryLayout ;
    rdfs:label "BadMemoryLayout" ;
    ue4:wasmStackSize 65536 ;
    ue4:wasmInitialMemory 50000000 ; # Not aligned to 65536
    ue4:wasmMaximumMemory 1073741824 ;
    ue4:wasmAllowMemoryGrowth true ;
    ue4:wasmExportedSymbol "_main" .
EOF
run_test_case "SHACL WASM Initial Memory page alignment check" "RuleWasmMemoryLayoutPageAlignment"

# 20. SHACL WASM Memory AllowMemoryGrowth constraint check (WasmMemoryLayoutShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:BadMemoryLayout a ue4:WasmMemoryLayout ;
    rdfs:label "BadMemoryLayout" ;
    ue4:wasmStackSize 65536 ;
    ue4:wasmInitialMemory 67108864 ; # 64MB (aligned)
    ue4:wasmMaximumMemory 134217728 ; # 128MB (aligned)
    ue4:wasmAllowMemoryGrowth false ; # Fixed heap but different initial/max memory
    ue4:wasmExportedSymbol "_main" .
EOF
run_test_case "SHACL WASM Fixed Heap bounds check" "RuleWasmMemoryBoundaries"

# 21. SHACL Static Baking Paths Check (StaticBakingPathsShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:StaticBakeConfig a ue4:StaticBakingConfiguration ;
    rdfs:label "StaticBakeConfig" ;
    ue4:isStaticallyBaked true ;
    ue4:headerOutputPath "Source/Generated/Headers" . # Missing other paths
EOF
run_test_case "SHACL Static Baking Paths check" "RuleStaticBakingPaths"

# 22. SHACL Static Baking VaRest Prohibition (StaticBakingNoVaRestShape)
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:StaticBakeConfig a ue4:StaticBakingConfiguration ;
    rdfs:label "StaticBakeConfig" ;
    ue4:isStaticallyBaked true ;
    ue4:headerOutputPath "Source/Headers" ;
    ue4:dataTableOutputPath "Content/DataTables" ;
    ue4:bomOutputPath "Build/BOM" ;
    ue4:walkthroughOutputPath "Tests/walkthrough.json" ;
    ue4:byteClassMatrixOutputPath "Build/Matrices" ;
    ue4:receiptOutputPath "Build/receipt.json" .

gundam:StaticBakingTarget a ue4:PackagingTarget ;
    rdfs:label "StaticBakingTarget" ;
    ue4:targetWorld gundam:GundamWorld ;
    ue4:buildConfiguration ue4:Config_Development ;
    ue4:targetRHIProfile ue4:WebGL2_RHI_Profile ;
    ue4:targetPlatformName "HTML5" ;
    ue4:hasStaticBaking gundam:StaticBakeConfig .

# Node calling VaRest inside GundamInputGraph
gundam:VaRestCallNode a ue4:UEdGraphNode ;
    rdfs:label "VaRestCallNode" ;
    ue4:nodeOf gundam:GundamInputGraph ;
    ue4:callsFunction <https://rocket-craft.io/ontology/ue4/VaRest_Call_Function> .
EOF
run_test_case "SHACL Static Baking VaRest Prohibition check" "RuleStaticBakingNoVaRest"

# 23. SHACL Material Instance Parameter Value Type Safety check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamBaseMaterial a ue4:UMaterial ;
    rdfs:label "GundamBaseMaterial" ;
    ue4:definesParameter gundam:GundamScalarParam .

gundam:GundamScalarParam a ue4:UScalarParameter ;
    rdfs:label "GundamScalarParam" .

gundam:GundamMaterialInstance a ue4:UMaterialInstance ;
    rdfs:label "GundamMaterialInstance" ;
    ue4:parentMaterial gundam:GundamBaseMaterial ;
    ue4:hasParameterValue gundam:GundamParamVal .

gundam:GundamParamVal a ue4:UMaterialParameterValue ;
    rdfs:label "GundamParamVal" ;
    ue4:parameterName "GundamScalarParam" ;
    ue4:vectorValue "(R=1.0,G=0.0,B=0.0,A=1.0)" .
EOF
run_test_case "SHACL Material Instance Parameter Value Type Safety check" "RuleMaterialInstanceParameterValueType"

# 24. SHACL Unregistered Collision Profile Usage check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamUnregisteredProfile a ue4:UCollisionProfile ;
    rdfs:label "GundamUnregisteredProfile" ;
    ue4:profileName "UnregisteredProfile" ;
    ue4:collisionEnabled ue4:QueryAndPhysics ;
    ue4:collisionObjectType ue4:ECC_Pawn .

gundam:GundamCollision ue4:hasCollisionProfile gundam:GundamUnregisteredProfile .
EOF
run_test_case "SHACL Unregistered Collision Profile Usage check" "RuleComponentCollisionProfileRegistration"

# 25. SHACL Server RPC missing validation check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamServerRPC a ue4:UServerRPC ;
    rdfs:label "GundamServerRPC" .

gundam:AGundamCharacter ue4:hasFunction gundam:GundamServerRPC .
EOF
run_test_case "SHACL Server RPC missing validation check" "RuleServerRPCValidationMandatory"

# 26. SHACL RPC validation function class scope violation check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:AMyNPC a owl:Class ; rdfs:label "AMyNPC" ; rdfs:subClassOf ue4:AActor .

gundam:GundamServerRPCForScope a ue4:UServerRPC ;
    rdfs:label "GundamServerRPCForScope" ;
    ue4:bWithValidation true ;
    ue4:validationFunction gundam:GundamValidationFuncWrongScope .

gundam:GundamValidationFuncWrongScope a ue4:UFunction ;
    rdfs:label "GundamValidationFuncWrongScope" ;
    ue4:returnProperty gundam:GundamValidationFuncWrongScopeRet .

gundam:GundamValidationFuncWrongScopeRet a ue4:UBoolProperty ;
    rdfs:label "GundamValidationFuncWrongScopeRet" .

gundam:AGundamCharacter ue4:hasFunction gundam:GundamServerRPCForScope .
gundam:AMyNPC ue4:hasFunction gundam:GundamValidationFuncWrongScope .
EOF
run_test_case "SHACL RPC validation function class scope check" "RuleRPCValidationClassScope"

# 27. SHACL Kinematic Simulation Disconnect check
restore
cat << 'EOF' >> "$CORE_TTL_PATH"
gundam:GundamBodyKinematic a ue4:URigidBody ;
    rdfs:label "GundamBodyKinematic" ;
    ue4:physicsType ue4:PhysType_Kinematic .

gundam:GundamComponentKinematicSim a ue4:UBoxComponent ;
    rdfs:label "GundamComponentKinematicSim" ;
    ue4:bSimulatePhysics true ;
    ue4:hasRigidBody gundam:GundamBodyKinematic .
EOF
run_test_case "SHACL Kinematic Simulation Disconnect check" "RuleKinematicSimulationDisconnect"


# Final result reporting
echo ""
if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo "ALL CODES AND CONSTRAINTS SUCCESSFULLY VERIFIED!"
    cleanup
    exit 0
else
    echo "SOME VERIFICATIONS FAILED! ($PASSED_TESTS / $TOTAL_TESTS passed)"
    cleanup
    exit 2
fi
