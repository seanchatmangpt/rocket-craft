import os
import sys
from rdflib import Graph, Namespace
from rdflib.namespace import RDF, RDFS, OWL

def main():
    ontology_dir = "/Users/sac/.ggen/packs/ue4_ontology"
    ttl_files = [
        "core.ttl",
        "reflection.ttl",
        "blueprints.ttl",
        "subsystems.ttl",
        "typestates.ttl"
    ]
    
    g = Graph()
    for ttl in ttl_files:
        path = os.path.join(ontology_dir, ttl)
        g.parse(path, format="turtle")
        
    print(f"Loaded {len(g)} triples.")
    
    # Query classes lacking labels
    query_no_label = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    SELECT ?cls WHERE {
        ?cls a owl:Class .
        FILTER NOT EXISTS { ?cls rdfs:label ?label }
        FILTER(STRSTARTS(STR(?cls), "https://rocket-craft.io/ontology/ue4/"))
    }
    """
    
    no_labels = list(g.query(query_no_label))
    if no_labels:
        print("FAIL: Classes without rdfs:label:")
        for row in no_labels:
            print(f"  {row.cls}")
        sys.exit(1)
    else:
        print("PASS: All classes have rdfs:label.")
        
    # Query classes lacking comments
    query_no_comment = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    SELECT ?cls WHERE {
        ?cls a owl:Class .
        FILTER NOT EXISTS { ?cls rdfs:comment ?comment }
        FILTER(STRSTARTS(STR(?cls), "https://rocket-craft.io/ontology/ue4/"))
    }
    """
    no_comments = list(g.query(query_no_comment))
    if no_comments:
        print("WARNING: Classes without rdfs:comment:")
        for row in no_comments:
            print(f"  {row.cls}")
    else:
        print("PASS: All classes have rdfs:comment.")

    # Check namespace sanity: verify no subject has urn:private: or similar opaque URI
    query_opaque = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    SELECT ?subject WHERE {
        ?subject ?p ?o .
        FILTER(STRSTARTS(STR(?subject), "urn:"))
    }
    """
    opaque_subjects = list(g.query(query_opaque))
    if opaque_subjects:
        print("FAIL: Found opaque subjects:")
        for row in opaque_subjects:
            print(f"  {row.subject}")
        sys.exit(1)
    else:
        print("PASS: Namespace sanity (no urn: subjects).")

if __name__ == "__main__":
    main()
