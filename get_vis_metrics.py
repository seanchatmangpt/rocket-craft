with open("scripts/compare_reference_render.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "wing_layer_count_delta =" in line or "feather_panel_curvature_score =" in line or "blade_length_angle_delta =" in line:
        start = max(0, i - 10)
        end = min(len(lines), i + 10)
        print(f"--- MATCH {i} ---")
        for j in range(start, end):
            print(f"{j}: {lines[j].rstrip()}")
