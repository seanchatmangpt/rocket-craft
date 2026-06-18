import os
import re
from collections import defaultdict
import json

ttl_prefix_pattern = re.compile(r'(?:@prefix|PREFIX)\s+([^:]+):\s+<([^>]+)>', re.IGNORECASE)
ttl_class_pattern = re.compile(r'([a-zA-Z0-9_:]+)\s+a\s+(?:owl:Class|rdfs:Class|sh:NodeShape)')
rq_query_type_pattern = re.compile(r'\b(SELECT|CONSTRUCT|ASK|DESCRIBE|INSERT|DELETE)\b', re.IGNORECASE)

def analyze_file(filepath, ext):
    metadata = {
        'prefixes': set(),
        'classes': set(),
        'query_type': None
    }
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read(100000)
            for match in ttl_prefix_pattern.finditer(content):
                metadata['prefixes'].add(f"{match.group(1)}: <{match.group(2)}>\n")
            if ext == 'ttl':
                for match in ttl_class_pattern.finditer(content):
                    metadata['classes'].add(match.group(1))
            elif ext == 'rq':
                qt_match = rq_query_type_pattern.search(content)
                if qt_match:
                    metadata['query_type'] = qt_match.group(1).upper()
    except Exception as e:
        pass
        
    metadata['prefixes'] = list(metadata['prefixes'])
    metadata['classes'] = list(metadata['classes'])
    return metadata

def main():
    if not os.path.exists('all_semantic_files.txt'):
        print("File list not found.")
        return

    with open('all_semantic_files.txt', 'r') as f:
        files = [line.strip() for line in f if line.strip()]

    projects = defaultdict(lambda: {'ttl_count': 0, 'rq_count': 0, 'sample_metadata': []})
    
    for path in files:
        parts = path.split(os.sep)
        if len(parts) > 3 and parts[1] == 'Users':
            project_name = parts[3]
            ext = path.split('.')[-1]
            proj_data = projects[project_name]
            
            if ext == 'ttl':
                proj_data['ttl_count'] += 1
                if proj_data['ttl_count'] <= 50: # Sample up to 50
                    meta = analyze_file(path, ext)
                    proj_data['sample_metadata'].append(meta)
            elif ext == 'rq':
                proj_data['rq_count'] += 1
                if proj_data['rq_count'] <= 50:
                    meta = analyze_file(path, ext)
                    proj_data['sample_metadata'].append(meta)

    sorted_projects = sorted(projects.items(), key=lambda x: (x[1]['ttl_count'] + x[1]['rq_count']), reverse=True)

    with open('SEMANTIC_UNIVERSE_REPORT.md', 'w') as out:
        out.write("# The Semantic Universe\n")
        out.write("## Global Analysis of 5,420 RDF & SPARQL Files\n\n")
        out.write(f"Total Top-Level Projects Discovered: {len(sorted_projects)}\n\n")
        out.write("### Project Breakdown\n")
        out.write("| Project | .ttl Files | .rq Files | Total | Dominant Focus |\n")
        out.write("|---|---|---|---|---|\n")
        
        for proj, data in sorted_projects:
            total = data['ttl_count'] + data['rq_count']
            focus = "Ontology (Schema/Data)" if data['ttl_count'] > data['rq_count'] else "Extraction (Queries)"
            out.write(f"| `{proj}` | {data['ttl_count']} | {data['rq_count']} | **{total}** | {focus} |\n")
            
        out.write("\n---\n## Deep Dive by Project\n\n")
        for proj, data in sorted_projects[:25]:
            out.write(f"### 🪐 Project: `{proj}`\n")
            out.write(f"- **Volume:** {data['ttl_count']} TTLs, {data['rq_count']} RQs\n")
            all_prefixes = set()
            all_classes = set()
            query_types = defaultdict(int)
            for meta in data['sample_metadata']:
                all_prefixes.update(meta['prefixes'])
                all_classes.update(meta['classes'])
                if meta['query_type']:
                    query_types[meta['query_type']] += 1
            if all_prefixes:
                out.write("- **Core Vocabularies:**\n")
                for p in list(all_prefixes)[:15]:
                    out.write(f"  - `{p.strip()}`\n")
            if all_classes:
                out.write("- **Domain Entities (Classes/Shapes):**\n")
                for c in list(all_classes)[:15]:
                    out.write(f"  - `{c}`\n")
            if query_types:
                out.write(f"- **Query Strategies:** {dict(query_types)}\n")
            out.write("\n")

if __name__ == '__main__':
    main()
