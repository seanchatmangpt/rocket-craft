import os
import re

replacements = [
    ('Mecha', 'Mecha'),
    ('mecha', 'mecha'),
    ('MECHA', 'MECHA'),
    ('Studio', 'Studio'),
    ('studio', 'studio'),
    ('GlobalCorp', 'GlobalCorp'),
    ('globalcorp', 'globalcorp'),
    ('BiotechMech', 'BiotechMech'),
    ('biotechmech', 'biotechmech'),
    ('BIOTECHMECH', 'BIOTECHMECH'),
    ('ArmorFrame', 'ArmorFrame'),
    ('armorframe', 'armorframe'),
    ('ArmorFrame', 'ArmorFrame'),
    ('armorframe', 'armorframe'),
    ('Alpha Walker', 'Alpha Walker'),
    ('Feral Walker', 'Feral Walker'),
    ('XM-01', 'XM-01'),
    ('XM-02', 'XM-02'),
    ('GruntUnit', 'GruntUnit'),
    ('ColonyRebels', 'ColonyRebels'),
    ('EarthAlliance', 'EarthAlliance'),
    ('CommandCenter', 'CommandCenter'),
    ('MechKit', 'MechKit'),
    ('ActionFigures', 'ActionFigures'),
    ('WinterProtocol', 'WinterProtocol'),
    ('winter_protocol', 'winter_protocol'),
    ('Winter_Protocol', 'Winter_Protocol')
]

exclude_dirs = {'.agents', '.git', 'target', 'node_modules', 'Build', 'Intermediate', 'Binaries', '.specify', 'Brm427'}

def replace_in_string(s):
    orig = s
    for old, new in replacements:
        s = s.replace(old, new)
    return s

def process_directory(root_dir):
    # First rename directories
    for root, dirs, files in os.walk(root_dir, topdown=False):
        for name in dirs:
            if name in exclude_dirs:
                continue
            new_name = replace_in_string(name)
            if new_name != name:
                os.rename(os.path.join(root, name), os.path.join(root, new_name))

    # Then rename files and process contents
    for root, dirs, files in os.walk(root_dir, topdown=False):
        dirs[:] = [d for d in dirs if d not in exclude_dirs and not d.startswith('.agents') and not d.startswith('.git')]
        
        for name in files:
            filepath = os.path.join(root, name)
            if not os.path.isfile(filepath):
                continue
            
            # process contents
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    content = f.read()
                new_content = replace_in_string(content)
                if new_content != content:
                    with open(filepath, 'w', encoding='utf-8') as f:
                        f.write(new_content)
                    print(f"Modified contents: {filepath}")
            except Exception:
                pass # skip binary

            # rename file
            new_name = replace_in_string(name)
            if new_name != name:
                new_filepath = os.path.join(root, new_name)
                os.rename(filepath, new_filepath)
                print(f"Renamed: {filepath} -> {new_filepath}")

if __name__ == '__main__':
    process_directory('.')
