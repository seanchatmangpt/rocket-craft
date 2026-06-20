import rdflib
import sys

# Load all TTL files
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

# Run query 1
q1 = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?world ?level ?actor ?subsystem ?subType
WHERE {
    ?world a/rdfs:subClassOf* ue4:UWorld ;
           ue4:hasLevel ?level .
    ?level ue4:hasActor ?actor .
    ?actor ue4:bReplicates true .
    OPTIONAL {
        ?world ue4:hasSubsystem ?subsystem .
        ?subsystem a ?subType .
    }
}
"""

print("Results for Q1:")
for r in g.query(q1):
    print(r)

print("\nRunning ASK query for RuleNetWorldSubsystemTopology:")
q_ask = """
PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
ASK {
  FILTER NOT EXISTS {
    ?world a/rdfs:subClassOf* ue4:UWorld ;
           ue4:hasLevel ?level .
    ?level ue4:hasActor ?actor .
    ?actor ue4:bReplicates true .
    FILTER NOT EXISTS {
      ?world ue4:hasSubsystem ?subsystem .
      ?subsystem a/rdfs:subClassOf* ue4:UNetworkingSubsystem .
    }
  }
}
"""
print(g.query(q_ask))
