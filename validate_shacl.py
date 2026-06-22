import glob
import rdflib
from pyshacl import validate

g = rdflib.Graph()
for f in glob.glob('ontology/*.ttl') + glob.glob('ontology/source_law/*.ttl'):
    if "all_merged.ttl" in f or "anti_llm" in f:
        continue # Avoid double loading if all_merged contains everything
    g.parse(f, format='turtle')

conforms, results_graph, results_text = validate(g, inference='rdfs')
print('Conforms:', conforms)
if not conforms:
    print(results_text)
