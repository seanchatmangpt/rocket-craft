import rdflib

with open("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl") as f:
    lines = f.readlines()
    for idx in range(208, 218):
        print(f"{idx+1}: {lines[idx]}", end="")

print("\n--- Parse with rdflib ---")
g = rdflib.Graph()
g.parse("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl", format="turtle")
for s, p, o in g.triples((None, None, None)):
    if "GundamWorld" in str(s):
        print(f"S: {s}, P: {p}, O: {o}")
