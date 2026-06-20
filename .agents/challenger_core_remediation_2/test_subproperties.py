import os
import sys
from rdflib import Graph, Namespace
from rdflib.namespace import RDF, RDFS, OWL

def main():
    ontology_dir = "/Users/sac/.ggen/packs/ue4_ontology"
    g = Graph()
    g.parse(os.path.join(ontology_dir, "core.ttl"), format="turtle")
    
    EX = Namespace("https://rocket-craft.io/example/")
    UE4 = Namespace("https://rocket-craft.io/ontology/ue4/")
    g.bind("ex", EX)
    g.bind("ue4", UE4)
    g.bind("rdfs", RDFS)
    
    # Let's add instance data using subproperties:
    # 1. ex:MyActor ue4:hasRootComponent ex:MyRootComp
    # Note: ue4:hasRootComponent is subproperty of ue4:hasComponent
    g.add((EX.MyActor, UE4.hasRootComponent, EX.MyRootComp))
    
    # 2. ex:MyWorld ue4:persistentLevel ex:MyPersistentLevel
    # Note: ue4:persistentLevel is subproperty of ue4:hasLevel
    g.add((EX.MyWorld, UE4.persistentLevel, EX.MyPersistentLevel))
    
    print("Testing standard query (no subproperty matching):")
    q1 = """
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    SELECT ?actor ?component WHERE {
        ?actor ue4:hasComponent ?component .
    }
    """
    res1 = list(g.query(q1))
    print(f"  Standard query found: {len(res1)} matches (Expected 0 if no subproperty reasoning)")
    
    print("\nTesting subproperty-aware query (using rdfs:subPropertyOf*):")
    q2 = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    SELECT ?actor ?component WHERE {
        ?prop rdfs:subPropertyOf* ue4:hasComponent .
        ?actor ?prop ?component .
    }
    """
    res2 = list(g.query(q2))
    print(f"  Subproperty-aware query found: {len(res2)} matches")
    for row in res2:
        print(f"    {row.actor.n3(g.namespace_manager)} -> {row.component.n3(g.namespace_manager)}")
        
    print("\nTesting subproperty-aware query for level:")
    q3 = """
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    PREFIX ue4: <https://rocket-craft.io/ontology/ue4/>
    SELECT ?world ?level WHERE {
        ?prop rdfs:subPropertyOf* ue4:hasLevel .
        ?world ?prop ?level .
    }
    """
    res3 = list(g.query(q3))
    print(f"  Subproperty-aware level query found: {len(res3)} matches")
    for row in res3:
        print(f"    {row.world.n3(g.namespace_manager)} -> {row.level.n3(g.namespace_manager)}")

if __name__ == "__main__":
    main()
