import rdflib
try:
    from pyshacl import validate
except ImportError:
    import sys
    print("pyshacl not installed")
    sys.exit(0)

# Load data and shapes
data_graph = rdflib.Graph()
files = [
    "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/reflection.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/blueprints.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/typestates.ttl"
]
for f in files:
    data_graph.parse(f, format="turtle")

# Add the violation manually to data graph
extra_ttl = """
@prefix ue4: <https://rocket-craft.io/ontology/ue4/> .
@prefix gundam: <https://rocket-craft.io/ontology/ue4/gundam#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

gundam:SimBodyZeroMass a ue4:URigidBody ;
    rdfs:label "SimBodyZeroMass" ;
    ue4:physicsType ue4:PhysType_Simulated ;
    ue4:massKg 0.0 .
"""
data_graph.parse(data=extra_ttl, format="turtle")

shacl_graph = rdflib.Graph()
shacl_graph.parse("/Users/sac/rocket-craft/ggen-validation-tests/shacl/validation.shacl.ttl", format="turtle")

conforms, results_graph, results_text = validate(data_graph, shacl_graph=shacl_graph)
print("Conforms:", conforms)
print("Results Text:")
print(results_text)
