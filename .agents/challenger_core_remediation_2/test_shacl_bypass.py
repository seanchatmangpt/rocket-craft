import os
import sys
from rdflib import Graph, Namespace, URIRef
from rdflib.namespace import RDF, RDFS, OWL

def main():
    ontology_dir = "/Users/sac/.ggen/packs/ue4_ontology"
    g = Graph()
    g.parse(os.path.join(ontology_dir, "core.ttl"), format="turtle")
    
    # Declare a class with opaque URI, but do NOT declare it as owl:Class
    opaque_class = URIRef("urn:private:opaqueClass")
    g.add((opaque_class, RDFS.subClassOf, URIRef("https://rocket-craft.io/ontology/ue4/AActor")))
    
    # Query matching Classes under targetClass rdfs:Class or owl:Class
    query = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX sh: <http://www.w3.org/ns/shacl#>
    
    SELECT ?subject WHERE {
        ?subject rdfs:subClassOf ?parent .
        FILTER(STRSTARTS(STR(?subject), "urn:"))
    }
    """
    results = list(g.query(query))
    print(f"Opaque subclass found in graph: {[str(r.subject) for r in results]}")
    
    # Check if SHACL targetClass owl:Class or rdfs:Class would target this subject
    # Since we did not add `urn:private:opaqueClass a owl:Class`, standard SHACL targetClass won't match it
    is_class_declared = (opaque_class, RDF.type, OWL.Class) in g or (opaque_class, RDF.type, RDFS.Class) in g
    print(f"Is the opaque subclass declared as owl:Class/rdfs:Class? {is_class_declared}")

if __name__ == "__main__":
    main()
