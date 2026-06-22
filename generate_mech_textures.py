import os
from PIL import Image, ImageDraw, ImageFilter
import math
import random

def generate_noise(width, height, scale, octaves=1, persistence=0.5, lacunarity=2.0):
    # A simple pseudo-random noise generator just to get some variation
    # Real perlin noise would be better, but this works without external libs other than PIL
    img = Image.new('L', (width, height))
    pixels = img.load()
    
    for y in range(height):
        for x in range(width):
            # very basic chaotic function
            val = int((math.sin(x/scale) + math.cos(y/scale)) * 127 + 128)
            noise_val = random.randint(-20, 20)
            pixels[x, y] = max(0, min(255, val + noise_val))
            
    # blur to make it smooth
    return img.filter(ImageFilter.GaussianBlur(radius=scale/4))

def generate_panel_lines(width, height):
    img = Image.new('L', (width, height), color=255)
    draw = ImageDraw.Draw(img)
    
    # draw some grid-like panel lines
    for i in range(0, width, width // 8):
        draw.line([(i, 0), (i, height)], fill=0, width=4)
        
    for j in range(0, height, height // 8):
        draw.line([(0, j), (width, j)], fill=0, width=4)
        
    # add some diagonals
    draw.line([(0, 0), (width, height)], fill=0, width=2)
    draw.line([(0, height), (width, 0)], fill=0, width=2)
    
    return img

def generate_masks(width, height):
    # Mask usually has RGB channels used for different things (e.g. R=Metal, G=Wear, B=Paint)
    img = Image.new('RGB', (width, height))
    pixels = img.load()
    
    for y in range(height):
        for x in range(width):
            r = 255 if (x // 64) % 2 == 0 else 0
            g = 255 if (y // 64) % 2 == 0 else 0
            b = 128
            pixels[x, y] = (r, g, b)
            
    return img

def generate_roughness(width, height):
    # Roughness map, grayscale
    base_noise = generate_noise(width, height, 32.0)
    
    # Increase contrast
    img = Image.new('L', (width, height))
    base_pixels = base_noise.load()
    pixels = img.load()
    
    for y in range(height):
        for x in range(width):
            val = base_pixels[x, y]
            # make it mostly shiny (low roughness) with some rough patches
            pixels[x, y] = int(val * 0.5 + 50)
            
    return img

def generate_normal(width, height):
    # Base flat normal is (128, 128, 255)
    img = Image.new('RGB', (width, height), color=(128, 128, 255))
    pixels = img.load()
    
    # add some bumpiness
    for y in range(height):
        for x in range(width):
            nx = int(math.sin(x/16.0) * 32 + 128)
            ny = int(math.cos(y/16.0) * 32 + 128)
            pixels[x, y] = (nx, ny, 255)
            
    return img

def generate_wear(width, height):
    # Wear map, grayscale - usually edges are white, rest is black.
    # We'll just generate some patchy noise
    img = generate_noise(width, height, 16.0)
    pixels = img.load()
    
    for y in range(height):
        for x in range(width):
            if pixels[x, y] > 180:
                pixels[x, y] = 255
            else:
                pixels[x, y] = 0
                
    return img.filter(ImageFilter.GaussianBlur(radius=2))

def main():
    out_dir = '/Users/sac/rocket-craft/final_mech_asset/textures'
    os.makedirs(out_dir, exist_ok=True)
    
    res = 1024 # 1k textures
    
    print("Generating masks...")
    generate_masks(res, res).save(os.path.join(out_dir, 'T_Mech_Masks.png'))
    
    print("Generating panel lines...")
    generate_panel_lines(res, res).save(os.path.join(out_dir, 'T_Mech_PanelLines.png'))
    
    print("Generating roughness...")
    generate_roughness(res, res).save(os.path.join(out_dir, 'T_Mech_Roughness.png'))
    
    print("Generating normal...")
    generate_normal(res, res).save(os.path.join(out_dir, 'T_Mech_Normal.png'))
    
    print("Generating wear...")
    generate_wear(res, res).save(os.path.join(out_dir, 'T_Mech_Wear.png'))
    
    print("All textures generated successfully.")

if __name__ == '__main__':
    main()
