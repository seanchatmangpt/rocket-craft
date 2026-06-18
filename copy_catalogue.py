import os
import shutil

def main():
    if not os.path.exists('all_semantic_files.txt'):
        print("all_semantic_files.txt not found.")
        return

    target_base = "/Users/sac/ggen/ontology_catalogue"

    with open('all_semantic_files.txt', 'r') as f:
        files = [line.strip() for line in f if line.strip()]

    copied = 0
    errors = 0

    for src_path in files:
        if not os.path.exists(src_path):
            continue

        # Maintain directory structure inside the catalogue to avoid naming collisions
        # Strip the leading '/Users/sac/' to create a relative path
        rel_path = src_path.replace("/Users/sac/", "", 1)
        if rel_path.startswith("/"): # Fallback if it wasn't in /Users/sac
            rel_path = rel_path.lstrip("/")
            
        dest_path = os.path.join(target_base, rel_path)
        
        try:
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy2(src_path, dest_path)
            copied += 1
        except Exception as e:
            errors += 1

    print(f"Catalogue consolidation complete. Copied {copied} files. {errors} errors.")

if __name__ == '__main__':
    main()
