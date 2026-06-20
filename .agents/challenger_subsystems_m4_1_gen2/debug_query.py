import rdflib

g = rdflib.Graph()
files = [
    "/Users/sac/rocket-craft/ggen-validation-tests/core.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/reflection.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/blueprints.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/subsystems.ttl",
    "/Users/sac/rocket-craft/ggen-validation-tests/typestates.ttl"
]

for f in files:
    g.parse(f, format="turtle")

# We want to find why the ASK query returns False.
# The ASK query is:
# ASK {
#   FILTER NOT EXISTS {
#     ?world a/rdfs:subClassOf* ue4:UWorld ;
#            ue4:hasLevel ?level .
#     ?level ue4:hasActor ?actor .
#     ?actor ue4:bReplicates true .
#     FILTER NOT EXISTS {
#       ?world ue4:hasSubsystem ?subsystem .
#       ?subsystem a/rdfs:subClassOf* ue4:UNetworkingSubsystem .
#     }
#   }
# }
# If ASK is False, it means the FILTER NOT EXISTS failed, meaning there IS some ?world, ?level, ?actor that:
# has UWorld type, has level, level has actor, actor replicates true, AND:
# there is NO subsystem of type UNetworkingSubsystem.

q_debug = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?world ?level ?actor
WHERE {
    ?world a/rdfs:subClassOf* ue4:UWorld ;
           ue4:hasLevel ?level .
    ?level ue4:hasActor ?actor .
    ?actor ue4:bReplicates true .
    FILTER NOT EXISTS {
      ?world ue4:hasSubsystem ?subsystem .
      ?subsystem a/rdfs:subClassOf* ue4:UNetworkingSubsystem .
    }
}
"""

print("Debug Query Results:")
results = list(g.query(q_debug))
print(f"Number of violations found by debug query: {len(results)}")
for r in results:
    print(r)
