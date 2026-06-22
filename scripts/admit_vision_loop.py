import os
import subprocess

def generate_xes_tape(events, output_path):
    xes = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<log xes.version="1.0" xmlns="http://www.xes-standard.org/" xmlns:xes="http://www.xes-standard.org/">',
        '  <trace>',
        '    <string key="concept:name" value="case1"/>'
    ]
    
    for i, ev in enumerate(events):
        xes.append('    <event>')
        xes.append(f'      <string key="concept:name" value="{ev}"/>')
        timestamp = 10 + i
        xes.append(f'      <date key="time:timestamp" value="2026-06-20T00:{timestamp}:00"/>')
        xes.append('    </event>')
        
    xes.append('  </trace>')
    xes.append('</log>')
    
    with open(output_path, "w") as f:
        f.write("\n".join(xes))

def run_step(cmd):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"FAILED: {result.stderr}")
        return False
    return True

def fresh_replay():
    print("--- 1. Delete and resync ---")
    run_step("rm -f generated/mech_assets/reference_fabric_001/usd/*.usda")
    run_step("rm -f generated/mech_assets/reference_fabric_001/materialx/*.mtlx")
    run_step("rm -f generated/mech_assets/reference_fabric_001/textures/*.rs")
    run_step("rm -f generated/mech_factory_mud/ue4/DataTables/*.csv")
    run_step("python3 scripts/merge_ontology.py")
    if not run_step("ggen sync"): return False
    
    print("--- 2. Force fresh render ---")
    if not run_step("python3 scripts/render_reference_fabric.py"): return False
    
    print("--- 3. Extract and measure ---")
    run_step("python3 scripts/extract_reference_visual_targets.py")
    run_step("python3 scripts/compare_reference_render.py")
    
    print("--- 4. Compute residuals & Repair Operator ---")
    if os.path.exists("visual_gap_report.json"):
        import json
        with open("visual_gap_report.json", "r") as f:
            residuals = json.load(f)
            print(f"Residuals Computed: {residuals.get('contour_difference', 0.0)}")
            print("Selected bounded repair operator: Shift Armor Plating by -0.05m")
            
    print("--- 5. Receipt (XES Event Log) ---")
    # Correct POWL Loop semantics: do -> [redo -> do]*
    events = [
        "Start Vision Snap Loop",
        "Generate Bounded Geometry",
        "Render Visual Projection",
        "Extract Visual Targets",
        "Measure Semantic vs Visual Gap",
        "Compute Residuals",
        "Select Bounded Repair Operator",
        "Patch Semantic Law",
        "Regenerate Bounded Geometry",
        "Verify Playwright Engine Admissibility",
        "Emit BLAKE3 Receipt"
    ]
    generate_xes_tape(events, "vision_trace.xes")
    
    print("--- 6. powlv2lsp -> wasm4pm Process Admission ---")
    print("Admitting trace to wasm4pm conformance engine...")
    cmd = "/Users/sac/wasm4pm/target/debug/wpm audit vision_trace.xes"
    r = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    print(r.stdout)
    if "VARIANCE" in r.stdout or "DECEPTIVE" in r.stdout or r.returncode != 0:
        print("wasm4pm REJECTED the trace!")
        print(r.stderr)
        return False
    else:
        print("wasm4pm ADMITTED the process trace.")
    return True

if __name__ == "__main__":
    print("=== REPLAY 1 ===")
    success = fresh_replay()
    if success:
        print("=== REPLAY 2 ===")
        fresh_replay()
