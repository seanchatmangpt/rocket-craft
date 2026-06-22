import subprocess
import os

base_events = [
    "Start Vision Snap Loop",
    "Generate Bounded Geometry",
    "Render Visual Projection",
    "Extract Visual Targets",
    "Measure Semantic vs Visual Gap",
    "Compute Residuals",
    "Select Bounded Repair Operator",
    "Patch Semantic Law",
    "Regenerate Bounded Geometry",
    "Compute Residuals",
    "Verify Playwright Engine Admissibility",
    "Emit BLAKE3 Receipt"
]

def generate_xes(events):
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
    with open("temp_trace.xes", "w") as f:
        f.write("\n".join(xes))

def get_fitness(events):
    generate_xes(events)
    cmd = "/Users/sac/wasm4pm/target/debug/wpm audit temp_trace.xes"
    r = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    for line in r.stdout.split("\n"):
        if "Fitness Score:" in line:
            try:
                return float(line.split(":")[1].strip())
            except:
                pass
    return 0.0

print(f"Base fitness: {get_fitness(base_events)}")

# Try removing each event
for i in range(len(base_events)):
    test_events = base_events[:i] + base_events[i+1:]
    score = get_fitness(test_events)
    print(f"Drop index {i} ({base_events[i]}): {score}")

# Wait, if M: 1, R: 1, maybe an event is WRONG? 
# If it's missing 1 and remaining 1, then the length is correct but one event name is wrong, OR we have one extra and one missing elsewhere.
# Try checking the exact names in the built-in model if possible.
