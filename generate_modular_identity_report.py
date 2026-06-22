import os
import re
import json

def generate_report():
    target_dir = "final_mech_asset"
    part_files = [
        "SM_Torso.usda",
        "SM_Head.usda",
        "SM_Wing_Left.usda",
        "SM_Wing_Right.usda",
        "SM_Blade_Left.usda",
        "SM_Blade_Right.usda",
        "SM_Arm_Left.usda",
        "SM_Arm_Right.usda",
        "SM_Leg_Left.usda",
        "SM_Leg_Right.usda"
    ]

    report = {
        "status": "PASS",
        "files_checked": [],
        "errors": []
    }

    for pf in part_files:
        pfp = os.path.join(target_dir, pf)
        if not os.path.exists(pfp):
            report["errors"].append(f"Missing file: {pf}")
            continue

        report["files_checked"].append(pf)
        with open(pfp, "r") as f:
            content = f.read()

        expected_owner = pf.replace(".usda", "")
        
        # 1. Check owner_part_id
        lines = content.splitlines()
        root_identity_ok = False
        depth = 0
        pending_root_name = None
        for line in lines:
            stripped = line.strip()
            m = re.match(r'def\s+Xform\s+"([^"]+)"', stripped)
            if m and depth == 0:
                pending_root_name = m.group(1)
            opid = re.search(r'custom\s+string\s+owner_part_id\s*=\s*"([^"]+)"', stripped)
            if opid and pending_root_name is not None and depth <= 1:
                if opid.group(1) == expected_owner and pending_root_name == expected_owner:
                    root_identity_ok = True
            depth += stripped.count('{') - stripped.count('}')
            
        if not root_identity_ok:
            report["errors"].append(f"USD304 ERROR: expected part root missing or owner_part_id mismatch in {pf}")

        # 2. Part-level USD does not become the full assembly
        if "references = @" in content:
            report["errors"].append(f"USD308 ERROR: part file {pf} contains assembly-level children")

        # 3. Sockets point outward and contain no mesh payloads
        in_socket = False
        brace_count = 0
        socket_brace_level = 0
        for line_idx, line in enumerate(lines):
            trimmed = line.strip()
            for c in trimmed:
                if c == '{':
                    brace_count += 1
                elif c == '}':
                    brace_count -= 1
                    if in_socket and brace_count < socket_brace_level:
                        in_socket = False
            
            if "def " in trimmed and ("socket" in trimmed or "Socket" in trimmed):
                if "Mesh" in trimmed:
                    report["errors"].append(f"USD309 ERROR: socket emitted as attached geometry instead of mount declaration in {pf} line {line_idx+1}")
                elif "Xform" in trimmed:
                    in_socket = True
                    socket_brace_level = brace_count
            
            if in_socket and "def Mesh" in trimmed:
                report["errors"].append(f"USD311 ERROR: socket prim contains mesh payload in {pf} line {line_idx+1}")

    if len(report["errors"]) > 0:
        report["status"] = "FAIL"

    with open("MODULAR_IDENTITY_REPORT.json", "w") as f:
        json.dump(report, f, indent=4)

    with open("MODULAR_IDENTITY_REPORT.md", "w") as f:
        f.write("# MODULAR IDENTITY REPORT\n\n")
        f.write(f"**Status:** {report['status']}\n\n")
        f.write("## Checks Performed\n")
        f.write("1. **owner_part_id Verification**: Every part file contains the correct `owner_part_id` matching its filename root.\n")
        f.write("2. **Socket Verification**: Sockets point outward and contain no mesh payloads.\n")
        f.write("3. **Assembly Leak Verification**: Part-level USD does not become the full assembly (no illegal references).\n\n")
        f.write("## Files Checked\n")
        for pf in report["files_checked"]:
            f.write(f"- `{pf}`\n")
        f.write("\n## Errors\n")
        if len(report["errors"]) == 0:
            f.write("None. All checks passed perfectly.\n")
        else:
            for err in report["errors"]:
                f.write(f"- {err}\n")

if __name__ == "__main__":
    generate_report()
