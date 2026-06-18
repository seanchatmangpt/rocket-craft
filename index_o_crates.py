import os
import json
import hashlib
import shutil
from pathlib import Path

# CONFIGURATION
CATALOGUE_ROOT = "/Users/sac/ggen/ontology_catalogue"
MARKETPLACE_ROOT = "/Users/sac/ggen/marketplace"
INDEX_JSON = os.path.join(MARKETPLACE_ROOT, "registry", "index.json")

def calculate_sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192):
            h.update(chunk)
    return h.hexdigest()

def index_crates():
    print(f"Indexing O-Crates in {CATALOGUE_ROOT}...")
    
    # Ensure registry path exists
    os.makedirs(os.path.join(MARKETPLACE_ROOT, "registry"), exist_ok=True)
    
    # Load or initialize index
    if os.path.exists(INDEX_JSON):
        with open(INDEX_JSON, 'r') as f:
            registry = json.load(f)
    else:
        registry = []

    # Traverse catalogue
    for root, dirs, files in os.walk(CATALOGUE_ROOT):
        rel_path = os.path.relpath(root, CATALOGUE_ROOT)
        if rel_path == ".":
            continue
            
        # If it's a top-level dir (the O-Crate root)
        if os.path.dirname(rel_path) == "":
            crate_id = rel_path
            
            # Aggregate stats
            ttl_count = 0
            rq_count = 0
            total_bytes = 0
            all_files = []
            
            for r, d, f in os.walk(root):
                for file in f:
                    full_path = os.path.join(r, file)
                    if file.endswith('.ttl'): ttl_count += 1
                    if file.endswith('.rq'): rq_count += 1
                    total_bytes += os.path.getsize(full_path)
                    all_files.append(full_path)
            
            # Generate hash of the entire crate
            crate_hash = hashlib.sha256()
            for f_path in sorted(all_files):
                crate_hash.update(calculate_sha256(f_path).encode())
            
            crate_meta = {
                "id": crate_id,
                "name": crate_id,
                "version": "1.0.0",
                "ontologyCount": ttl_count,
                "queryCount": rq_count,
                "sha256Hash": crate_hash.hexdigest(),
                "category": "ontology-crate"
            }
            
            # Update registry
            existing = next((item for item in registry if item["id"] == crate_id), None)
            if existing:
                registry.remove(existing)
            registry.append(crate_meta)

    # Save index
    with open(INDEX_JSON, 'w') as f:
        json.dump(registry, f, indent=2)
        
    print(f"Indexed {len(registry)} O-Crates into {INDEX_JSON}")

if __name__ == '__main__':
    index_crates()
