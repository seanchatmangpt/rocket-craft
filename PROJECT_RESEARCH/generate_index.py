import os
import glob
import re

def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))
    output_file = os.path.join(base_dir, "README.md")
    
    md_files = glob.glob(os.path.join(base_dir, "*.md"))
    # Exclude README.md if it already exists in the list
    md_files = [f for f in md_files if os.path.basename(f) != "README.md"]
    
    # Sort files alphabetically by filename
    md_files.sort(key=lambda x: os.path.basename(x).lower())

    dossiers = []
    
    for file_path in md_files:
        filename = os.path.basename(file_path)
        title = filename.replace(".md", "").replace("_", " ").title()
        
        # Try to read the first heading from the file to use as title
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("# "):
                        title = line[2:].strip()
                        break
        except Exception:
            pass
            
        # Get file size to display
        size_bytes = os.path.getsize(file_path)
        size_kb = size_bytes / 1024
        
        dossiers.append({
            "filename": filename,
            "title": title,
            "size_kb": size_kb
        })

    with open(output_file, "w", encoding="utf-8") as f:
        f.write("# Master Index: The Semantic Universe\n\n")
        f.write("This index maps the entire local Semantic Universe ($A = \\mu(O^*)$), cross-linking all project dossiers within the architecture.\n\n")
        f.write("## Project Dossiers\n\n")
        f.write("| Dossier | File | Size (KB) |\n")
        f.write("|---------|------|-----------|\n")
        
        for d in dossiers:
            f.write(f"| [{d['title']}](./{d['filename']}) | `{d['filename']}` | {d['size_kb']:.2f} |\n")

    print(f"Generated {output_file} with {len(dossiers)} entries.")

if __name__ == "__main__":
    main()
