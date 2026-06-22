import rdflib

g = rdflib.Graph()
g.parse("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl", format="turtle")

world = rdflib.URIRef("https://rocket-craft.io/ontology/ue4/mecha#MechaWorld")

print("Triples for MechaWorld:")
for s, p, o in g.triples((world, None, None)):
    print(f"P: {p}, O: {o}")
