import os
import sys
from rdflib import Graph, Namespace, URIRef
from rdflib.namespace import RDF, RDFS, OWL

def main():
    ontology_dir = "/Users/sac/.ggen/packs/ue4_ontology"
    g = Graph()
    g.parse(os.path.join(ontology_dir, "core.ttl"), format="turtle")
    
    EX = Namespace("https://rocket-craft.io/example/")
    g.bind("ex", EX)
    
    # Introduce circular inheritance
    print("Introducing circular inheritance: ex:ClassA subClassOf ex:ClassB . ex:ClassB subClassOf ex:ClassA .")
    g.add((EX.ClassA, RDF.type, OWL.Class))
    g.add((EX.ClassB, RDF.type, OWL.Class))
    g.add((EX.ClassA, RDFS.subClassOf, EX.ClassB))
    g.add((EX.ClassB, RDFS.subClassOf, EX.ClassA))
    
    # Query to detect circular inheritance
    query_circular = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT DISTINCT ?cls WHERE {
        ?cls rdfs:subClassOf+ ?cls .
    }
    """
    
    results = list(g.query(query_circular))
    if results:
        print(f"Circular inheritance DETECTED: {[str(r.cls) for r in results]}")
    else:
        print("No circular inheritance detected.")

if __name__ == "__main__":
    main()
