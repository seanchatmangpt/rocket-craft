import os
import re
from collections import defaultdict
import json

# Advanced Regex Patterns
ttl_prefix_pattern = re.compile(r'(?:@prefix|PREFIX)\s+([^:]+):\s+<([^>]+)>', re.IGNORECASE)
ttl_class_pattern = re.compile(r'([a-zA-Z0-9_:-]+)\s+a\s+(?:owl:Class|rdfs:Class|sh:NodeShape)')
ttl_import_pattern = re.compile(r'owl:imports\s+<([^>]+)>')
ttl_subclass_pattern = re.compile(r'([a-zA-Z0-9_:-]+)\s+rdfs:subClassOf\s+([a-zA-Z0-9_:-]+)')
ttl_prop_pattern = re.compile(r'sh:path\s+([a-zA-Z0-9_:-]+)')

rq_query_type_pattern = re.compile(r'\b(SELECT|CONSTRUCT|ASK|DESCRIBE|INSERT|DELETE)\b', re.IGNORECASE)
rq_select_vars_pattern = re.compile(r'SELECT\s+(?:DISTINCT\s+)?((?:\?[a-zA-Z0-9_]+\s*)+)', re.IGNORECASE)

def analyze_file(filepath, ext):
    metadata = {
        'file': filepath,
        'prefixes': set(),
        'classes': set(),
        'imports': set(),
        'subclasses': set(),
        'properties': set(),
        'query_type': None,
        'select_vars': set(),
        'size_bytes': 0
    }
    try:
        metadata['size_bytes'] = os.path.getsize(filepath)
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read(500000) # Up to 500KB per file
            
            for match in ttl_prefix_pattern.finditer(content):
                metadata['prefixes'].add(f"{match.group(1)}: <{match.group(2)}>")
                
            if ext == 'ttl':
                for match in ttl_class_pattern.finditer(content):
                    metadata['classes'].add(match.group(1))
                for match in ttl_import_pattern.finditer(content):
                    metadata['imports'].add(match.group(1))
                for match in ttl_subclass_pattern.finditer(content):
                    metadata['subclasses'].add(f"{match.group(1)} -> {match.group(2)}")
                for match in ttl_prop_pattern.finditer(content):
                    metadata['properties'].add(match.group(1))
                    
            elif ext == 'rq':
                qt_match = rq_query_type_pattern.search(content)
                if qt_match:
                    metadata['query_type'] = qt_match.group(1).upper()
                vars_match = rq_select_vars_pattern.search(content)
                if vars_match:
                    vars_str = vars_match.group(1)
                    metadata['select_vars'].update([v.strip() for v in vars_str.split() if v.strip()])
                    
    except Exception as e:
        pass
        
    # Convert sets to lists for easy JSON serialization if needed later
    metadata['prefixes'] = list(metadata['prefixes'])
    metadata['classes'] = list(metadata['classes'])
    metadata['imports'] = list(metadata['imports'])
    metadata['subclasses'] = list(metadata['subclasses'])
    metadata['properties'] = list(metadata['properties'])
    metadata['select_vars'] = list(metadata['select_vars'])
    
    return metadata

def main():
    if not os.path.exists('all_semantic_files.txt'):
        print("File list not found.")
        return

    with open('all_semantic_files.txt', 'r') as f:
        files = [line.strip() for line in f if line.strip()]

    projects = defaultdict(lambda: {'ttl_count': 0, 'rq_count': 0, 'files_meta': []})
    
    print(f"Deep analyzing {len(files)} files...")
    
    for path in files:
        parts = path.split(os.sep)
        if len(parts) > 3 and parts[1] == 'Users':
            project_name = parts[3]
            ext = path.split('.')[-1]
            proj_data = projects[project_name]
            
            meta = analyze_file(path, ext)
            if ext == 'ttl':
                proj_data['ttl_count'] += 1
            elif ext == 'rq':
                proj_data['rq_count'] += 1
                
            proj_data['files_meta'].append(meta)

    # Sort projects by total files descending
    sorted_projects = sorted(projects.items(), key=lambda x: (x[1]['ttl_count'] + x[1]['rq_count']), reverse=True)

    print("Generating comprehensive dossiers...")
    
    for proj, data in sorted_projects:
        # Aggregate data for the dossier
        all_prefixes = set()
        all_classes = set()
        all_imports = set()
        all_subclasses = set()
        all_properties = set()
        query_types = defaultdict(int)
        all_select_vars = set()
        
        for meta in data['files_meta']:
            all_prefixes.update(meta['prefixes'])
            all_classes.update(meta['classes'])
            all_imports.update(meta['imports'])
            all_subclasses.update(meta['subclasses'])
            all_properties.update(meta['properties'])
            if meta['query_type']:
                query_types[meta['query_type']] += 1
            all_select_vars.update(meta['select_vars'])

        # Write Dossier
        # We sanitize project name for filesystem
        safe_proj = "".join([c if c.isalnum() else "_" for c in proj])
        dossier_path = os.path.join("PROJECT_RESEARCH", f"{safe_proj}.md")
        
        with open(dossier_path, 'w', encoding='utf-8') as out:
            out.write(f"# Research Dossier: `{proj}`\n\n")
            out.write(f"**Total Files:** {data['ttl_count']} Ontologies (.ttl) | {data['rq_count']} Queries (.rq)\n")
            out.write(f"**Total Volume:** {data['ttl_count'] + data['rq_count']} files\n\n")
            
            out.write("## 1. Architectural Dependencies (owl:imports)\n")
            if all_imports:
                for imp in sorted(list(all_imports)):
                    out.write(f"- `<{imp}>`\n")
            else:
                out.write("- *No external ontologies imported.*\n")
            out.write("\n")
            
            out.write("## 2. Core Vocabularies (Prefixes)\n")
            # Show top 25 prefixes
            for p in sorted(list(all_prefixes))[:25]:
                out.write(f"- `{p}`\n")
            if len(all_prefixes) > 25:
                out.write(f"- *...and {len(all_prefixes) - 25} more.*\n")
            out.write("\n")
            
            out.write("## 3. Domain Taxonomy & Entities\n")
            out.write("### Base Classes & Shapes\n")
            for c in sorted(list(all_classes))[:30]:
                out.write(f"- `{c}`\n")
            if len(all_classes) > 30:
                out.write(f"- *...and {len(all_classes) - 30} more.*\n")
                
            out.write("\n### Inheritances (Subclasses)\n")
            for sub in sorted(list(all_subclasses))[:20]:
                out.write(f"- `{sub}`\n")
            if len(all_subclasses) > 20:
                out.write(f"- *...and {len(all_subclasses) - 20} more.*\n")
                
            out.write("\n### Constrained Properties (SHACL Paths)\n")
            for prop in sorted(list(all_properties))[:20]:
                out.write(f"- `{prop}`\n")
            if len(all_properties) > 20:
                out.write(f"- *...and {len(all_properties) - 20} more.*\n")
            out.write("\n")
            
            out.write("## 4. Extraction Layer (SPARQL)\n")
            if query_types:
                out.write(f"- **Query Types Executed:** {dict(query_types)}\n")
            else:
                out.write("- *No queries executed in this project.*\n")
                
            out.write("\n### Projected Variables (SELECT ?var)\n")
            if all_select_vars:
                out.write("This project actively projects the following variables into code/templates:\n")
                # Join vars with backticks
                vars_str = ", ".join([f"`{v}`" for v in sorted(list(all_select_vars))])
                out.write(f"> {vars_str}\n")
            else:
                out.write("- *No specific projection variables identified.*\n")
            out.write("\n")
            
            out.write("## 5. File Inventory\n")
            out.write("<details>\n<summary>Click to expand all files</summary>\n\n")
            for meta in sorted(data['files_meta'], key=lambda x: x['file']):
                out.write(f"- `{meta['file']}` ({meta['size_bytes']} bytes)\n")
            out.write("\n</details>\n")

    print(f"Generated {len(sorted_projects)} research dossiers in PROJECT_RESEARCH/")

if __name__ == '__main__':
    main()
