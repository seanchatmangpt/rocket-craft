#!/usr/bin/env python3
import os
import subprocess
import shutil
import sys

CORE_TTL_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl"
BACKUP_PATH = CORE_TTL_PATH + ".challenge.bak"
GGEN_BIN = "/Users/sac/.local/bin/ggen"
MANIFEST_PATH = "/Users/sac/rocket-craft/ggen-validation-tests/ggen.toml"

# Test cases definition
TEST_CASES = [
    {
        "name": "Challenge Case 1: Material Instance Loop (Acyclicity)",
        "description": "Construct a loop between two material instances.",
        "extra_ttl": """
gundam:MaterialInstLoopA a ue4:UMaterialInstance ;
    rdfs:label "MaterialInstLoopA" ;
    ue4:parentMaterial gundam:MaterialInstLoopB .

gundam:MaterialInstLoopB a ue4:UMaterialInstance ;
    rdfs:label "MaterialInstLoopB" ;
    ue4:parentMaterial gundam:MaterialInstLoopA .
""",
        "expected_error_sparql": "RuleJ",
        "expected_error_shacl": "Material inheritance loop detected"
    },
    {
        "name": "Challenge Case 2: Material Instance Rootedness (No Base Material)",
        "description": "Construct a material instance parent chain that does not end in a base UMaterial.",
        "extra_ttl": """
gundam:MaterialInstOrphan a ue4:UMaterialInstance ;
    rdfs:label "MaterialInstOrphan" ;
    ue4:parentMaterial gundam:MaterialInstParentOrphan .

gundam:MaterialInstParentOrphan a ue4:UMaterialInstance ;
    rdfs:label "MaterialInstParentOrphan" .
""",
        "expected_error_sparql": "RuleK",
        "expected_error_shacl": "Root material missing"
    },
    {
        "name": "Challenge Case 3: Negative Index check",
        "description": "Introduce a parameter with index -1.",
        "extra_ttl": """
ue4:NegativeParam a ue4:UFunctionParameter ;
    rdfs:label "NegativeParam" ;
    ue4:parameterOf ue4:AddMovementInput ;
    ue4:parameterDirection ue4:Input ;
    ue4:parameterIndex -1 .
""",
        "expected_error_sparql": "RuleParameterIndex",
        "expected_error_shacl": "non-negative integer"
    },
    {
        "name": "Challenge Case 4: Missing Collision on Simulated Gravity Body",
        "description": "Create a simulated rigid body with gravity enabled, but with NoCollision enabled on its scene component.",
        "extra_ttl": """
gundam:SimBodyNoColl a ue4:URigidBody ;
    rdfs:label "SimBodyNoColl" ;
    ue4:physicsType ue4:PhysType_Simulated ;
    ue4:bEnableGravity true .

gundam:SimCompNoColl a ue4:USceneComponent ;
    rdfs:label "SimCompNoColl" ;
    ue4:hasRigidBody gundam:SimBodyNoColl ;
    ue4:collisionEnabled ue4:NoCollision .
""",
        "expected_error_sparql": "", # Only validated via SHACL shape
        "expected_error_shacl": "Simulated rigid bodies with gravity enabled must have active collision"
    },
    {
        "name": "Challenge Case 4b: Missing Collision on Simulated Gravity Body (Default NoCollision via lack of properties)",
        "description": "Create a simulated rigid body with gravity enabled, but with no collision properties declared (which defaults to NoCollision).",
        "extra_ttl": """
gundam:SimBodyNoCollDefault a ue4:URigidBody ;
    rdfs:label "SimBodyNoCollDefault" ;
    ue4:physicsType ue4:PhysType_Simulated ;
    ue4:bEnableGravity true .

gundam:SimCompNoCollDefault a ue4:USceneComponent ;
    rdfs:label "SimCompNoCollDefault" ;
    ue4:hasRigidBody gundam:SimBodyNoCollDefault .
""",
        "expected_error_sparql": "", # Only validated via SHACL shape
        "expected_error_shacl": "Simulated rigid bodies with gravity enabled must have active collision"
    },
    {
        "name": "Challenge Case 5: Simulated Body Mass <= 0.0 (Zero Mass)",
        "description": "Create a simulated rigid body with mass Kg set to 0.0.",
        "extra_ttl": """
gundam:SimBodyZeroMass a ue4:URigidBody ;
    rdfs:label "SimBodyZeroMass" ;
    ue4:physicsType ue4:PhysType_Simulated ;
    ue4:massKg 0.0 .
""",
        "expected_error_sparql": "", # Only validated via SHACL shape
        "expected_error_shacl": "Simulated rigid bodies (PhysType_Simulated) must have a declared mass greater than 0.0 kg"
    },
    {
        "name": "Challenge Case 5b: Simulated Body Mass Missing",
        "description": "Create a simulated rigid body with missing mass.",
        "extra_ttl": """
gundam:SimBodyNoMass a ue4:URigidBody ;
    rdfs:label "SimBodyNoMass" ;
    ue4:physicsType ue4:PhysType_Simulated .
""",
        "expected_error_sparql": "", # Only validated via SHACL shape
        "expected_error_shacl": "Simulated rigid bodies (PhysType_Simulated) must have a declared mass greater than 0.0 kg"
    },
    {
        "name": "Challenge Case 6: Primary and Fallback RHI loop",
        "description": "Configure a rendering subsystem with primary and fallback RHI being the same.",
        "extra_ttl": """
gundam:RhiLoopSubsystem a ue4:URenderingSubsystem ;
    rdfs:label "RhiLoopSubsystem" ;
    ue4:primaryRHI ue4:RHI_DirectX11 ;
    ue4:fallbackRHI ue4:RHI_DirectX11 ;
    ue4:supportsRHI ue4:RHI_DirectX11 .
""",
        "expected_error_sparql": "RuleL",
        "expected_error_shacl": "RHI fallback loop"
    },
    {
        "name": "Challenge Case 7: WASM World Rendering Subsystem WebGL Fallback Missing",
        "description": "Create a world with WasmPackagingTypestate, but its rendering subsystem does not support WebGL.",
        "extra_ttl": """
gundam:WasmNoWebGLSubsystem a ue4:URenderingSubsystem ;
    rdfs:label "WasmNoWebGLSubsystem" ;
    ue4:primaryRHI ue4:RHI_DirectX11 ;
    ue4:supportsRHI ue4:RHI_DirectX11 .

gundam:WasmNoWebGLWorld a ue4:UWorld ;
    rdfs:label "WasmNoWebGLWorld" ;
    ue4:hasSubsystem gundam:WasmNoWebGLSubsystem ;
    ue4:hasPackagingState gundam:WasmReady .
""",
        "expected_error_sparql": "RuleM",
        "expected_error_shacl": "WASM WebGL compliance defect"
    }
]

def setup_backup():
    if os.path.exists(BACKUP_PATH):
        os.remove(BACKUP_PATH)
    shutil.copy(CORE_TTL_PATH, BACKUP_PATH)

def restore_backup():
    if os.path.exists(BACKUP_PATH):
        shutil.copy(BACKUP_PATH, CORE_TTL_PATH)

def cleanup():
    restore_backup()
    if os.path.exists(BACKUP_PATH):
        os.remove(BACKUP_PATH)

def main():
    print("Setting up backup of core.ttl...")
    setup_backup()
    
    # First, verify baseline passes
    print("Running baseline check...")
    res = subprocess.run(
        [GGEN_BIN, "sync", "--manifest", MANIFEST_PATH, "--validate-only", "true"],
        capture_output=True,
        text=True
    )
    output = res.stdout + "\n" + res.stderr
    if "FAIL" in output or "Validation failed" in output:
        print("FAIL: Baseline validation is currently failing!")
        print(f"Output was:\n{output}")
        # We don't abort immediately, but we warning
    else:
        print("PASS: Baseline validation clean.")
        
    passed_cases = 0
    total_cases = len(TEST_CASES)
    
    try:
        for case in TEST_CASES:
            print(f"\nRunning {case['name']}...")
            restore_backup()
            
            # Append invalid TTL
            with open(CORE_TTL_PATH, "a") as f:
                f.write(case['extra_ttl'])
                
            # Run ggen sync
            res = subprocess.run(
                [GGEN_BIN, "sync", "--manifest", MANIFEST_PATH, "--validate-only", "true"],
                capture_output=True,
                text=True
            )
            
            output = res.stdout + "\n" + res.stderr
            
            # Verify the expected error messages are present
            sparql_ok = True
            if case['expected_error_sparql']:
                if case['expected_error_sparql'] in output:
                    print(f"  [OK] Found expected SPARQL error: '{case['expected_error_sparql']}'")
                else:
                    print(f"  [FAIL] Missing expected SPARQL error: '{case['expected_error_sparql']}'")
                    sparql_ok = False
                    
            shacl_ok = True
            if case['expected_error_shacl']:
                if case['expected_error_shacl'] in output:
                    print(f"  [OK] Found expected SHACL error: '{case['expected_error_shacl']}'")
                else:
                    print(f"  [FAIL] Missing expected SHACL error: '{case['expected_error_shacl']}'")
                    shacl_ok = False
                    
            if sparql_ok and shacl_ok:
                print(f"PASS: {case['name']} successfully caught invalid topology.")
                passed_cases += 1
            else:
                print(f"FAIL: {case['name']} did not produce the expected validation errors.")
                print(f"Output was:\n{output}")
                
    finally:
        print("\nCleaning up core.ttl...")
        cleanup()
        
    print(f"\nChallenge Results: {passed_cases} / {total_cases} cases passed.")
    if passed_cases == total_cases:
        print("ALL CHALLENGE CASES SUCCESSFULLY VERIFIED!")
        sys.exit(0)
    else:
        print("SOME CHALLENGE CASES FAILED!")
        sys.exit(1)

if __name__ == "__main__":
    main()
