with open("scripts/compare_reference_render.py", "r") as f:
    lines = f.readlines()
for i, line in enumerate(lines):
    if "all_feather_rys =" in line or "all_feather_rys.append" in line:
        print(f"{i}: {line.strip()}")
