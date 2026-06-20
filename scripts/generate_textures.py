import os
import numpy as np
from PIL import Image

def generate_noise_map(width, height, channels=1, frequency=10.0, base_val=0.5, amplitude=0.5):
    # simple white noise + slight smoothing for a procedural look
    noise = np.random.rand(height, width, channels) if channels > 1 else np.random.rand(height, width)
    
    # Scale between 0 and 255
    data = (base_val + (noise - 0.5) * amplitude)
    data = np.clip(data, 0.0, 1.0) * 255.0
    return data.astype(np.uint8)

def generate_normal_map(width, height):
    # Base normal is (128, 128, 255)
    img = np.zeros((height, width, 3), dtype=np.uint8)
    img[:,:,0] = 128 + np.random.randint(-10, 10, (height, width), dtype=np.int16)
    img[:,:,1] = 128 + np.random.randint(-10, 10, (height, width), dtype=np.int16)
    img[:,:,2] = 255
    return img

def main():
    out_dir = "/Users/sac/rocket-craft/.agents/sub_orch_implementation_aaa_ue4_mech_pack_001/textures"
    os.makedirs(out_dir, exist_ok=True)
    
    width, height = 2048, 2048
    
    maps = {
        "BaseColor": generate_noise_map(width, height, 3, base_val=0.5, amplitude=0.8),
        "Normal": generate_normal_map(width, height),
        "Roughness": generate_noise_map(width, height, 1, base_val=0.6, amplitude=0.4),
        "Metallic": generate_noise_map(width, height, 1, base_val=0.8, amplitude=0.2),
        "AO": generate_noise_map(width, height, 1, base_val=0.9, amplitude=0.1),
        "Emissive": generate_noise_map(width, height, 3, base_val=0.1, amplitude=0.5), # Slight glow
        "WearMask": generate_noise_map(width, height, 1, base_val=0.2, amplitude=0.4),
        "DamageMask": generate_noise_map(width, height, 1, base_val=0.1, amplitude=0.2),
        "PanelLineMask": generate_noise_map(width, height, 1, base_val=0.3, amplitude=0.6),
        "DecalMask": generate_noise_map(width, height, 1, base_val=0.1, amplitude=0.2)
    }
    
    for map_name, data in maps.items():
        if len(data.shape) == 2:
            img = Image.fromarray(data, 'L')
        else:
            img = Image.fromarray(data, 'RGB')
            
        out_path = os.path.join(out_dir, f"T_Mech_{map_name}.png")
        img.save(out_path)
        print(f"Generated {out_path}")

if __name__ == "__main__":
    main()
