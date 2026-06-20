import os
import glob

def merge_ontologies():
    source_dir = "/Users/sac/rocket-craft/ontology/source_law/"
    output_file = "/Users/sac/rocket-craft/ontology/all_merged.ttl"
    
    # Ensure source directory exists
    if not os.path.exists(source_dir):
        print(f"Source directory {source_dir} not found.")
        return
        
    ttl_files = glob.glob(os.path.join(source_dir, "*.ttl"))
    ttl_files.sort()
    
    with open(output_file, 'w') as out:
        out.write("# GENERATED FILE - DO NOT EDIT DIRECTLY\n")
        out.write("# This file is cleanly compiled from ontology/source_law/*.ttl\n\n")
        
        for ttl_file in ttl_files:
            with open(ttl_file, 'r') as f:
                out.write(f"# --- Source: {os.path.basename(ttl_file)} ---\n")
                out.write(f.read())
                out.write("\n\n")
                
    print(f"Cleanly merged {len(ttl_files)} files into {output_file}")

if __name__ == "__main__":
    merge_ontologies()
