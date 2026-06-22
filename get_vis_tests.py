with open("scripts/compare_reference_render.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "VIS203" in line or "VIS202" in line or "VIS205" in line:
        start = max(0, i - 15)
        end = min(len(lines), i + 15)
        print(f"--- MATCH {i} ---")
        for j in range(start, end):
            print(f"{j}: {lines[j].rstrip()}")
