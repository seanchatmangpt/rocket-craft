import os
import re
from collections import defaultdict

# Simplified, faster regex
ttl_prefix_pattern = re.compile(r'(?:@prefix|PREFIX)\s+([^:]+):\s+<([^>]+)>', re.IGNORECASE)
ttl_class_pattern = re.compile(r'([a-zA-Z0-9_:-]+)\s+a\s+(?:owl:Class|rdfs:Class|sh:NodeShape)')
rq_query_type_pattern = re.compile(r'\b(SELECT|CONSTRUCT|ASK|DESCRIBE|INSERT|DELETE)\b', re.IGNORECASE)
rq_select_vars_pattern = re.compile(r'SELECT\s+(?:DISTINCT\s+)?((?:\?[a-zA-Z0-9_]+\s*)+)', re.IGNORECASE)

def main():
    if not os.path.exists('all_semantic_files.txt'):
        print("File list not found.")
        return

    with open('all_semantic_files.txt', 'r') as f:
        files = [line.strip() for line in f if line.strip()]

    print(f"Fast analyzing {len(files)} files...")
    
    # Group files by project first
    projects = defaultdict(list)
    for path in files:
        parts = path.split(os.sep)
        if len(parts) > 3 and parts[1] == 'Users':
            project_name = parts[3]
            projects[project_name].append(path)

    os.makedirs("PROJECT_RESEARCH", exist_ok=True)

    for proj, proj_files in projects.items():
        all_prefixes = set()
        all_classes = set()
        query_types = defaultdict(int)
        all_select_vars = set()
        ttl_count = 0
        rq_count = 0
        file_inventory = []

        for path in proj_files:
            ext = path.split('.')[-1]
            try:
                size = os.path.getsize(path)
                file_inventory.append(f"- `{path}` ({size} bytes)")
                
                if ext == 'ttl':
                    ttl_count += 1
                elif ext == 'rq':
                    rq_count += 1

                # Only parse if file is reasonably small (< 500KB) to prevent lockups
                if size < 500000:
                    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                        
                        for match in ttl_prefix_pattern.finditer(content):
                            all_prefixes.add(f"{match.group(1)}: <{match.group(2)}>")
                            
                        if ext == 'ttl':
                            for match in ttl_class_pattern.finditer(content):
                                all_classes.add(match.group(1))
                        elif ext == 'rq':
                            qt_match = rq_query_type_pattern.search(content)
                            if qt_match:
                                query_types[qt_match.group(1).upper()] += 1
                            vars_match = rq_select_vars_pattern.search(content)
                            if vars_match:
                                vars_str = vars_match.group(1)
                                all_select_vars.update([v.strip() for v in vars_str.split() if v.strip()])
            except Exception as e:
                pass

        # Write Dossier directly
        safe_proj = "".join([c if c.isalnum() else "_" for c in proj])
        dossier_path = os.path.join("PROJECT_RESEARCH", f"{safe_proj}.md")
        
        with open(dossier_path, 'w', encoding='utf-8') as out:
            out.write(f"# Research Dossier: `{proj}`\n\n")
            out.write(f"**Total Files:** {ttl_count} Ontologies (.ttl) | {rq_count} Queries (.rq)\n")
            out.write(f"**Total Volume:** {ttl_count + rq_count} files\n\n")
            
            out.write("## 1. Core Vocabularies (Prefixes)\n")
            for p in sorted(list(all_prefixes))[:50]:
                out.write(f"- `{p}`\n")
            if len(all_prefixes) > 50:
                out.write(f"- *...and {len(all_prefixes) - 50} more.*\n")
            out.write("\n")
            
            out.write("## 2. Domain Taxonomy & Entities\n")
            out.write("### Base Classes & Shapes\n")
            for c in sorted(list(all_classes))[:50]:
                out.write(f"- `{c}`\n")
            if len(all_classes) > 50:
                out.write(f"- *...and {len(all_classes) - 50} more.*\n")
            out.write("\n")
            
            out.write("## 3. Extraction Layer (SPARQL)\n")
            if query_types:
                out.write(f"- **Query Types Executed:** {dict(query_types)}\n")
            else:
                out.write("- *No queries executed in this project.*\n")
                
            out.write("\n### Projected Variables (SELECT ?var)\n")
            if all_select_vars:
                out.write("This project actively projects the following variables into code/templates:\n")
                vars_str = ", ".join([f"`{v}`" for v in sorted(list(all_select_vars))])
                out.write(f"> {vars_str}\n")
            else:
                out.write("- *No specific projection variables identified.*\n")
            out.write("\n")
            
            out.write("## 4. File Inventory\n")
            out.write("<details>\n<summary>Click to expand all files</summary>\n\n")
            for item in sorted(file_inventory):
                out.write(f"{item}\n")
            out.write("\n</details>\n")

    print(f"Generated {len(projects)} research dossiers in PROJECT_RESEARCH/")

if __name__ == '__main__':
    main()
