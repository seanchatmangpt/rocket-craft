import os
import sys
from rdflib import Graph, Namespace, URIRef
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
    
    # Load all turtle files
    for ttl in ttl_files:
        path = os.path.join(ontology_dir, ttl)
        if not os.path.exists(path):
            print(f"Error: {path} does not exist.")
            sys.exit(1)
        print(f"Loading {ttl}...")
        g.parse(path, format="turtle")
        
    print(f"Total loaded triples: {len(g)}")
    
    # Define Namespace
    UE4 = Namespace("https://rocket-craft.io/ontology/ue4/")
    g.bind("ue4", UE4)
    g.bind("rdfs", RDFS)
    g.bind("owl", OWL)
    g.bind("rdf", RDF)
    
    print("\n--- 1. Class Hierarchy Verification ---")
    class_query = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    
    SELECT ?cls ?parent WHERE {
        ?cls a owl:Class .
        OPTIONAL { ?cls rdfs:subClassOf ?parent . }
        FILTER(STRSTARTS(STR(?cls), "https://rocket-craft.io/ontology/ue4/"))
    } ORDER BY ?parent ?cls
    """
    
    results = g.query(class_query)
    print(f"{'Class':<30} | {'Parent Class':<30}")
    print("-" * 65)
    for row in results:
        cls_str = str(row.cls).replace("https://rocket-craft.io/ontology/ue4/", "ue4:")
        parent_str = str(row.parent).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.parent else "None"
        print(f"{cls_str:<30} | {parent_str:<30}")
        
    print("\n--- 2. Relationship / Property Verification ---")
    prop_query = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    
    SELECT ?property ?domain ?range ?inverse WHERE {
        VALUES ?property { ue4:isComponentOf ue4:isLevelOf ue4:owner }
        OPTIONAL { ?property rdfs:domain ?domain . }
        OPTIONAL { ?property rdfs:range ?range . }
        OPTIONAL { ?property owl:inverseOf ?inverse . }
    }
    """
    
    results = g.query(prop_query)
    print(f"{'Property':<20} | {'Domain':<25} | {'Range':<20} | {'InverseOf':<20}")
    print("-" * 95)
    for row in results:
        prop_str = str(row.property).replace("https://rocket-craft.io/ontology/ue4/", "ue4:")
        dom_str = str(row.domain).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.domain else "None"
        rng_str = str(row.range).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.range else "None"
        inv_str = str(row.inverse).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.inverse else "None"
        print(f"{prop_str:<20} | {dom_str:<25} | {rng_str:<20} | {inv_str:<20}")

    # Let's perform a sanity check on inverse relationships to ensure their counterparts exist and are consistent
    print("\n--- 3. Inverse Consistency Sanity Check ---")
    consistency_query = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    
    SELECT ?p1 ?p2 ?p1_dom ?p1_rng ?p2_dom ?p2_rng WHERE {
        ?p1 owl:inverseOf ?p2 .
        OPTIONAL { ?p1 rdfs:domain ?p1_dom . }
        OPTIONAL { ?p1 rdfs:range ?p1_rng . }
        OPTIONAL { ?p2 rdfs:domain ?p2_dom . }
        OPTIONAL { ?p2 rdfs:range ?p2_rng . }
    }
    """
    results = g.query(consistency_query)
    for row in results:
        p1_str = str(row.p1).replace("https://rocket-craft.io/ontology/ue4/", "ue4:")
        p2_str = str(row.p2).replace("https://rocket-craft.io/ontology/ue4/", "ue4:")
        p1_dom = str(row.p1_dom).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.p1_dom else "None"
        p1_rng = str(row.p1_rng).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.p1_rng else "None"
        p2_dom = str(row.p2_dom).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.p2_dom else "None"
        p2_rng = str(row.p2_rng).replace("https://rocket-craft.io/ontology/ue4/", "ue4:") if row.p2_rng else "None"
        
        # Check if Domain(p1) == Range(p2) and Range(p1) == Domain(p2)
        match_domain_range = (p1_dom == p2_rng) and (p1_rng == p2_dom)
        status = "PASSED" if match_domain_range else "FAILED"
        print(f"Inverse Pair: {p1_str} <-> {p2_str}")
        print(f"  {p1_str} domain: {p1_dom}, range: {p1_rng}")
        print(f"  {p2_str} domain: {p2_dom}, range: {p2_rng}")
        print(f"  Domain/Range swap validation: {status}")
        print()

if __name__ == "__main__":
    main()
