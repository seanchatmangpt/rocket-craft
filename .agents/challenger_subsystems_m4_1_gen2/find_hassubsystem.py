import rdflib

g = rdflib.Graph()
try:
    g.parse("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl", format="turtle")
except Exception as e:
    print(f"Parsing error: {e}")

print(f"Total triples loaded: {len(g)}")

for s, p, o in g:
    if "hasSubsystem" in str(p) or "hasSubsystem" in str(s) or "hasSubsystem" in str(o):
        print(f"S: {s}, P: {p}, O: {o}")
