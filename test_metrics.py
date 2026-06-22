import subprocess
import json
import os

# Render and test metrics
subprocess.run(["python3", "scripts/render_reference_fabric.py"])
subprocess.run(["python3", "scripts/compare_reference_render.py"])

try:
    with open("generated/mech_assets/reference_fabric_001/reports/visual_gap_report.json") as f:
        print(f.read())
except Exception as e:
    print("Error reading report:", e)
