import rdflib
import glob

g = rdflib.Graph()
for f in glob.glob("/Users/sac/.ggen/packs/eden_server/ontology/*.ttl"):
    g.parse(f, format="turtle")

print("Total triples:", len(g))

# Find all instances of mars:DimensionalAsset
query = """
SELECT ?s WHERE {
    ?s a <https://ggen.io/ontology/mars-market/DimensionalAsset> .
} ORDER BY ?s
"""

for row in g.query(query):
    print("Instance of DimensionalAsset:", row[0])
