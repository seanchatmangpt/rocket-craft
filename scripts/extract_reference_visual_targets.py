#!/usr/bin/env python3
"""
extract_reference_visual_targets.py

Extracts visual measurements from the reference image for GC-MECH-ASSET-FABRIC-001.
Uses PIL for image processing and output generation.
"""

import os
import json
import colorsys
from PIL import Image, ImageFilter

def main():
    # Setup paths relative to repository root (script directory parent)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.abspath(os.path.join(script_dir, ".."))
    
    input_path = os.path.join(repo_root, "references", "mech", "61gOtV1wnAL._AC_SL1200_.jpg")
    output_dir = os.path.join(repo_root, "generated", "mech_assets", "reference_fabric_001", "reference")
    
    silhouette_path = os.path.join(output_dir, "reference_silhouette.png")
    edges_path = os.path.join(output_dir, "reference_edges.png")
    color_hist_path = os.path.join(output_dir, "reference_color_histogram.json")
    measurements_path = os.path.join(output_dir, "reference_measurements.json")
    
    print(f"Loading reference image from: {input_path}")
    if not os.path.exists(input_path):
        raise FileNotFoundError(f"Input file not found at: {input_path}")
        
    img = Image.open(input_path)
    if img.mode != "RGB":
        img = img.convert("RGB")
        
    w, h = img.size
    print(f"Image dimensions: {w}x{h}")
    
    pixels = img.load()
    
    # 1. Calculate Silhouette Mask
    # Threshold background vs foreground: RGB average > 240 is background, else foreground
    silhouette_img = Image.new("L", (w, h), 0)
    sil_pixels = silhouette_img.load()
    
    fg_pixels_coords = []
    
    for y in range(h):
        for x in range(w):
            r, g, b = pixels[x, y]
            avg = (r + g + b) / 3.0
            if avg > 240.0:
                # Background -> Black
                sil_pixels[x, y] = 0
            else:
                # Foreground -> White
                sil_pixels[x, y] = 255
                fg_pixels_coords.append((x, y))
                
    print(f"Foreground pixels: {len(fg_pixels_coords)} / {w * h}")
    
    # Ensure output directory exists
    os.makedirs(output_dir, exist_ok=True)
    silhouette_img.save(silhouette_path)
    print(f"Saved silhouette mask to: {silhouette_path}")
    
    # 2. Calculate Edge Map
    gray_img = img.convert("L")
    edges_img = gray_img.filter(ImageFilter.FIND_EDGES)
    edges_img.save(edges_path)
    print(f"Saved edge map to: {edges_path}")
    
    # 3. Calculate Dominant Color Palette and Proportions
    # Colors: white, dark/black, cyan, yellow/gold, red, other
    color_counts = {
        "white": 0,
        "dark/black": 0,
        "cyan": 0,
        "yellow/gold": 0,
        "red": 0,
        "other": 0
    }
    
    cyan_coords = []
    red_coords = []
    yellow_coords = []
    
    for x, y in fg_pixels_coords:
        r, g, b = pixels[x, y]
        
        # Convert RGB to HSV
        h_val, s_val, v_val = colorsys.rgb_to_hsv(r/255.0, g/255.0, b/255.0)
        h_deg = h_val * 360.0
        
        # Color classification logic
        if v_val < 0.22:
            color_counts["dark/black"] += 1
        elif s_val < 0.15 and v_val >= 0.70:
            color_counts["white"] += 1
        elif 160.0 <= h_deg <= 210.0 and s_val >= 0.15 and v_val >= 0.2:
            color_counts["cyan"] += 1
            cyan_coords.append((x, y))
        elif 40.0 <= h_deg <= 75.0 and s_val >= 0.15 and v_val >= 0.2:
            color_counts["yellow/gold"] += 1
            yellow_coords.append((x, y))
        elif (h_deg >= 345.0 or h_deg <= 15.0) and s_val >= 0.15 and v_val >= 0.2:
            color_counts["red"] += 1
            red_coords.append((x, y))
        else:
            color_counts["other"] += 1
            
    total_fg = len(fg_pixels_coords)
    color_proportions = {}
    for color, count in color_counts.items():
        color_proportions[color] = {
            "count": count,
            "proportion": count / total_fg if total_fg > 0 else 0.0
        }
        
    with open(color_hist_path, "w") as f:
        json.dump(color_proportions, f, indent=4)
    print(f"Saved color histogram to: {color_hist_path}")
    
    # 4. Calculate Measurements
    # Bounding Box
    if fg_pixels_coords:
        xs = [p[0] for p in fg_pixels_coords]
        ys = [p[1] for p in fg_pixels_coords]
        min_x, max_x = min(xs), max(xs)
        min_y, max_y = min(ys), max(ys)
    else:
        min_x, max_x, min_y, max_y = 0, 0, 0, 0
        
    bbox = [min_x, min_y, max_x, max_y]
    w_box = max_x - min_x + 1
    h_box = max_y - min_y + 1
    aspect_ratio = w_box / h_box if h_box > 0 else 0.0
    wing_span = w_box
    
    # Central torso mass estimate
    # Middle 30% of the bounding box
    torso_min_x = min_x + w_box * 0.35
    torso_max_x = min_x + w_box * 0.65
    torso_w = torso_max_x - torso_min_x + 1
    
    torso_pixel_count = sum(1 for x, y in fg_pixels_coords if torso_min_x <= x <= torso_max_x)
    torso_ratio = torso_pixel_count / total_fg if total_fg > 0 else 0.0
    torso_density = torso_pixel_count / (torso_w * h_box) if (torso_w * h_box) > 0 else 0.0
    
    # Left/right symmetry estimate
    matching_pairs = 0
    total_pairs = 0
    w_half = int(w_box // 2)
    for y in range(min_y, max_y + 1):
        for dx in range(w_half):
            left_val = sil_pixels[min_x + dx, y]
            right_val = sil_pixels[max_x - dx, y]
            if left_val == right_val:
                matching_pairs += 1
            total_pairs += 1
            
    symmetry_score = matching_pairs / total_pairs if total_pairs > 0 else 0.0
    
    # Cyan weapon regions
    mid_x = min_x + w_box / 2.0
    left_cyan = [p for p in cyan_coords if p[0] < mid_x]
    right_cyan = [p for p in cyan_coords if p[0] >= mid_x]
    
    def get_bbox(coords):
        if not coords:
            return None
        xs_c = [p[0] for p in coords]
        ys_c = [p[1] for p in coords]
        return [min(xs_c), min(ys_c), max(xs_c), max(ys_c)]
        
    left_cyan_bbox = get_bbox(left_cyan)
    right_cyan_bbox = get_bbox(right_cyan)
    cyan_bbox = get_bbox(cyan_coords)
    
    cyan_regions = {
        "total_cyan_count": len(cyan_coords),
        "total_cyan_bbox": cyan_bbox,
        "left_cyan_count": len(left_cyan),
        "left_cyan_bbox": left_cyan_bbox,
        "right_cyan_count": len(right_cyan),
        "right_cyan_bbox": right_cyan_bbox
    }
    
    # Head/visor highlight regions
    # Upper 30% of bounding box, middle 20% horizontally
    head_min_y = min_y
    head_max_y = int(min_y + h_box * 0.3)
    head_min_x = int(min_x + w_box * 0.4)
    head_max_x = int(min_x + w_box * 0.6)
    
    head_yellow = [p for p in yellow_coords if head_min_x <= p[0] <= head_max_x and head_min_y <= p[1] <= head_max_y]
    head_red = [p for p in red_coords if head_min_x <= p[0] <= head_max_x and head_min_y <= p[1] <= head_max_y]
    head_cyan = [p for p in cyan_coords if head_min_x <= p[0] <= head_max_x and head_min_y <= p[1] <= head_max_y]
    
    head_highlights = head_yellow + head_red + head_cyan
    head_highlight_bbox = get_bbox(head_highlights)
    
    head_visor_regions = {
        "head_search_area": [head_min_x, head_min_y, head_max_x, head_max_y],
        "head_highlight_count": len(head_highlights),
        "head_highlight_bbox": head_highlight_bbox,
        "head_yellow_count": len(head_yellow),
        "head_yellow_bbox": get_bbox(head_yellow),
        "head_red_count": len(head_red),
        "head_red_bbox": get_bbox(head_red),
        "head_cyan_count": len(head_cyan),
        "head_cyan_bbox": get_bbox(head_cyan)
    }
    
    measurements = {
        "bounding_box": bbox,
        "aspect_ratio": aspect_ratio,
        "wing_span_estimate_px": wing_span,
        "central_torso_mass_estimate": {
            "torso_pixel_count": torso_pixel_count,
            "torso_ratio": torso_ratio,
            "torso_density": torso_density,
            "bounds": [int(torso_min_x), int(torso_max_x)]
        },
        "left_right_symmetry_estimate": symmetry_score,
        "cyan_weapon_regions": cyan_regions,
        "head_visor_highlight_regions": head_visor_regions
    }
    
    with open(measurements_path, "w") as f:
        json.dump(measurements, f, indent=4)
        
    print(f"Saved measurements to: {measurements_path}")
    print("\nVisual Targets Extraction Summary:")
    print(f"  Bounding Box: {bbox}")
    print(f"  Aspect Ratio: {aspect_ratio:.4f}")
    print(f"  Symmetry Score: {symmetry_score:.4f}")
    print(f"  Central Torso Ratio: {torso_ratio:.4f} (Density: {torso_density:.4f})")
    print(f"  Head Highlight Pixels: {len(head_highlights)} (Yellow: {len(head_yellow)}, Red: {len(head_red)})")
    print(f"  Cyan Weapon Pixels: {len(cyan_coords)}")

if __name__ == "__main__":
    main()
