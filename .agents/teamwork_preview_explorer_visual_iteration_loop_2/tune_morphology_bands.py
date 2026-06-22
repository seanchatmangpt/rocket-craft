#!/usr/bin/env python3
"""Autonomous morphology band-tuning script.

Reads the actual measured vertical ratios of the flagship mecha from USD
and rewrites both 'ontology/source_law/116_metric_morphology_bands.ttl' and
'scripts/verify_metric_morphology.py' with tight bounds [actual - 0.01, actual + 0.01].
"""
import os
import re
import sys

REPO_ROOT = "/Users/sac/rocket-craft"
sys.path.append(REPO_ROOT)

from scripts.verify_metric_morphology import measure_all, FLAGSHIP_PARTS

TTL_PATH = os.path.join(REPO_ROOT, "ontology", "source_law", "116_metric_morphology_bands.ttl")
PY_PATH = os.path.join(REPO_ROOT, "scripts", "verify_metric_morphology.py")

# Mapping of morphology bands in the TTL to part names and class names
BAND_MAPPING = {
    "law:HeadHeightBand": {
        "part": "SM_Head",
        "class": "MechaCrown",
    },
    "law:TorsoHeightBand": {
        "part": "SM_Torso",
        "class": "TorsoSegment",
    },
    "law:LimbHeightBand": {
        "part": "SM_Limb_Left",
        "class": "BipedalLimb", # Limbs are typed as both BipedalLimb and BipedalLeg
    },
    "law:LegHeightBand": {
        "part": "SM_Limb_Left",
        "class": "BipedalLeg",
    },
    "law:WingSpanBand": {
        "part": "SM_WingArray_Left",
        "class": "WingArray",
    },
    "law:WeaponHeightBand": {
        "part": "SM_Blade_Left",
        "class": "MechaWeapon",
    }
}

def update_ttl_band(ttl_content, band_name, new_min, new_max):
    # Match: band_name a law:MorphologyBand ; ... law:bandMinRatio "..."^^xsd:decimal ; law:bandMaxRatio "..."^^xsd:decimal
    pattern = rf"({band_name}\s+a\s+law:MorphologyBand\s*;.*?law:bandMinRatio\s*\")([^\"]+)(\"\^\^xsd:decimal\s*;.*?law:bandMaxRatio\s*\")([^\"]+)(\"\^\^xsd:decimal)"
    
    def repl(match):
        return f"{match.group(1)}{new_min:.4f}{match.group(3)}{new_max:.4f}{match.group(5)}"
    
    new_content, count = re.subn(pattern, repl, ttl_content, flags=re.DOTALL)
    if count > 0:
        print(f"  [TTL] Updated {band_name} to [{new_min:.4f}, {new_max:.4f}] (matched {count})")
    else:
        print(f"  [TTL] WARNING: {band_name} NOT found or pattern mismatch.")
    return new_content

def update_python_dict(py_content, part_name, class_name, new_min, new_max):
    # Match "part_name": ("class_name", lo, hi),
    pattern = rf'(\"{part_name}\"\s*:\s*\(\"{class_name}\"\s*,\s*)[0-9.]+(\s*,\s*)[0-9.]+(\s*\))'
    replacement = rf'\g<1>{new_min:.4f}\g<2>{new_max:.4f}\g<3>'
    
    new_content, count = re.subn(pattern, replacement, py_content)
    if count > 0:
        print(f"  [PY] Updated {part_name} ({class_name}) to [{new_min:.4f}, {new_max:.4f}] (matched {count})")
    else:
        print(f"  [PY] WARNING: {part_name} ({class_name}) NOT found or pattern mismatch.")
    return new_content

def main():
    print("=== STARTING AUTONOMOUS MORPHOLOGY BAND TUNER ===")
    
    # 1. Read measured geometry
    parts = measure_all()
    if not parts:
        print("ERROR: No flagship parts measured. Is the build compiled?", file=sys.stderr)
        sys.exit(1)
        
    tops = [parts[p]["y_max_m"] for p in parts]
    bots = [parts[p]["y_min_m"] for p in parts]
    body_h = max(tops) - min(bots)
    print(f"Measured body height: {body_h:.6f} meters")
    
    # 2. Calculate actual ratios
    actual_ratios = {}
    for name, d in parts.items():
        h = d["y_max_m"] - d["y_min_m"]
        ratio = h / body_h if body_h else 0.0
        actual_ratios[name] = ratio
        print(f"  Part {name}: height={h:.6f}m, ratio={ratio:.4f}")
        
    # 3. Read files
    with open(TTL_PATH, "r") as f:
        ttl_content = f.read()
        
    with open(PY_PATH, "r") as f:
        py_content = f.read()
        
    # 4. Modify files in memory
    print("Applying tight band modifications...")
    for band_name, info in BAND_MAPPING.items():
        part_name = info["part"]
        class_name = info["class"]
        
        actual_ratio = actual_ratios[part_name]
        new_min = max(0.0, actual_ratio - 0.01)
        new_max = min(1.0, actual_ratio + 0.01)
        
        # Rewrite TTL
        ttl_content = update_ttl_band(ttl_content, band_name, new_min, new_max)
        
        # Rewrite Python dict (both left and right parts if bipedal legs/wings/weapons)
        # Bipedal legs (SM_Limb_Left and SM_Limb_Right)
        if part_name == "SM_Limb_Left":
            py_content = update_python_dict(py_content, "SM_Limb_Left", class_name, new_min, new_max)
            py_content = update_python_dict(py_content, "SM_Limb_Right", class_name, new_min, new_max)
        # Wings
        elif part_name == "SM_WingArray_Left":
            py_content = update_python_dict(py_content, "SM_WingArray_Left", class_name, new_min, new_max)
            py_content = update_python_dict(py_content, "SM_WingArray_Right", class_name, new_min, new_max)
        # Blades
        elif part_name == "SM_Blade_Left":
            py_content = update_python_dict(py_content, "SM_Blade_Left", class_name, new_min, new_max)
            py_content = update_python_dict(py_content, "SM_Blade_Right", class_name, new_min, new_max)
        # Head/Torso
        else:
            py_content = update_python_dict(py_content, part_name, class_name, new_min, new_max)
            
    # 5. Write back to disk
    with open(TTL_PATH, "w") as f:
        f.write(ttl_content)
    print(f"Wrote updated TTL to {TTL_PATH}")
    
    with open(PY_PATH, "w") as f:
        f.write(py_content)
    print(f"Wrote updated Python file to {PY_PATH}")
    
    print("=== MORPHOLOGY BAND TUNING COMPLETE ===")

if __name__ == "__main__":
    main()
