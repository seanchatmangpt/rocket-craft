#!/usr/bin/env python3
"""
render_reference_fabric.py

Procedurally generates mech textures, runs headless usdrecord to render front and angled views,
and post-processes the renders to generate binary silhouettes and edge maps.
"""

import os
import sys
import subprocess
import json
from PIL import Image, ImageDraw, ImageFilter

def generate_textures(texture_dir):
    os.makedirs(texture_dir, exist_ok=True)
    
    base_color_path = os.path.join(texture_dir, "T_WhiteArmor_BaseColor.png")
    roughness_path = os.path.join(texture_dir, "T_WhiteArmor_Roughness.png")
    normal_path = os.path.join(texture_dir, "T_WhiteArmor_Normal.png")
    emissive_path = os.path.join(texture_dir, "T_CyanBlade_Emissive.png")
    manifest_path = os.path.join(texture_dir, "texture_manifest.json")
    
    print("Generating procedural textures...")
    
    # 1. BaseColor: 512x512 white image with subtle grid/panel details
    base_img = Image.new("RGB", (512, 512), (245, 245, 245))
    draw = ImageDraw.Draw(base_img)
    grid_color = (220, 220, 220)
    # Draw horizontal and vertical grid lines
    for i in range(0, 512, 64):
        draw.line([(i, 0), (i, 512)], fill=grid_color, width=1)
        draw.line([(0, i), (512, i)], fill=grid_color, width=1)
    # Draw diagonal panel lines
    draw.line([(0, 0), (512, 512)], fill=grid_color, width=1)
    draw.line([(0, 512), (512, 0)], fill=grid_color, width=1)
    base_img.save(base_color_path)
    
    # 2. Roughness: 512x512 light gray image
    rough_img = Image.new("RGB", (512, 512), (200, 200, 200))
    rough_img.save(roughness_path)
    
    # 3. Normal: 512x512 flat normal map [128, 128, 255]
    normal_img = Image.new("RGB", (512, 512), (128, 128, 255))
    normal_img.save(normal_path)
    
    # 4. Emissive: 512x512 cyan image
    emissive_img = Image.new("RGB", (512, 512), (0, 255, 255))
    emissive_img.save(emissive_path)
    
    # 5. Manifest
    manifest = {
        "textures": [
            {
                "file": "T_WhiteArmor_BaseColor.png",
                "dimensions": [512, 512],
                "purpose": "BaseColor"
            },
            {
                "file": "T_WhiteArmor_Roughness.png",
                "dimensions": [512, 512],
                "purpose": "Roughness"
            },
            {
                "file": "T_WhiteArmor_Normal.png",
                "dimensions": [512, 512],
                "purpose": "Normal"
            },
            {
                "file": "T_CyanBlade_Emissive.png",
                "dimensions": [512, 512],
                "purpose": "Emissive"
            }
        ]
    }
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=4)
        
    print("Texture generation complete!")

def run_renderer(repo_root, renders_dir):
    os.makedirs(renders_dir, exist_ok=True)
    
    master_usd = os.path.join(repo_root, "generated", "mech_assets", "reference_fabric_001", "usd", "ASSET_ReferenceFabric_001.usda")
    render_front = os.path.join(renders_dir, "render_front.png")
    render_angled = os.path.join(renders_dir, "render_angled.png")
    
    print(f"Master USD file: {master_usd}")
    if not os.path.exists(master_usd):
        print(f"Error: Master USD file not found at {master_usd}")
        sys.exit(1)
        
    # Render Front View: simple reference USDA with no rotation
    temp_front_path = os.path.join(repo_root, "temp_render_front.usda")
    temp_front_content = f"""#usda 1.0
(
    upAxis = "Y"
)
def Xform "Model" (
    references = @{master_usd}@
)
{{
}}
"""
    with open(temp_front_path, "w") as f:
        f.write(temp_front_content)
        
    print("Rendering front view...")
    subprocess.run([
        "/usr/bin/usdrecord",
        "--renderer", "Metal",
        temp_front_path,
        render_front
    ], check=True)
    
    if os.path.exists(temp_front_path):
        os.remove(temp_front_path)
        
    # Render Angled View: reference USDA with Y/X rotations
    temp_angled_path = os.path.join(repo_root, "temp_render_angled.usda")
    temp_angled_content = f"""#usda 1.0
(
    upAxis = "Y"
)
def Xform "Model" (
    references = @{master_usd}@
)
{{
    double3 xformOp:rotateXYZ = (15, 30, 0)
    uniform token[] xformOpOrder = ["xformOp:rotateXYZ"]
}}
"""
    with open(temp_angled_path, "w") as f:
        f.write(temp_angled_content)
        
    print("Rendering angled view...")
    subprocess.run([
        "/usr/bin/usdrecord",
        "--renderer", "Metal",
        temp_angled_path,
        render_angled
    ], check=True)
    
    if os.path.exists(temp_angled_path):
        os.remove(temp_angled_path)
        
    print("Rendering complete!")

def post_process_render(renders_dir):
    render_front = os.path.join(renders_dir, "render_front.png")
    render_silhouette = os.path.join(renders_dir, "render_silhouette.png")
    render_edges = os.path.join(renders_dir, "render_edges.png")
    
    print("Post-processing front render...")
    if not os.path.exists(render_front):
        print(f"Error: render_front.png not found at {render_front}")
        sys.exit(1)
        
    img = Image.open(render_front)
    w, h = img.size
    
    # 1. Create binary silhouette mask where foreground is white (255) and background is black (0)
    # Background is assumed to be transparent (alpha = 0)
    silhouette_img = Image.new("L", (w, h), 0)
    sil_pixels = silhouette_img.load()
    img_pixels = img.load()
    
    for y in range(h):
        for x in range(w):
            r, g, b, a = img_pixels[x, y]
            if a > 0:
                sil_pixels[x, y] = 255
            else:
                sil_pixels[x, y] = 0
                
    silhouette_img.save(render_silhouette)
    print(f"Saved silhouette mask to: {render_silhouette}")
    
    # 2. Create edge detection map using FIND_EDGES
    # Composite the image onto a black background to avoid edge artifacts from alpha channel
    black_bg = Image.new("RGB", (w, h), (0, 0, 0))
    black_bg.paste(img, (0, 0), img)
    gray_img = black_bg.convert("L")
    edges_img = gray_img.filter(ImageFilter.FIND_EDGES)
    edges_img.save(render_edges)
    print(f"Saved edge map to: {render_edges}")

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.abspath(os.path.join(script_dir, ".."))
    
    asset_dir = os.path.join(repo_root, "generated", "mech_assets", "reference_fabric_001")
    texture_dir = os.path.join(asset_dir, "textures")
    renders_dir = os.path.join(asset_dir, "renders")
    
    # Generate textures
    generate_textures(texture_dir)
    
    # Run renderer
    run_renderer(repo_root, renders_dir)
    
    # Post-process
    post_process_render(renders_dir)

if __name__ == "__main__":
    main()
