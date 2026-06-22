import rdflib
from pyshacl import validate

g = rdflib.Graph()
g.parse('ontology/all_merged.ttl', format='turtle')

conforms, results_graph, results_text = validate(g, inference='rdfs')
print('Conforms:', conforms)
if not conforms:
    print(results_text)
