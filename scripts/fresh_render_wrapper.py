#!/usr/bin/env python3
import os
import sys
import subprocess
import time
import json
import glob
from datetime import datetime

def run_cmd(cmd):
    print(f">> Running: {' '.join(cmd)}")
    subprocess.run(cmd, check=True)

def b3sum_file(filepath):
    result = subprocess.run(["b3sum", filepath], capture_output=True, text=True, check=True)
    return result.stdout.split()[0]

def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(repo_root)
    
    asset_dir = os.path.join(repo_root, "generated", "mech_assets", "reference_fabric_001")
    renders_dir = os.path.join(asset_dir, "renders")
    reports_dir = os.path.join(asset_dir, "reports")
    
    # 1. Reject stale render reuse by deleting existing renders and reports
    print(">> Deleting stale renders and reports...")
    stale_files = glob.glob(os.path.join(renders_dir, "*.png"))
    stale_files.append(os.path.join(reports_dir, "visual_gap_report.json"))
    stale_files.append(os.path.join(reports_dir, "visual_gap_report.md"))
    
    for f in stale_files:
        if os.path.exists(f):
            os.remove(f)
            
    # Stale-render refusal fixture: assert they are gone
    for f in stale_files:
        assert not os.path.exists(f), f"Failed to delete {f}"
        
    start_time = datetime.utcnow().isoformat() + "Z"
    
    # 2. Run source merge
    run_cmd(["python3", "scripts/merge_ontology.py"])
    
    # 3. Run ggen sync
    ggen_bin = os.environ.get("GGEN_BIN", "/Users/sac/.local/bin/ggen")
    run_cmd([ggen_bin, "sync"])
    
    # 4. Run headless render
    run_cmd(["python3", "scripts/render_reference_fabric.py"])
    
    # 5. Run visual comparison
    run_cmd(["python3", "scripts/compare_reference_render.py"])
    
    # 6. Read gap report
    report_path = os.path.join(reports_dir, "visual_gap_report.json")
    with open(report_path, "r") as f:
        gap_report = json.load(f)
        
    # 7. Collect Hashes
    render_front = os.path.join(renders_dir, "render_front.png")
    render_angled = os.path.join(renders_dir, "render_angled.png")
    
    hashes = {
        "render_front.png": b3sum_file(render_front),
        "render_angled.png": b3sum_file(render_angled),
        "visual_gap_report.json": b3sum_file(report_path)
    }
    
    # 8. Emit FRESH_RENDER_VERIFICATION_REPORT.json
    fresh_report = {
        "verification_name": "FRESH_RENDER_VERIFICATION",
        "timestamp": start_time,
        "wrapper_script": "scripts/fresh_render_wrapper.py",
        "stale_render_refusal": "passed",
        "render_hashes_blake3": hashes,
        "visual_gap_report_summary": {
            "silhouette_iou": gap_report.get("silhouette_iou"),
            "color_palette_similarity": gap_report.get("color_palette_similarity"),
            "morphology_errors": len(gap_report.get("vis_errors", [])),
            "usd_errors": len(gap_report.get("usd_errors", [])),
            "thresholds_met": gap_report.get("thresholds_met")
        },
        "full_comparison_report_hash": hashes["visual_gap_report.json"]
    }
    
    out_json = "FRESH_RENDER_VERIFICATION_REPORT.json"
    with open(out_json, "w") as f:
        json.dump(fresh_report, f, indent=4)
        
    out_md = "FRESH_RENDER_VERIFICATION_REPORT.md"
    with open(out_md, "w") as f:
        f.write("# Fresh Render Verification Report\n\n")
        f.write(f"- **Timestamp**: {start_time}\n")
        f.write(f"- **Wrapper Script**: `scripts/fresh_render_wrapper.py`\n")
        f.write(f"- **Stale Render Refusal**: PASSED (renders deleted before sync)\n")
        f.write("## BLAKE3 Hashes\n")
        for k, v in hashes.items():
            f.write(f"- `{k}`: {v}\n")
        f.write("\n## Comparison Summary\n")
        f.write(f"- Silhouette IoU: {gap_report.get('silhouette_iou')}\n")
        f.write(f"- Color Palette Similarity: {gap_report.get('color_palette_similarity')}\n")
        f.write(f"- Morphology Errors: {len(gap_report.get('vis_errors', []))}\n")
        f.write(f"- USD Errors: {len(gap_report.get('usd_errors', []))}\n")
        f.write(f"- Thresholds Met: {gap_report.get('thresholds_met')}\n")
        
    print(f">> Wrote {out_json} and {out_md}")

if __name__ == "__main__":
    main()
