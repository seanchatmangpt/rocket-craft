import os
import re

terms_map = {
    r'\bMecha\b': 'Mecha',
    r'\bmecha\b': 'mecha',
    r'\bMECHA\b': 'MECHA',
    r'\bStudio\b': 'Studio',
    r'\bstudio\b': 'studio',
    r'\bGlobalCorp\b': 'GlobalCorp',
    r'\bglobalcorp\b': 'globalcorp',
    r'\bBiotechMech\b': 'BiotechMech',
    r'\bbiotechmech\b': 'biotechmech',
    r'\bBIOTECHMECH\b': 'BIOTECHMECH',
    r'\bArmorFrame\b': 'ArmorFrame',
    r'\barmorframe\b': 'armorframe',
    r'\bArmorFrame\b': 'ArmorFrame',
    r'\barmorframe\b': 'armorframe',
    r'\bAlpha Walker\b': 'Alpha Walker',
    r'\bFeral Walker\b': 'Feral Walker',
    r'\bXM-01\b': 'XM-01',
    r'\bXM-02\b': 'XM-02',
    r'\bGruntUnit\b': 'GruntUnit',
    r'\bColonyRebels\b': 'ColonyRebels',
    r'\bEarthAlliance\b': 'EarthAlliance',
    r'\bCommandCenter\b': 'CommandCenter',
    r'\bMechKit\b': 'MechKit',
    r'\bActionFigures\b': 'ActionFigures',
    r'\bWinterProtocol\b': 'WinterProtocol',
    r'\bwinter_protocol\b': 'winter_protocol',
    r'\bWinter_Protocol\b': 'Winter_Protocol'
}

exclude_dirs = {'.agents', '.git', 'target', 'node_modules', 'Build', 'Intermediate', 'Binaries', '.specify', 'Brm427'}

def replace_in_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        return False # skip binary
    except Exception:
        return False

    orig_content = content
    for pattern, repl in terms_map.items():
        content = re.sub(pattern, repl, content)

    if content != orig_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def process_directory(root_dir):
    # First rename directories
    for root, dirs, files in os.walk(root_dir, topdown=False):
        for name in dirs:
            if name in exclude_dirs:
                continue
            new_name = name
            for pattern, repl in terms_map.items():
                new_name = re.sub(pattern, repl, new_name)
            if new_name != name:
                os.rename(os.path.join(root, name), os.path.join(root, new_name))

    # Then rename files and process contents
    for root, dirs, files in os.walk(root_dir, topdown=False):
        # Exclude directories
        dirs[:] = [d for d in dirs if d not in exclude_dirs and not d.startswith('.agents') and not d.startswith('.git')]
        
        for name in files:
            filepath = os.path.join(root, name)
            if not os.path.isfile(filepath):
                continue
            
            # process contents
            replaced = replace_in_file(filepath)
            if replaced:
                print(f"Modified contents: {filepath}")

            # rename file
            new_name = name
            for pattern, repl in terms_map.items():
                new_name = re.sub(pattern, repl, new_name)
            if new_name != name:
                new_filepath = os.path.join(root, new_name)
                os.rename(filepath, new_filepath)
                print(f"Renamed: {filepath} -> {new_filepath}")

if __name__ == '__main__':
    process_directory('.')
