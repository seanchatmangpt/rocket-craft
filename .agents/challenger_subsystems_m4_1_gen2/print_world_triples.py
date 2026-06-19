import rdflib

g = rdflib.Graph()
g.parse("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl", format="turtle")

world = rdflib.URIRef("https://rocket-craft.io/ontology/ue4/gundam#GundamWorld")

print("Triples for GundamWorld:")
for s, p, o in g.triples((world, None, None)):
    print(f"P: {p}, O: {o}")
