import rdflib
import glob
from pyshacl import validate

data_graph = rdflib.Graph()
for f in glob.glob("/Users/sac/.ggen/packs/eden_server/ontology/*.ttl"):
    if "validation_shapes.ttl" not in f:
        data_graph.parse(f, format="turtle")

shacl_graph = rdflib.Graph()
shacl_graph.parse("/Users/sac/.ggen/packs/eden_server/ontology/validation_shapes.ttl", format="turtle")

conforms, results_graph, results_text = validate(
    data_graph,
    shacl_graph=shacl_graph,
    inference='rdfs',
    serialize_results=True
)

print("Conforms:", conforms)
print(results_text)
