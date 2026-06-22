#!/usr/bin/env python3
import os
import json
import math
import struct
import argparse

# Pure Python deterministic texture generator
# To avoid external dependencies (like PIL or NumPy), this writes simple uncompressed TGA/PPM files
# representing the evaluated material states across a UV mapped domain.

def write_ppm(filename, width, height, pixels):
    """Write RGB pixels to a PPM image file"""
    with open(filename, 'wb') as f:
        f.write(f"P6\n{width} {height}\n255\n".encode('ascii'))
        # pixels is a flat list of (r, g, b) tuples, 0-255
        pixel_data = bytearray()
        for r, g, b in pixels:
            pixel_data.append(int(r))
            pixel_data.append(int(g))
            pixel_data.append(int(b))
        f.write(pixel_data)

def write_pgm(filename, width, height, pixels):
    """Write grayscale pixels to a PGM image file"""
    with open(filename, 'wb') as f:
        f.write(f"P5\n{width} {height}\n255\n".encode('ascii'))
        # pixels is a flat list of intensity values, 0-255
        pixel_data = bytearray(int(p) for p in pixels)
        f.write(pixel_data)

# Deterministic Pseudo-Random Noise (very simplified for structural standing)
def pseudo_noise(x, y, scale=10.0, seed=1337):
    # A simple deterministic hash-based noise
    n = math.sin((x * scale + seed) * 12.9898 + (y * scale) * 78.233) * 43758.5453
    return n - math.floor(n)

def generate_textures_for_material(mat_def, width=512, height=512, output_dir="."):
    name = mat_def['name']
    hex_color = mat_def['base_color']
    r_base = int(hex_color[1:3], 16) / 255.0
    g_base = int(hex_color[3:5], 16) / 255.0
    b_base = int(hex_color[5:7], 16) / 255.0
    roughness_base = float(mat_def.get('roughness', 0.5))
    metallic_base = float(mat_def.get('metallic', 0.0))
    emissive_base = float(mat_def.get('emissive', 0.0))

    base_color_pixels = []
    roughness_pixels = []
    metallic_pixels = []
    normal_pixels = []
    emissive_pixels = []
    
    # Deterministic procedural projection loop
    for y in range(height):
        for x in range(width):
            u = x / float(width)
            v = y / float(height)
            
            # Procedural combination logic based on material name (simulating OSL execution)
            if name == "M_WhiteArmor":
                noise_val = pseudo_noise(u, v, scale=50.0)
                scratch = pseudo_noise(u, v, scale=150.0, seed=42) > 0.95
                
                r = r_base - noise_val * 0.05
                g = g_base - noise_val * 0.05
                b = b_base - noise_val * 0.05
                
                rough = roughness_base + (0.15 if scratch else 0.0)
                nx, ny, nz = (0.5, 0.5, 1.0)
                if scratch:
                    nx += (pseudo_noise(u, v, scale=200.0) - 0.5) * 0.1
                    ny += (pseudo_noise(u, v, scale=200.0, seed=11) - 0.5) * 0.1
                
                emi = emissive_base

            elif name == "M_DarkFrame":
                grit = pseudo_noise(u, v, scale=100.0)
                r = r_base + grit * 0.05
                g = g_base + grit * 0.05
                b = b_base + grit * 0.05
                
                rough = roughness_base - grit * 0.1
                nx, ny, nz = (0.5 + (grit - 0.5)*0.1, 0.5 + (grit - 0.5)*0.1, 1.0)
                emi = emissive_base

            elif name == "M_CyanBlade":
                energy = pseudo_noise(u, v, scale=10.0)
                r = r_base
                g = g_base * (0.8 + energy * 0.2)
                b = b_base * (0.8 + energy * 0.2)
                rough = roughness_base
                emi = emissive_base * (0.5 + energy * 0.5)
                nx, ny, nz = (0.5, 0.5, 1.0)

            elif name == "M_GoldVisor":
                hex_pat = 1.0 if (math.sin(u*100) * math.cos(v*100) > 0.0) else 0.0
                r = r_base * (0.9 + hex_pat * 0.1)
                g = g_base * (0.9 + hex_pat * 0.1)
                b = b_base * (0.9 + hex_pat * 0.1)
                rough = roughness_base + (0.05 if hex_pat == 0.0 else 0.0)
                emi = emissive_base + hex_pat * 0.2
                nx, ny, nz = (0.5 + hex_pat*0.01, 0.5 + hex_pat*0.01, 1.0)
            
            else:
                # Default generic projection
                r, g, b = r_base, g_base, b_base
                rough = roughness_base
                emi = emissive_base
                nx, ny, nz = (0.5, 0.5, 1.0)

            # Clamp and convert to 0-255
            base_color_pixels.append((
                max(0, min(255, r * 255)),
                max(0, min(255, g * 255)),
                max(0, min(255, b * 255))
            ))
            roughness_pixels.append(max(0, min(255, rough * 255)))
            metallic_pixels.append(max(0, min(255, metallic_base * 255)))
            emissive_pixels.append(max(0, min(255, emi * 255)))
            
            # Normalize and map Normal (-1 to 1) to (0 to 255)
            # Standard normal map is (x*0.5+0.5, y*0.5+0.5, z*0.5+0.5)
            nl = math.sqrt(nx*nx + ny*ny + nz*nz)
            normal_pixels.append((
                max(0, min(255, (nx/nl) * 255)),
                max(0, min(255, (ny/nl) * 255)),
                max(0, min(255, (nz/nl) * 255))
            ))

    # Write output maps deterministically
    write_ppm(os.path.join(output_dir, f"{name}_BaseColor.ppm"), width, height, base_color_pixels)
    write_pgm(os.path.join(output_dir, f"{name}_Roughness.pgm"), width, height, roughness_pixels)
    write_pgm(os.path.join(output_dir, f"{name}_Metallic.pgm"), width, height, metallic_pixels)
    write_ppm(os.path.join(output_dir, f"{name}_Normal.ppm"), width, height, normal_pixels)
    if emissive_base > 0.0:
        write_pgm(os.path.join(output_dir, f"{name}_Emissive.pgm"), width, height, emissive_pixels)

    print(f"Generated textures for {name} in {output_dir}")

def main():
    parser = argparse.ArgumentParser(description="Procedural Texture Combiner / Generator")
    parser.add_argument("--manifest", type=str, default="texture_manifest.json", help="Path to texture manifest")
    parser.add_argument("--output", type=str, default=".", help="Output directory")
    parser.add_argument("--res", type=int, default=512, help="Resolution of the generated textures")
    args = parser.parse_args()

    print(f"Loading manifest from {args.manifest}...")
    with open(args.manifest, 'r') as f:
        manifest = json.load(f)
    
    os.makedirs(args.output, exist_ok=True)
    
    for mat in manifest['materials']:
        print(f"Processing material: {mat['name']}")
        generate_textures_for_material(mat, width=args.res, height=args.res, output_dir=args.output)
        
    print("Texture generation complete. Verifiable artifacts written.")

if __name__ == "__main__":
    main()
