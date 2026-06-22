#!/usr/bin/env python3
import os
import re
import math
import random
import sys

# We will generate raw uncompressed PPM and PGM, then if necessary, we can use PIL
# However, the requirement is `.png`. Let's see if we can just write `.png` by using PIL.
# The environment clearly has Pillow because we ran PIL successfully earlier.
try:
    from PIL import Image, ImageDraw, ImageFilter
except ImportError:
    print("PIL not found")
    sys.exit(1)

def deterministic_noise(width, height, scale, seed_val):
    random.seed(seed_val)
    img = Image.new('L', (width, height))
    pixels = img.load()
    for y in range(height):
        for x in range(width):
            val = int((math.sin(x/scale) + math.cos(y/scale)) * 127 + 128)
            noise_val = random.randint(-20, 20)
            pixels[x, y] = max(0, min(255, val + noise_val))
    return img.filter(ImageFilter.GaussianBlur(radius=scale/4))

def generate_panel_lines(width, height, seed_val):
    random.seed(seed_val)
    img = Image.new('L', (width, height), color=255)
    draw = ImageDraw.Draw(img)
    spacing = width // random.randint(4, 10)
    for i in range(0, width, spacing):
        draw.line([(i, 0), (i, height)], fill=0, width=random.randint(2, 6))
    for j in range(0, height, spacing):
        draw.line([(0, j), (width, j)], fill=0, width=random.randint(2, 6))
    return img

def generate_masks(width, height, seed_val):
    random.seed(seed_val)
    img = Image.new('RGB', (width, height))
    pixels = img.load()
    for y in range(height):
        for x in range(width):
            r = 255 if (x // 64) % 2 == 0 else 0
            g = 255 if (y // 64) % 2 == 0 else 0
            b = random.randint(100, 150)
            pixels[x, y] = (r, g, b)
    return img

def generate_roughness(width, height, seed_val):
    base_noise = deterministic_noise(width, height, 32.0, seed_val)
    img = Image.new('L', (width, height))
    base_pixels = base_noise.load()
    pixels = img.load()
    for y in range(height):
        for x in range(width):
            val = base_pixels[x, y]
            pixels[x, y] = int(val * 0.5 + 50)
    return img

def generate_normal(width, height, seed_val):
    img = Image.new('RGB', (width, height), color=(128, 128, 255))
    pixels = img.load()
    for y in range(height):
        for x in range(width):
            nx = int(math.sin((x+seed_val)/16.0) * 32 + 128)
            ny = int(math.cos((y+seed_val)/16.0) * 32 + 128)
            pixels[x, y] = (nx, ny, 255)
    return img

def generate_wear(width, height, seed_val):
    img = deterministic_noise(width, height, 16.0, seed_val)
    pixels = img.load()
    for y in range(height):
        for x in range(width):
            if pixels[x, y] > 180:
                pixels[x, y] = 255
            else:
                pixels[x, y] = 0
    return img.filter(ImageFilter.GaussianBlur(radius=2))

def main():
    ontology_path = "/Users/sac/rocket-craft/ontology/all_merged.ttl"
    out_dir = "/Users/sac/rocket-craft/generated/mech_assets/reference_fabric_001/textures"
    os.makedirs(out_dir, exist_ok=True)
    
    with open(ontology_path, "r") as f:
        content = f.read()
    
    # Extract materials from ontology law
    materials = set(re.findall(r'mud:(M_[a-zA-Z0-9_]+)\s+rdf:type\s+mud:Material', content))
    
    if not materials:
        print("No materials found in source law!")
        # Fallback to the required textures generically
        materials = {"GenericMech"}
    
    res = 512
    
    for mat in materials:
        print(f"Generating programmatic textures for {mat} driven by source law...")
        import hashlib
        h = hashlib.md5(mat.encode('utf-8')).hexdigest()
        seed_val = int(h, 16) % 100000
        
        generate_masks(res, res, seed_val).save(os.path.join(out_dir, f"{mat}_Masks.png"))
        generate_panel_lines(res, res, seed_val+1).save(os.path.join(out_dir, f"{mat}_PanelLines.png"))
        generate_roughness(res, res, seed_val+2).save(os.path.join(out_dir, f"{mat}_Roughness.png"))
        generate_normal(res, res, seed_val+3).save(os.path.join(out_dir, f"{mat}_Normal.png"))
        generate_wear(res, res, seed_val+4).save(os.path.join(out_dir, f"{mat}_Wear.png"))
        
    print("Texture generation complete.")

if __name__ == "__main__":
    main()
