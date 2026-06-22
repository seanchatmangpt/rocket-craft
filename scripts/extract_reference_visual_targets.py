#!/usr/bin/env python3
"""
extract_reference_visual_targets.py

Extracts visual measurements from the reference image for GC-MECH-ASSET-FABRIC-001.

The reference (references/mech/61gOtV1wnAL._AC_SL1200_.jpg) is a WHITE winged Mecha
photographed on a SMOOTH LIGHT-GRAY GRADIENT background, with Japanese caption bars at
the bottom of the frame.

A naive global threshold FAILS here for two reasons:
  1. The caption bars (white background + black text) at the bottom get grabbed as foreground.
  2. White-armor-on-gray-gradient is low contrast; a single brightness threshold either grabs
     the bright background or drops the shadowed/gray parts of the mech.

This extractor instead:
  1. CROPS the bottom caption region out before segmentation.
  2. SEGMENTS the mech robustly by combining three foreground cues that the smooth gray
     background does NOT exhibit:
        - local texture / high local variance (mech panel lines, edges, greebles)
        - color saturation (cyan sabers, yellow mecha crown, red decals are strongly colored)
        - high brightness (white armor brighter than the gray gradient)
     followed by morphological closing, largest-connected-component selection, hole filling,
     and a light opening to drop speckle.
  3. Emits reference_silhouette.png as WHITE MECH ON BLACK (same polarity as
     renders/render_silhouette.png).
  4. Recomputes reference_edges.png, reference_measurements.json, and
     reference_color_histogram.json from the FOREGROUND MASK ONLY (background excluded).
"""

import os
import json
import colorsys

import numpy as np
from PIL import Image, ImageFilter
from scipy import ndimage

# --- Segmentation tuning constants ---
CAPTION_CROP_FRAC = 0.80   # keep top 80% of the frame; bottom is caption bars
TEXTURE_WINDOW = 11        # local-variance window (px)
TEXTURE_THRESH = 15.0      # local std above this => textured (mech detail)
SAT_THRESH = 22.0          # max-min channel spread above this => colored (saber/mecha crown/decal)
BRIGHT_THRESH = 192.0      # brightness above this => white armor (gray bg maxes ~175)


def segment_mech(rgb):
    """Return (mask, crop_row) where mask is a full-frame bool array (mech=True)."""
    H, W, _ = rgb.shape
    crop = int(H * CAPTION_CROP_FRAC)
    sub = rgb[:crop].astype(np.float32)

    gray = sub.mean(axis=2)
    sat = sub.max(axis=2) - sub.min(axis=2)

    # local standard deviation (texture)
    mean = ndimage.uniform_filter(gray, TEXTURE_WINDOW)
    mean_sq = ndimage.uniform_filter(gray * gray, TEXTURE_WINDOW)
    std = np.sqrt(np.clip(mean_sq - mean * mean, 0.0, None))

    colored = sat > SAT_THRESH
    bright = gray > BRIGHT_THRESH
    textured = std > TEXTURE_THRESH

    fg = colored | bright | textured

    # kill the crop-edge gradient artifact before morphology
    fg[-4:] = False

    fg = ndimage.binary_closing(fg, iterations=4)
    fg = ndimage.binary_fill_holes(fg)

    # keep only the largest connected component (the mech)
    lbl, n = ndimage.label(fg)
    if n > 0:
        sizes = ndimage.sum(np.ones_like(lbl), lbl, range(1, n + 1))
        biggest = int(np.argmax(sizes)) + 1
        fg = lbl == biggest

    fg = ndimage.binary_fill_holes(fg)
    fg = ndimage.binary_opening(fg, iterations=1)
    fg = ndimage.binary_fill_holes(fg)

    mask = np.zeros((H, W), dtype=bool)
    mask[:crop] = fg
    return mask, crop


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.abspath(os.path.join(script_dir, ".."))

    input_path = os.path.join(repo_root, "references", "mech", "61gOtV1wnAL._AC_SL1200_.jpg")
    output_dir = os.path.join(
        repo_root, "generated", "mech_assets", "reference_fabric_001", "reference"
    )

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

    rgb = np.asarray(img)  # (H, W, 3) uint8

    # 1. Robust silhouette mask (white mech on black) -- background excluded.
    mask, crop_row = segment_mech(rgb)
    os.makedirs(output_dir, exist_ok=True)

    silhouette_arr = (mask.astype(np.uint8)) * 255
    Image.fromarray(silhouette_arr, mode="L").save(silhouette_path)
    fg_count = int(mask.sum())
    print(f"Caption crop row: {crop_row} (kept top {CAPTION_CROP_FRAC:.0%})")
    print(f"Foreground pixels: {fg_count} / {w * h}  (fill={fg_count/(w*h):.4f})")
    print(f"Saved silhouette mask to: {silhouette_path}")

    # 2. Edge map -- computed from FOREGROUND ONLY (background zeroed so edges trace the mech).
    gray_full = np.asarray(img.convert("L"))
    masked_gray = np.where(mask, gray_full, 0).astype(np.uint8)
    edges_img = Image.fromarray(masked_gray, mode="L").filter(ImageFilter.FIND_EDGES)
    # suppress the mask-boundary edge so only internal mech detail remains
    eroded = ndimage.binary_erosion(mask, iterations=2)
    edges_arr = np.asarray(edges_img)
    edges_arr = np.where(eroded, edges_arr, 0).astype(np.uint8)
    Image.fromarray(edges_arr, mode="L").save(edges_path)
    print(f"Saved edge map to: {edges_path}")

    # 3. Color histogram -- FOREGROUND PIXELS ONLY.
    fg_ys, fg_xs = np.nonzero(mask)
    fg_coords = list(zip(fg_xs.tolist(), fg_ys.tolist()))

    color_counts = {
        "white": 0,
        "dark/black": 0,
        "cyan": 0,
        "yellow/gold": 0,
        "red": 0,
        "other": 0,
    }
    cyan_coords = []
    red_coords = []
    yellow_coords = []

    for x, y in fg_coords:
        r, g, b = (int(c) for c in rgb[y, x])
        h_val, s_val, v_val = colorsys.rgb_to_hsv(r / 255.0, g / 255.0, b / 255.0)
        h_deg = h_val * 360.0

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

    total_fg = len(fg_coords)
    color_proportions = {}
    for color, count in color_counts.items():
        color_proportions[color] = {
            "count": count,
            "proportion": count / total_fg if total_fg > 0 else 0.0,
        }

    with open(color_hist_path, "w") as f:
        json.dump(color_proportions, f, indent=4)
    print(f"Saved color histogram to: {color_hist_path}")

    # 4. Measurements (all derived from the foreground mask).
    if fg_coords:
        min_x, max_x = int(fg_xs.min()), int(fg_xs.max())
        min_y, max_y = int(fg_ys.min()), int(fg_ys.max())
    else:
        min_x = max_x = min_y = max_y = 0

    bbox = [min_x, min_y, max_x, max_y]
    w_box = max_x - min_x + 1
    h_box = max_y - min_y + 1
    aspect_ratio = w_box / h_box if h_box > 0 else 0.0
    wing_span = w_box

    torso_min_x = min_x + w_box * 0.35
    torso_max_x = min_x + w_box * 0.65
    torso_w = torso_max_x - torso_min_x + 1
    torso_pixel_count = int(((fg_xs >= torso_min_x) & (fg_xs <= torso_max_x)).sum())
    torso_ratio = torso_pixel_count / total_fg if total_fg > 0 else 0.0
    torso_density = (
        torso_pixel_count / (torso_w * h_box) if (torso_w * h_box) > 0 else 0.0
    )

    # left/right symmetry from the mask
    w_half = int(w_box // 2)
    if w_half > 0:
        left = mask[min_y : max_y + 1, min_x : min_x + w_half]
        right = mask[min_y : max_y + 1, max_x - w_half + 1 : max_x + 1][:, ::-1]
        n = min(left.shape[1], right.shape[1])
        symmetry_score = float((left[:, :n] == right[:, :n]).mean())
    else:
        symmetry_score = 0.0

    mid_x = min_x + w_box / 2.0
    left_cyan = [p for p in cyan_coords if p[0] < mid_x]
    right_cyan = [p for p in cyan_coords if p[0] >= mid_x]

    def get_bbox(coords):
        if not coords:
            return None
        xs_c = [p[0] for p in coords]
        ys_c = [p[1] for p in coords]
        return [min(xs_c), min(ys_c), max(xs_c), max(ys_c)]

    cyan_regions = {
        "total_cyan_count": len(cyan_coords),
        "total_cyan_bbox": get_bbox(cyan_coords),
        "left_cyan_count": len(left_cyan),
        "left_cyan_bbox": get_bbox(left_cyan),
        "right_cyan_count": len(right_cyan),
        "right_cyan_bbox": get_bbox(right_cyan),
    }

    head_min_y = min_y
    head_max_y = int(min_y + h_box * 0.3)
    head_min_x = int(min_x + w_box * 0.4)
    head_max_x = int(min_x + w_box * 0.6)

    def in_head(p):
        return head_min_x <= p[0] <= head_max_x and head_min_y <= p[1] <= head_max_y

    head_yellow = [p for p in yellow_coords if in_head(p)]
    head_red = [p for p in red_coords if in_head(p)]
    head_cyan = [p for p in cyan_coords if in_head(p)]
    head_highlights = head_yellow + head_red + head_cyan

    head_visor_regions = {
        "head_search_area": [head_min_x, head_min_y, head_max_x, head_max_y],
        "head_highlight_count": len(head_highlights),
        "head_highlight_bbox": get_bbox(head_highlights),
        "head_yellow_count": len(head_yellow),
        "head_yellow_bbox": get_bbox(head_yellow),
        "head_red_count": len(head_red),
        "head_red_bbox": get_bbox(head_red),
        "head_cyan_count": len(head_cyan),
        "head_cyan_bbox": get_bbox(head_cyan),
    }

    measurements = {
        "bounding_box": bbox,
        "aspect_ratio": aspect_ratio,
        "wing_span_estimate_px": wing_span,
        "central_torso_mass_estimate": {
            "torso_pixel_count": torso_pixel_count,
            "torso_ratio": torso_ratio,
            "torso_density": torso_density,
            "bounds": [int(torso_min_x), int(torso_max_x)],
        },
        "left_right_symmetry_estimate": symmetry_score,
        "cyan_weapon_regions": cyan_regions,
        "head_visor_highlight_regions": head_visor_regions,
    }

    with open(measurements_path, "w") as f:
        json.dump(measurements, f, indent=4)
    print(f"Saved measurements to: {measurements_path}")

    print("\nVisual Targets Extraction Summary:")
    print(f"  Bounding Box: {bbox}")
    print(f"  Aspect Ratio: {aspect_ratio:.4f}")
    print(f"  Symmetry Score: {symmetry_score:.4f}")
    print(f"  Central Torso Ratio: {torso_ratio:.4f} (Density: {torso_density:.4f})")
    print(
        f"  Head Highlight Pixels: {len(head_highlights)} "
        f"(Yellow: {len(head_yellow)}, Red: {len(head_red)})"
    )
    print(f"  Cyan Weapon Pixels: {len(cyan_coords)}")
    print(f"  Color proportions: " + ", ".join(
        f"{k}={v['proportion']:.3f}" for k, v in color_proportions.items()))


if __name__ == "__main__":
    main()
