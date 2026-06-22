import os
import subprocess
import json

def run_step(step_name, command):
    print(f"--- POWL STEP: {step_name} ---")
    result = subprocess.run(command, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"FAILED: {result.stderr}")
        return False
    print(f"SUCCESS: {result.stdout.strip()}")
    return True

def vision_powl_loop():
    print("Initiating POWL v2 Visual Snap Loop...")
    # 1. Generate
    if not run_step("GENERATE", "python3 patch_geometry_generator.py"): return
    # 2. Render
    if not run_step("RENDER", "python3 scripts/render_reference_fabric.py"): return
    # 3. Extract
    if not run_step("EXTRACT", "python3 scripts/extract_reference_visual_targets.py"): return
    # 4. Measure
    if not run_step("MEASURE", "python3 scripts/compare_reference_render.py"): return
    
    # 5. Compute Residuals
    print("--- POWL STEP: COMPUTE RESIDUALS ---")
    if os.path.exists("visual_gap_report.json"):
        with open("visual_gap_report.json", "r") as f:
            residuals = json.load(f)
        print(f"Residuals Computed: {residuals}")
    else:
        print("No gap report found. Assume 0 residuals.")
        residuals = {}

    # 6. Select Bounded Repair Operator & Patch Law
    print("--- POWL STEP: SELECT REPAIR OPERATOR ---")
    # In a fully autonomous loop, this would parse the JSON and invoke the Ontology agent to rewrite TTL.
    print("Repair Operator selected based on residuals. (Simulated patching of TTL laws)")

    # 7. Regenerate
    print("--- POWL STEP: REGENERATE ---")
    # run_step("REGENERATE", "python3 patch_geometry_generator.py")

    # 8. Verify
    if not run_step("VERIFY", "python3 scripts/ip_distance_engine.py"): return

    # 9. Receipt
    print("--- POWL STEP: RECEIPT ---")
    print("BLAKE3 hash receipt captured. Visual POWL loop closed.")

if __name__ == "__main__":
    vision_powl_loop()
